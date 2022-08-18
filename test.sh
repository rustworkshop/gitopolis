#!/bin/sh
# Hacky smoke test of whole thing, needs making proper
set -e # stop on error

# Arrange - make test folder & repos
src=`pwd`
exe="$src/target/debug/gitopolis"
cd /tmp/

test_folder=gitopolis_test
if [ -d "$test_folder"  ]; then
	rm -rf "$test_folder"
fi
mkdir "$test_folder"
cd "$test_folder"

(
	mkdir foo
	cd foo
	git init >> /dev/null
	git remote add origin "git@example.org/some_repo.git"
)

# Act - try various commands
echo "$exe help"
eval "$exe help"
echo

echo "$exe add foo"
eval "$exe add foo"
eval "$exe tag RED foo"
echo
echo "====== .gitopolis.toml ======"
cat .gitopolis.toml
echo

echo "$exe list"
eval "$exe list"
eval "$exe tag -r RED foo"
echo
echo "====== .gitopolis.toml ======"
cat .gitopolis.toml
echo

echo "$exe remove foo"
eval "$exe remove foo"
echo "====== .gitopolis.toml ======"
cat .gitopolis.toml
