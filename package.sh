#!/bin/sh -v
set -e
# used by .github/workflows/release.yml

tar -czvf gitopolis.tar.gz target/*

