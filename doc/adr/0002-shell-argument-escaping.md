# ADR 2: Shell Argument Escaping in Exec Command

## Status

Proposed

## Context

Issue [#215](https://github.com/rustworkshop/gitopolis/issues/215) reported that running `gitopolis exec -t tim -- echo 'oh " no'` results in an "Unterminated quoted string" error. This revealed a fundamental tension in how the `exec` command handles arguments.

### How Shell Execution Works in Gitopolis

The `exec` command passes arguments to a shell (`sh -c` on Unix, `cmd /C` on Windows). The original implementation simply joined all arguments with spaces:

```rust
let command_string = exec_args.join(" ");
// Then passes to: sh -c "echo oh \" no"
```

### The Problem

When a user types `gitopolis exec -- echo 'oh " no'`, the user's shell processes the quotes first:
1. User shell parses: `echo 'oh " no'` → gitopolis receives: `["echo", "oh \" no"]`
2. Gitopolis joins with spaces: `echo oh " no`
3. This gets passed to `sh -c "echo oh " no"`
4. The inner shell sees an unterminated quote and errors

### Two Conflicting Use Cases

The exec command needs to support two fundamentally different patterns:

**Use Case 1: Multiple separate arguments**
```bash
gitopolis exec -- echo 'oh " no'
# User's shell gives gitopolis: ["echo", "oh \" no"]
# User expects: the literal string "oh \" no" to be echoed
```

**Use Case 2: Single shell command with syntax**
```bash
gitopolis exec -- 'echo test output | sort'
# User's shell gives gitopolis: ["echo test output | sort"]
# User expects: echo's output to be piped to sort
```

Both are valid! The first needs each argument properly escaped. The second needs the shell to interpret pipes, redirects, etc.

### Key Insight: Pipes Are Not Passed Through

Initially, I misunderstood how shell pipes interact with gitopolis. When a user types:
```bash
gitopolis exec -- echo test | sort
```

The **outer shell** processes the pipe BEFORE gitopolis runs:
- gitopolis receives: `["echo", "test"]`
- gitopolis's OUTPUT gets piped to sort by the outer shell

For pipes to work **inside** gitopolis (i.e., in each repo), the user must quote:
```bash
gitopolis exec -- 'echo test | sort'
```

This gives gitopolis: `["echo test | sort"]` as a single argument.

## Decision

Implement **conditional escaping** based on argument count:

1. **Single argument**: Pass through unmodified to allow shell syntax (pipes, redirects, etc.)
2. **Multiple arguments**: Escape each argument individually to prevent injection

### Implementation

```rust
let command_string = if exec_args.len() == 1 {
    exec_args[0].clone()  // Allow shell syntax
} else {
    exec_args
        .iter()
        .map(|arg| shell_escape(arg))  // Prevent injection
        .collect::<Vec<_>>()
        .join(" ")
};
```

### Shell Escaping Strategy

**Unix (POSIX shells):**
- Wrap arguments in single quotes: `'argument'`
- Escape embedded single quotes using `'\''` pattern
- Example: `oh ' no` → `'oh '\'' no'`

**Windows (cmd.exe):**
- Wrap arguments in double quotes: `"argument"`
- Escape embedded double quotes by doubling them: `""`
- Example: `oh " no` → `"oh "" no"`

## Consequences

### Positive

1. **Fixes the quote injection bug**: Arguments with special characters are properly escaped
2. **Preserves shell features**: Single-argument commands can still use pipes, redirects, etc.
3. **Backward compatible**: Existing tests pass, including piping tests
4. **Security improvement**: Reduces risk of shell injection attacks

### Negative

1. **Subtle behavior difference**: Users must understand the single-arg vs multi-arg distinction
2. **Documentation burden**: Need to explain when to quote and when not to
3. **Potential confusion**: The behavior changes based on how the user quotes their command

### User Guidance Needed

Users need to know:

**For literal arguments with special characters:**
```bash
# Multiple arguments - each is escaped automatically
gitopolis exec -- echo 'argument with "quotes"'
# gitopolis receives: ["echo", "argument with \"quotes\""]
# gitopolis executes: 'echo' 'argument with "quotes"'
# Output: argument with "quotes"
```

**For shell commands with syntax (pipes inside repos):**
```bash
# Single argument - passed through for shell interpretation
gitopolis exec -- 'echo hello | sort'
# gitopolis receives: ["echo hello | sort"]
# gitopolis executes: echo hello | sort
# Output: hello (piped through sort in each repo)
```

**For simple commands:**
```bash
# Multiple arguments - each is escaped automatically
gitopolis exec -- git status
# gitopolis receives: ["git", "status"]
# gitopolis executes: 'git' 'status'
# Works as expected
```

**For combined cases (pipes inside AND outside):**
```bash
# Single argument to gitopolis, then outer pipe processes gitopolis output
gitopolis exec -t tim -- 'echo "hey there" | wc' | sort
# gitopolis receives: ["echo \"hey there\" | wc"]
# gitopolis executes in each repo: echo "hey there" | wc
# Each repo outputs word count, then ALL outputs are piped to outer sort
```

**Edge case - argument with embedded quotes AND pipes:**
```bash
# Single argument with both quotes and pipes - shell interprets everything
gitopolis exec -- 'echo "foo | bar" | grep bar'
# gitopolis receives: ["echo \"foo | bar\" | grep bar"]
# gitopolis executes: echo "foo | bar" | grep bar
# Output: foo | bar (the literal string, not a pipe, then grepped)
```

**Edge case - multiple arguments with quotes:**
```bash
# Multiple arguments - the reported issue from #215
gitopolis exec -- echo 'oh " no'
# gitopolis receives: ["echo", "oh \" no"]
# gitopolis executes: 'echo' 'oh " no'
# Output: oh " no (FIXED by this ADR)
```

## Alternatives Considered

### Alternative 1: Always Escape Everything

Escape all arguments regardless of count.

**Rejected because**: Breaks shell syntax features that users rely on (see test cases `exec_shell_piping` and `exec_command_oneline_with_piping`).

### Alternative 2: Never Escape Anything

Keep the original `join(" ")` behavior.

**Rejected because**: Allows shell injection and causes unterminated quote errors (Issue #215).

### Alternative 3: Detect Shell Operators

Only escape arguments that don't contain shell operators like `|`, `>`, `&`, etc.

**Rejected because**:
- Complex heuristic that's error-prone
- Can't distinguish between literal `|` and pipe operator
- User intent is ambiguous

### Alternative 4: Add a Flag

Add `--raw` flag to disable escaping, or `--safe` to enable it.

**Rejected because**:
- Adds complexity to the CLI
- Most users won't understand when to use which
- The argument count heuristic achieves the same goal implicitly

## Testing

Added test case `exec_with_nested_quotes` to verify the fix:
```rust
gitopolis exec -- echo "oh \" no"
// Should successfully output: oh " no
```

Existing tests verify backward compatibility:
- `exec_shell_piping`: Single-arg command with pipe works
- `exec_command_oneline_with_piping`: Single-arg piped command works
- `exec_shell_quoted_args`: Quoted arguments work correctly
- All other exec tests continue to pass

## Notes

This decision is a pragmatic compromise. The fundamental issue is that `sh -c` expects a command string, but we receive pre-parsed arguments from clap. There's no perfect solution that handles all cases intuitively.

The single-argument exception is based on the observation that:
1. Shell syntax (pipes, redirects) naturally forms a single argument when properly quoted
2. Multiple arguments represent discrete values that should be escaped for safety
3. This matches user expectations in practice (as evidenced by existing test cases)

### Potential Weaknesses of the Single-Argument Approach

The single-argument heuristic is **not perfect** and has edge cases:

**Case 1: Single argument that should be escaped**
```bash
# User wants to echo a literal string containing shell syntax
gitopolis exec -- 'echo "this | is | not | a | pipe"'
# Works fine - echoes the literal string
```
This works because the inner shell interprets the double quotes correctly.

**Case 2: Single argument with unbalanced quotes**
```bash
# What if the single argument itself has quote issues?
gitopolis exec -- 'echo "unterminated'
# Still fails - but now at the inner shell level
# The single-arg exception doesn't help here
```
The user's outer shell processes the quotes first, so this might not even reach gitopolis.

**Case 3: Command injection via single argument**
```bash
# If a single argument comes from untrusted input (unlikely in CLI usage)
gitopolis exec -- "$USER_INPUT"
# Could allow shell injection if USER_INPUT contains malicious commands
# However, this is a shell security issue, not gitopolis-specific
```
This is mitigated by the fact that gitopolis is a CLI tool - the user controls all input.

**The key question**: Are there realistic user scenarios where:
1. A single argument is provided (so no escaping happens)
2. That argument contains characters that SHOULD be escaped
3. But the user did NOT intend shell interpretation

We haven't identified such a case yet. The examples above either work correctly or represent fundamental shell security issues beyond gitopolis's control.

## References

- Issue #215: https://github.com/rustworkshop/gitopolis/issues/215
- POSIX shell quoting: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02
- Windows cmd.exe escaping: https://ss64.com/nt/syntax-esc.html
