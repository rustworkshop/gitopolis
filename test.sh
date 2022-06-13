#!/bin/sh
set -e

if [ -f .gitopolis.toml ]; then
	rm .gitopolis.toml
fi
cargo run help
cargo run add x y z
cargo run list
cat .gitopolis.toml
