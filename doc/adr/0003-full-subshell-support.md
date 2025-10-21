# ADR-0003: Full subshell support

**Date:** 2025-10-17
**Status:** Accepted
**Related Issues:**

ADR for implementing [#13 – Make `gitopolis exec` honor aliases and shell functions](https://github.com/rustworkshop/gitopolis/issues/13?utm_source=chatgpt.com)

ADR generated from [discussion with GPT exploring possible solutions](https://chatgpt.com/share/68f1974b-95b4-8006-8fff-72687d05d3cf).

## Context

`gitopolis exec` currently runs external commands without awareness of user-defined aliases or shell functions.
Users expect it to behave like their terminal — resolving aliases, functions, and environment automatically.

Initial designs proposed multiple flags (`--no-profile`, `--shell`, `--login`, etc.) and complex shell detection logic.
However, the guiding principle for `gitopolis` is **KISS**, **zero configuration**, and **pit of success** defaults.

We also agreed that TTY passthrough and reproducibility flags (like `--no-profile`) are separate concerns and should not be mixed into this design.

Related issue: [#209 – Handle TTY passthrough separately](https://github.com/rustworkshop/gitopolis/issues/209)

Related Commit: [6e11734b15d4a989b34bec6dd1fc21b42a630c32 – Fixed command escaping](https://github.com/rustworkshop/gitopolis/commit/6e11734b15d4a989b34bec6dd1fc21b42a630c32)

## Decision

`gitopolis exec` will execute commands through the **current user shell**, with **minimal detection and zero configuration**.

### Behavior Summary

1. **Shell Detection**
    * If `$SHELL` is set → use that.
    * On Windows:
        * If `$SHELL` exists (MSYS2, Git Bash, Cygwin) → use it.
        * Else if PowerShell environment (`PSModulePath` exists) → use `pwsh -NoLogo -c`.
        * Else → fallback to `cmd /s /c`.
    * On POSIX systems where `$SHELL` is unset → fallback to `/bin/sh -c`.
2. **Invocation**
    * **POSIX shells:**
        `"$SHELL" -i -c "<command>"`
        * `-i` is used only when running interactively.
    * **PowerShell:**
        `pwsh -NoLogo -c "<command>"`
    * **CMD:**
        `cmd /s /c "<command>"`
3. **Escaping**
    * Handled using the improved escaping logic from [commit 6e11734b15d4a989b34bec6dd1fc21b42a630c32](https://github.com/rustworkshop/gitopolis/commit/6e11734b15d4a989b34bec6dd1fc21b42a630c32).
4. **No configuration or flags**
    * No `--no-profile`, `--shell`, or other customization flags.
    * No TTY passthrough logic (handled under [#209](https://github.com/rustworkshop/gitopolis/issues/209)).


## Rationale

* **Zero configuration:** It “just works” with aliases, functions, and environment variables.
* **Pit of success:** Default behavior matches what users expect from their normal shell.
* **Predictable fallback:** `/bin/sh` and `cmd.exe` ensure universal compatibility.
* **Simplicity:** No premature optimization or unnecessary flags.
* **Cross-platform naturalness:** Works seamlessly under MSYS2, Git Bash, WSL, CMD, and PowerShell.

## Alternatives considered

1. Leave as is - already causing problems with lack of support for shell aliases
2. Check ComSpec on windows - dropped because the generated fallback to cmd was not modified by this check

## Consequences

### Positive

* Users’ shell environments behave identically inside and outside `gitopolis exec`.
* Minimal implementation complexity.
* Compatible across Unix, macOS, WSL, and Windows-native shells.


### Negative / Deferred

* No explicit way to disable profiles or enforce reproducibility (not yet needed).
* TTY handling remains a separate concern ([#209](https://github.com/rustworkshop/gitopolis/issues/209)).


## Example Behavior

| Environment | `$SHELL` | Invocation | Result |
| --- | --- | --- | --- |
| Bash, Zsh, Fish | `/usr/bin/bash` | `bash -i -c "<cmd>"` | aliases/functions load |
| Git Bash / MSYS2 | `/usr/bin/bash` | `bash -i -c "<cmd>"` | behaves as expected |
| PowerShell | _(no `$SHELL`)_ | `pwsh -NoLogo -c "<cmd>"` | works in native PS |
| CMD | _(no `$SHELL`)_ | `cmd /s /c "<cmd>"` | standard Windows behavior |
| Minimal POSIX (no `$SHELL`) | _(unset)_ | `/bin/sh -c "<cmd>"` | still executes cleanly |

## Implementation Sketch (Rust)

```rust
fn detect_shell() -> Vec<String> {
    if let Ok(shell) = std::env::var("SHELL") {
        return vec![shell, "-i".into(), "-c".into()];
    }

    #[cfg(windows)]
    {
        use std::env;
        if env::var("SHELL").is_ok() {
            return vec![env::var("SHELL").unwrap(), "-i".into(), "-c".into()];
        }
        if env::var("PSModulePath").is_ok() {
            return vec!["pwsh".into(), "-NoLogo".into(), "-c".into()];
        }
        return vec!["cmd".into(), "/s".into(), "/c".into()];
    }

    vec!["/bin/sh".into(), "-c".into()]
}
```

## Status

**Accepted and to be implemented** in line with [#13](https://github.com/rustworkshop/gitopolis/issues/13?utm_source=chatgpt.com).
TTY passthrough improvements will follow separately under [#209](https://github.com/rustworkshop/gitopolis/issues/209).

## Multi-Shell Testing Strategy

**Date Added:** 2025-10-20
**Status:** Proposed

### Context

Windows CI failures revealed that the shell detection works correctly (detecting PowerShell via `PSModulePath`), but our tests were written assuming cmd.exe behavior. The tests also use Unix commands (`ls`, `cat`) that don't exist in native PowerShell.

Key findings:
1. GitHub Actions Windows runners have `PSModulePath` set, so our code correctly detects and uses PowerShell
2. Error messages differ across shells:
   - cmd.exe: "not recognized as an internal or external command"
   - PowerShell: "The term 'X' is not recognized as a name of a cmdlet, function, script file, or executable program"
   - Unix shells: "not found"
3. Command availability varies:
   - `ls`, `cat` exist in Unix/Git Bash but not native PowerShell
   - PowerShell equivalents: `Get-ChildItem`, `Get-Content`
4. Our positional parameters approach (`"$@"`) only works on POSIX-like shells

### Testing Philosophy

All tests must run on all supported shell environments to catch behavioral differences and platform-specific issues. We must NOT skip tests based on platform - instead, tests should adapt their expectations to the shell environment.

### Decision

Expand CI to test three Windows shell configurations and update test infrastructure to support shell-aware testing.

#### CI Matrix Expansion

Add Windows shell variants to the CI build matrix:

```yaml
- os: windows-latest
  target: x86_64-pc-windows-msvc
  artifact_name: gitopolis-windows-x86_64
  binary_name: gitopolis.exe
  shell_env: cmd
  shell_setup: |
    # Clear SHELL and PSModulePath to force cmd.exe
    echo "SHELL=" >> $GITHUB_ENV
    echo "PSModulePath=" >> $GITHUB_ENV

- os: windows-latest
  target: x86_64-pc-windows-msvc
  artifact_name: gitopolis-windows-powershell-x86_64
  binary_name: gitopolis.exe
  shell_env: powershell
  shell_setup: |
    # Ensure PSModulePath is set to force PowerShell detection
    # (This is usually already set on GitHub Actions)

- os: windows-latest
  target: x86_64-pc-windows-msvc
  artifact_name: gitopolis-windows-bash-x86_64
  binary_name: gitopolis.exe
  shell_env: bash
  shell_setup: |
    # Set SHELL to Git Bash (comes with Git for Windows)
    echo "SHELL=C:\Program Files\Git\bin\bash.exe" >> $GITHUB_ENV
```

#### Test Infrastructure Changes

Create shell-aware test helpers in `tests/end_to_end_tests.rs`:

```rust
enum ShellEnvironment {
    Bash,      // Unix shells, Git Bash, MSYS2
    Cmd,       // Windows cmd.exe
    PowerShell // Windows PowerShell/pwsh
}

fn detect_test_shell_environment() -> ShellEnvironment {
    // Detect which shell gitopolis will use based on environment
    if env::var("SHELL").is_ok() {
        return ShellEnvironment::Bash;
    }

    #[cfg(windows)]
    {
        if env::var("PSModulePath").is_ok() {
            return ShellEnvironment::PowerShell;
        }
        return ShellEnvironment::Cmd;
    }

    #[cfg(not(windows))]
    ShellEnvironment::Bash
}

// Shell-specific command helpers
fn get_list_command(shell: &ShellEnvironment) -> &'static str {
    match shell {
        ShellEnvironment::Bash | ShellEnvironment::Cmd => "ls",
        ShellEnvironment::PowerShell => "Get-ChildItem",
    }
}

fn get_cat_command(shell: &ShellEnvironment) -> &'static str {
    match shell {
        ShellEnvironment::Bash | ShellEnvironment::Cmd => "cat",
        ShellEnvironment::PowerShell => "Get-Content",
    }
}

fn get_invalid_command_error(shell: &ShellEnvironment) -> &'static str {
    match shell {
        ShellEnvironment::Bash => "not found",
        ShellEnvironment::Cmd => "not recognized as an internal or external command",
        ShellEnvironment::PowerShell => "is not recognized as a name of a cmdlet",
    }
}
```

#### Test Categorization

**Shell-Agnostic Tests** - Use commands that work everywhere:
- `git` commands (universally available)
- `echo` commands (works in all shells)
- Tests that verify core gitopolis functionality without shell-specific commands

**Shell-Specific Tests** - Parameterized for correct commands per shell:
- Tests using `ls`, `cat`, `grep`, etc. should use helper functions
- Error message assertions should use `get_invalid_command_error()`
- Commands with arguments should use shell-appropriate quoting

**Cross-Shell Compatibility Tests** - Verify positional parameters work across shells:
- Multi-argument command execution
- Quote handling in arguments
- Special character handling

### Implementation Plan

1. **Phase 1: Test Infrastructure**
   - Add `ShellEnvironment` enum and detection function
   - Add shell-specific command helper functions
   - Add shell-specific error message helpers

2. **Phase 2: Update Existing Tests**
   - Replace hardcoded `ls`, `cat` with helper functions
   - Replace hardcoded error messages with helper functions
   - Ensure all tests can run on all shell environments

3. **Phase 3: CI Matrix Expansion**
   - Add cmd, PowerShell, and Git Bash configurations to Windows CI
   - Verify all tests pass on all shell environments
   - Update artifact naming to distinguish shell variants (for release builds, keep single artifact)

4. **Phase 4: Documentation**
   - Document shell support matrix
   - Add troubleshooting guide for shell-specific issues
   - Update test README with shell testing guidelines

### Consequences

#### Positive

- **Comprehensive Coverage**: Tests verify behavior on all supported shell environments
- **Early Detection**: Platform-specific issues caught in CI before release
- **Better Confidence**: Multi-shell testing ensures our positional parameters approach works everywhere
- **Clear Expectations**: Tests document expected behavior per shell

#### Negative

- **CI Time**: Additional Windows configurations increase CI duration
- **Test Complexity**: Tests need shell-aware logic instead of simple assertions
- **Maintenance**: Shell-specific helpers need updates when adding new shell support

### Open Questions

1. Should we test additional shells (fish, nushell, zsh) or focus on the most common ones?
2. Should release artifacts include shell type in name, or keep single Windows binary?
3. How do we handle shells that don't support our positional parameters approach (`"$@"`)?

### References

- GitHub Actions run with CI failures: https://github.com/rustworkshop/gitopolis/actions/runs/18636759293/job/53128991888
- Failed tests: exec_invalid_command, exec_oneline_multiline_output, exec_non_zero, exec_oneline_multiple_args_with_single_quotes, exec_oneline_with_special_chars, exec_oneline_non_zero, exec_with_special_chars
- Related: Issue #209 (TTY passthrough)
- Related: Issue #215 (Quote handling in exec commands)
