#!/bin/sh -v
set -e # exit on error
cargo fmt
./clippy-harsh.sh
cargo deny check licenses

# Check YAML files in .github
if command -v yamllint >/dev/null 2>&1; then
    echo "Checking GitHub YAML files..."
    for file in .github/**/*.yml .github/**/*.yaml; do
        if [ -f "$file" ]; then
            echo "Checking $file"
            yamllint -d relaxed "$file"
        fi
    done
else
    echo "yamllint not installed, skipping YAML checks"
fi
