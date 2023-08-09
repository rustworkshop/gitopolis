#!/bin/sh -v
set -e # exit on error
cargo update
cargo upgrade
cargo test
git commit -am "cargo update/upgrade"
