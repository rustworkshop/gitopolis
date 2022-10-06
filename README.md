# Gitopolis

[![main](https://github.com/timabell/gitopolis/actions/workflows/main.yml/badge.svg)](https://github.com/timabell/gitopolis/actions/workflows/main.yml)

Manage multiple git repositories. Does the following on one or more repositories:

* Run any shell command. (Including git commands of course).
* Re-clone from config file (good for new starters, new laptops, and keeping up with ever-growing microservices).
* Run commands on subsets with tagging.

Like [gita](https://github.com/nosarthur/gita) but written in [Rust](https://www.rust-lang.org/) so you don't need python etc to run it.

## Installation

Grab the [latest release](https://github.com/timabell/gitopolis/releases/latest), unzip it and put the binary somewhere in your `PATH`.

I suggest adding a shorter shell alias to save typing. Perhaps `gm` for git many or `gop`.

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

# bonus: copes with repos in nested folders just fine
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

### Commands

Use `-h` to see available commands and arguments. E.g.:

```
$ gitopolis -h        
gitopolis, a cli tool for managing multiple git repositories - https://github.com/timabell/gitopolis

Usage: gitopolis <COMMAND>

Commands:
  add     add one or more git repos to manage
  remove  
  list    
  exec    
  tag     
  clone   Use an existing .gitopolis.toml state file to clone any/all missing repositories
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information


$ gitopolis tag -h
Usage: gitopolis tag [OPTIONS] <TAG_NAME> <REPO_FOLDERS>...

Arguments:
  <TAG_NAME>         
  <REPO_FOLDERS>...  

Options:
  -r, --remove  Remove this tag from these repo_folders
  -h, --help    Print help information


$ gitopolis list -h
Usage: gitopolis list [OPTIONS]

Options:
  -t, --tag-name <TAG_NAME>  
  -h, --help                 Print help information

```

The `--` after exec is to avoid arguments for your commands being confused with arguments to gitopolis.

## Config format

The repo list + tags are stored in a toml file in the current folder called `.gitopolis.toml` that looks like this:

```toml
[[repos]]
path = "timwise.co.uk"
tags = ["web"]
[repos.remotes.origin]
name = "origin"
url = "git@github.com:timabell/timwise.co.uk.git"

[[repos]]
path = "schema-explorer"
tags = ["golang"]
[repos.remotes.origin]
name = "origin"
url = "git@github.com:timabell/schema-explorer.git"
```

## Setting sync

In the manner of dotfiles, you can symlink, check-in and/or sync the config that gitopolis uses so that you can version control it and use it across multiple machines.

gitopolis creates the `.gitopolis.toml` file in the current working directory (expected to be the parent folder of the repos).

## The name

Think a [metropolis](https://en.wikipedia.org/wiki/Metropolis) of git repos.

It's a lot to type as a name, but it's nice and unique, and if you use it a lot I suggest you create a shell alias to something shorter.

## Social

If you like this, then twitter love would be appreciated, here's [a tweet to like/retweet/reply-to](https://twitter.com/tim_abell/status/1577421122739601408).

## Contributing

Suggestions welcome, particularly adding your experience, problems and ideas to the issues list.

I'm happy for people to open issues that are actually just questions and support queries.

Rough internal design and ambitions can be found at [Design.md](Design.md).

PRs are appreciated but bear in mind I have my own plans and this is a side project for me to learn rust in a useful way, so worth talking to me before investing too much time in anything that might not fit yet. I hope to make this smoother with better CI tests etc. Start by opening an issue with your thoughts, or ping me some other way (I'm easy to find for better or worse).
