#!/bin/sh -v
set -e # exit on error
asdf plugin update rust
latest=`asdf list all rust | tail -n 2`
echo $latest
asdf install rust $latest
asdf set rust $latest
cargo test
git commit -i .tool-versions -m "chore: Upgrade build to latest rust"
