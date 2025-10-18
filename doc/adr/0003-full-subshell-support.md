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
