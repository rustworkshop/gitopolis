#!/bin/sh -v
set -e # exit on error
cargo fmt
./clippy-harsh.sh
cargo deny check licenses
