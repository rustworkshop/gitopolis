#!/bin/sh -v
# run clippy the same way as github actions
cargo clippy --all-targets --all-features -- -D warnings
