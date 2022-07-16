#!/bin/sh -v
set -e

if [ -f .gitopolis.toml ]; then
	rm .gitopolis.toml
fi
cargo run help
cargo run add foo bar "baz aroony" deleteme
cargo run list
cargo run remove deleteme
cargo run tag red foo bar
cargo run tag deadtag bar
cargo run tag -r deadtag bar
cat .gitopolis.toml
