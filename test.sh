#!/bin/sh -v
set -e

if [ -f .gitopolis.toml ]; then
	rm .gitopolis.toml
fi
cargo run help
cargo run add foo bar "baz aroony" buzz
cargo run list
cargo run remove buzz
cargo run tag red foo bar
cargo run tag green bar
cat .gitopolis.toml
