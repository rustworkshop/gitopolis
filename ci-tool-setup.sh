#!/bin/sh -v
# Tools needed to run the ci checks locally, only needed when debugging ci failures
cargo install --locked cargo-deny
pip install --user yamllint
