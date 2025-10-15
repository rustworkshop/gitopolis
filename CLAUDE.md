# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Building and Testing
- **Build**: `cargo build`
- **Run tests**: `cargo test`
- **Run a specific test**: `cargo test test_name`
- **Run end-to-end tests**: `cargo test --test end_to_end_tests`
- **Run gitopolis tests**: `cargo test --test gitopolis_tests`

### Linting and Formatting
- **Run all lints**: `./lint.sh` (runs cargo fmt, clippy with harsh settings, cargo deny license check, and yamllint for GitHub workflows)
- **Format code**: `cargo fmt`
- **Run clippy**: `./clippy-harsh.sh` or `cargo clippy --all-targets --all-features -- -D warnings`
- **Check licenses**: `cargo deny check licenses`
- **Check unused dependencies**: `cargo machete`

### Development
- **Install CI tools locally**: `./ci-tool-setup.sh` (installs cargo-deny and yamllint)
- **Upgrade dependencies**: `./upgrades.sh` (upgrades both Rust version and crate dependencies)

## Architecture

Gitopolis is a CLI tool for managing multiple git repositories, built in Rust with a clean separation of concerns:

### Core Components

- **main.rs**: CLI interface using clap, bridges between console and logic, injects dependencies
- **lib.rs**: Module aggregation point for the library
- **gitopolis.rs**: Core business logic with injected dependencies (storage, git operations, stdout)
- **repos.rs**: Domain models for repository state management
- **exec.rs**: Command execution across multiple repositories
- **storage.rs**: Trait for state persistence (currently using TOML file `.gitopolis.toml`)
- **git.rs**: Trait for git operations (using git2 crate)

### Testing Strategy

- **End-to-end tests** (`tests/end_to_end_tests.rs`): Test complete workflows with real file system and git repos
- **Unit tests** (`tests/gitopolis_tests.rs`): Test core logic with mocked dependencies (storage, git)

### Key Design Patterns

- Dependency injection through traits for testability (Storage, Git traits)
- Clean separation between CLI parsing and business logic
- State management through `.gitopolis.toml` file with TOML serialization
- Command pattern for different operations (add, remove, exec, tag, clone, list)

### Important Notes

- The tool uses `git2` crate for git operations with vendored OpenSSL
- Windows has special handling for glob expansion (not done by shell)
- Supports multiple git remotes per repository
- Rust version is pinned in `.tool-versions` and used by CI workflows