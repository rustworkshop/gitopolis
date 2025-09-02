#!/bin/sh -v
cargo clippy
cargo deny check licenses
