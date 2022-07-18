# Gitopolis

[![main](https://github.com/timabell/gitopolis/actions/workflows/main.yml/badge.svg)](https://github.com/timabell/gitopolis/actions/workflows/main.yml)

Manage multiple git repositories, like [gita](https://github.com/nosarthur/gita) but written in [Rust](https://www.rust-lang.org/) so you don't need python etc to run it.

It's intended to not know too much about git and just delegate everything to the `git` command which assumed to be available on the path.

## The name

Think a [metropolis](https://en.wikipedia.org/wiki/Metropolis) of git repos.

It's a lot to type as a name, but it's nice and unique, and if you use it a lot I suggest you create a shell alias to something shorter.

## Usage

```sh
mkdir ~/repos/
cd ~/repos/
git clone https://github.com/timabell/gitopolis.git
git clone https://github.com/timabell/gitopolis.git gitopolis-my-fork
git clone https://github.com/timabell/schema-explorer.git
git clone https://github.com/timabell/dotmatrix tims-dotmatrix

# tell gitopolis to track all the repos in the current directory
gitopolis add *

# repos in nested folders
mkdir thoughtbot
git clone https://github.com/thoughtbot/dotfiles.git thoughtbot/dotfiles
gitopolis add thoughtbot/dotfiles

# run commands in all managed repos
gitopolis exec -- git status
gitopolis exec -- du -sh .

# tagging repos
gitopolis tag tim tims-dotmatrix thoughtbot/dotfiles
gitopolis tag --remove tim tims-dotmatrix

# using tags
gitopolis clone -t tim
gitopolis exec -t tim -- git status
gitopolis list -t tim
```
It currently assumes that it can just grab the url for `origin`, we could add support for multiple origins and different names later.

### Command hierarchy

```
* gitopolis  # prints usage info
	* help / --help / -h (default) - print usage
	* add <folder(s)...>  # add one or more git repos to manage
	* remove <folder(s)...>  # remove one or more git repos from repo management
	* exec -- <command> <args...>  # execute any shell command in the repo (including git commands)
		* -t <tag>  # only exec on repos with this tag
	* list  # list all managed repos and their state
		* -t <tag>  # only list repos with this tag
	* clone  # clone any repositories that are managed but don't exist locally
		* -t <tag>  # only clone repos with this tag
	* tag  # prints subcommnd help
		* tag <tag_name> <folder...> # add tag repo(s)
		* tag --remove <tag_name> <folder...> # remove tag from repo(s)
```

Currently need `--` after exec to avoid git args being interpreted by the rust arg parser. Hope to fix that and just consume everything after exec. 

## Config format

The repo list + tags are stored in a toml file in the current folder called `.gitopolis.toml` that looks like this:

```toml
[[repos]]
path = "schema-explorer"
remotes = { origin = "https://github.com/timabell/schema-explorer.git" }
tags = ["tim"]

[[repos]]
path = "tims-dotmatrix"
remotes = { origin = "https://github.com/timabell/dotmatrix" }
tags = ["tim", "dotfiles"]
```

The double-square-bracket is the [array of tables toml markup](https://toml.io/en/v1.0.0#array-of-tables).

## Setting sync

In the manner of dotfiles, you can symlink, check-in and/or sync the config that gitopolis uses so that you can version control it and use it across multiple machines.

gitopolis creates a `.gitopolis` file in the current working directory (expected to be the parent folder of the repos). We could make this more flexible in future but it'll do for now.
