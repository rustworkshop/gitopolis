#!/bin/sh -v
set -e # exit on error
asdf plugin update rust
latest=`asdf list all rust | tail -n 1`
echo $latest
asdf install rust $latest
asdf local rust $latest
cargo test
git commit -i .tool-versions -m "Upgrade rust to latest"
