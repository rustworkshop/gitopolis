# ADR-0001: Execute All Commands Through Shell

## Status

Proposed

## Context

The current `gitopolis exec` command executes commands directly using Rust's `Command::new()`, which bypasses the shell entirely. This prevents users from using standard shell features like pipes (`|`), redirection (`>`, `<`), command substitution, and other shell constructs that are essential for complex operations.

Issue #170 highlights this limitation with a simple example:
```bash
gitopolis exec -- git branch -r | wc -l
```

Currently, this gives you the total count of ALL output from ALL repositories, not the count per repository. The pipe operates on gitopolis's output, not within each repository's command execution.

### Problems with Current Direct Execution

1. **No shell constructs within repos** - Pipes, redirections, globbing, command substitution don't work inside each repository's command
2. **Limited composability** - Can't chain commands with standard Unix tools per repository
3. **User expectations** - Most users expect shell-like behavior when running commands
4. **Workaround friction** - Users must wrap commands in explicit shell invocations like `sh -c`

## Decision

**Replace direct command execution with shell execution for all `exec` commands.**

All commands will be executed through the system shell:
- Unix/Linux/macOS: `/bin/sh -c "command"`
- Windows: `cmd /C "command"`

This is a breaking change that aligns the tool with user expectations and enables full shell functionality.

### Implementation Approach

```rust
// Simplified execution - always use shell
fn repo_exec(path: &str, exec_args: &Vec<String>) -> Result<ExitStatus, Error> {
    let command_string = exec_args.join(" ");

    #[cfg(unix)]
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command_string)
        .current_dir(path)
        .spawn()?;

    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .arg("/C")
        .arg(&command_string)
        .current_dir(path)
        .spawn()?;

    child.wait()
}
```

## Consequences

### Positive

1. **Full shell power** - All shell constructs work as expected
2. **User-friendly** - Matches user expectations from other tools
3. **Simpler mental model** - One consistent execution mode
4. **Enhanced productivity** - Complex operations become possible
5. **Standard behavior** - Similar to how `make`, `npm scripts`, and other tools work

### Negative

1. **Breaking change** - Commands with special characters that were previously escaped may behave differently
2. **Shell dependency** - Requires a shell to be present (though this is universal)
3. **Security considerations** - Shell injection risks if handling untrusted input
4. **Performance overhead** - Minor overhead from shell process (negligible in practice)
5. **Platform differences** - Shell syntax varies between Unix and Windows

### Migration Impact

Users may need to adjust commands that:
- Include literal special characters (now need escaping)
- Rely on exact argument passing without shell interpretation

However, most common use cases will work better after this change.

## Alternatives Considered

### Keep Direct Execution with Optional Shell Mode
- **Rejected**: Maintaining two modes adds complexity and confusion. Most users want shell features by default.

### Custom Shell Parser
- **Rejected**: Reimplementing shell parsing is complex and error-prone. Better to use the actual shell.

### Use a Cross-Platform Shell (like bash via Git Bash on Windows)
- **Rejected**: Adds dependencies and complexity. Native shells are sufficient.

## Implementation Plan (TDD Approach)

1. **Phase 1: Write Failing Tests**
   - Add test for basic piping: `git branch -r | wc -l`
   - Add test for the gold standard: `gitopolis exec --oneline -- 'git branch -r | wc -l' | sort -n`
   - Add test for redirection: `git log > output.txt`
   - Add test for command chaining: `git fetch && git pull`
   - Add test for command substitution: `echo $(git rev-parse HEAD)`

2. **Phase 2: Implementation**
   - Modify `repo_exec()` to use shell execution
   - Modify `repo_exec_oneline()` to use shell execution
   - Ensure proper command string construction
   - Handle both Unix and Windows platforms

3. **Phase 3: Verify Tests Pass**
   - All piping tests should pass
   - The gold standard test must work: output from gitopolis should be pipeable to external commands
   - Cross-platform testing (Unix/Windows)

4. **Phase 4: Documentation**
   - Update README with examples
   - Document the breaking change prominently
   - Provide migration guide for affected use cases

## Examples

### Before (Direct Execution)
```bash
# Gives total count across all repos, not per-repo count
gitopolis exec -- git branch -r | wc -l

# Current workaround - explicitly invoke shell
gitopolis exec -- sh -c "git branch -r | wc -l"
```

### After (Shell Execution)
```bash
# Each repo runs the piped command internally
gitopolis exec -- 'git branch -r | wc -l'

# Gold standard test case - sortable numeric output
gitopolis exec --oneline -- 'git branch -r | wc -l' | sort -n

# Complex operations work per-repository
gitopolis exec -- 'git log --oneline | head -5'
gitopolis exec -- 'git status && git diff --stat'
gitopolis exec -- "find . -name '*.rs' | xargs wc -l"
```

## Security Notes

When accepting user input that will be passed to exec:
- Be aware of shell injection risks
- Consider validating or sanitizing input if from untrusted sources
- Document security considerations for users

## References

- Issue #170: Support for piping multiple commands with exec
- Similar tools: GNU Parallel, xargs, make (all use shell execution)
- Security: OWASP Command Injection documentation