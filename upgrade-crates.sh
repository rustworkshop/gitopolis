#!/bin/sh -v
set -e # exit on error
cargo update
cargo install cargo-edit
cargo upgrade # from cargo-edit
cargo test
git commit -i Cargo.lock -i Cargo.toml -m "cargo update/upgrade"
