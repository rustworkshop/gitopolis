# Gitopolis

Manage multiple git repositories with ease.

* 🤓 -> Run any shell or git command on multiple `git` repositories.
* 🤓 -> Re-clone all your repos on new machines.
* 🤓 -> Limit actions to custom tags.
* 🤓 -> Easy to remember and use command list (`add`, `exec`, `clone`, `tag` etc.).
* 🤓 -> A-GPL v3 licensed labour of love ❤️.

To get an idea what a real use might look like take a look at this blog post: ["Using Gitopolis to Manage Multiple Git Repositories"](https://0x5.uk/2025/10/25/using-gitopolis-to-manage-multiple-git-repositories/)

## Installation

### Download binary release from github

1. Grab the [latest release](https://github.com/timabell/gitopolis/releases/latest),
2. unzip it
3. put the binary somewhere in your `PATH`.

I suggest adding a shorter shell alias to save typing. Perhaps `gm` for git many or `gop`.

### From crates.io

If you're a rust user, you can install from crates.io:

<https://crates.io/crates/gitopolis>

```sh
cargo install gitopolis
```

This will download the latest release crates.io, build and install it to `~/.cargo/bin/`.

### Arch linux

[Gitopolis is in the AUR](https://aur.archlinux.org/packages/gitopolis), so you can install it with `paru` or `yay` or your AUR helper of choice.

```sh
paru -S gitopolis
```

or

```sh
yay -S gitopolis
```

## Built in help

gitopolis has a fully documented command system, so use `-h` to get help for each command:

```sh
gitopolis -h
gitopolis clone -h
```

## Initial setup

There are several ways to get started:

### 1. Add your existing local repos

```sh
cd ~/repos/
gitopolis add *
```

### 2. Clone new repos

```sh
gitopolis clone https://github.com/username/repo1.git
```

### 3. Start from an existing gitopolis config

```sh
# Decide where to put the repos
mkdir ~/repos/
cd ~/repos/

# Grab a config file from somewhere (maybe a colleage) and add to the folder you'll keep the repos in
wget https://gist.githubusercontent.com/timabell/87add070a8a44db4985586efe380757d/raw/08be5b3c38190eeed4fda0060818fa39f3c67ee3/.gitopolis.toml

# Clone all the repos held in the downloaded config file to the current folder
gitopolis clone
```

### 4. Configure many repos from the github or azure-devops api

Take a look at the python scripts at [github.com/timabell/cloner](https://github.com/timabell/cloner)

This script can read repo lists from github and azure devops and write them to a gitopolis config file ready for cloning, including some sensible default tags.

## Usage

### Running shell / git commands in many repos

```sh
gitopolis exec -- git pull
gitopolis exec -- git status
```

**Note:** Commands executed with `exec` run in a non-interactive (non-TTY) environment to prevent hanging on prompts or pagers. This means:
- Git commands won't pause for pagers (like `less` for `git log`)
- Git will default to no-color output, but you can re-enable it with `--color` (e.g., `gitopolis exec -- git log --color`)
- Commands won't prompt for interactive input
- SSH/GPG keys must already be loaded and unlocked in ssh-agent or similar
- Remote SSH host keys must already be accepted (in `~/.ssh/known_hosts`)
- The easiest way to ensure keys and hosts are all setup and ready if you run into this problem is to run a single git fetch/clone outside gitopolis first. If this proves to be a regular hassle for new users then we could look at doing something about it so add your experience to [issue #236](https://github.com/timabell/gitopolis/issues/236).

#### Getting output as single lines

For compact, parsable output that's easy to sort and analyze use `--oneline`, this will put all the output on a single line for each repo (removing newlines).

e.g. to see the latest commit for all the repos, with the most recently touched repo first:

```sh
gitopolis exec --oneline -- git log --oneline -n 1
```

### Tagging

When dealing with many git repos, it can be cumbersome and slow to have to run commands on every repo every time, so you can use tags to filter down what's relevant to you in the moment, e.g. `backend`, `my-team`, `rust` or any other way of categorizing you can thing of.

```sh
gitopolis tag some_tag repo1 repo2
gitopolis exec -t some_tag -- git pull
```

#### Advanced tag filtering with AND/OR logic

You can use multiple tags with powerful AND/OR logic to precisely filter repositories:

- **Comma-separated tags** (within a single `--tag` flag) use **AND logic**: all tags must match
- **Multiple `--tag` flags** use **OR logic**: at least one tag group must match

Examples:

```sh
# Match repos with BOTH 'backend' AND 'rust'
gitopolis list --tag backend,rust

# Match repos with (backend AND rust) OR (frontend AND typescript)
gitopolis exec --tag backend,rust --tag frontend,typescript -- git pull

# Clone repos with (production AND critical) OR (staging AND critical)
gitopolis clone --tag production,critical --tag staging,critical
```

This allows for flexible repository filtering based on combinations of characteristics.

### Viewing repository information

Show the recorded information about a specific repository:

```sh
$ gitopolis show 0x5.uk

Tags:
  public
  github
  blog
  rust

Remotes:
  origin: git@github.com:timabell/0x5.uk
```

List all repositories with tags and remote URLs:

```sh
gitopolis list --long
```

List all tags and the repositories they're applied to:

```sh
gitopolis tags --long
```

### Moving repositories

Move a repository to a new location and update the configuration:

```sh
gitopolis move repo old-path new-path
```

### Managing multiple remotes

Gitopolis supports multiple git remotes per repository. Sync remotes from your git repositories into the `.gitopolis.toml` file:

```sh
gitopolis sync --read-remotes
```

Sync remotes from `.gitopolis.toml` back to your git repositories:

```sh
gitopolis sync --write-remotes
```

Note there is no automatic sync, gitopolis will never fiddle with the remotes in the managed repos or its own config unless relevant commands are invoked.

### Using complex shell commands

Gitopolis supports executing complex shell commands for each repository - including pipes, redirection, and chaining with `&&` and `||`.

To use these features, pass your entire command as a single quoted string to avoid the shell you are using processing them before they get to gitopolis:

```sh
gitopolis exec -- 'git status && git pull'
gitopolis exec -- 'git log -1 | grep "feat:"'
```

You can combine this with normal shell piping/redirection of the entire gitopolis output, e.g.:

```sh
gitopolis exec -- 'git log -1 | grep "feat:"' | wc -l
```

### State file

Gitopolis creates and manages all its state in a single simple `.gitopolis.toml` file in the working directory that you can edit, read, share with others and copy to other machines.

It is stored in [TOML](https://toml.io/) format which is a well-supported config markup with parsers for many programming languages.

Here's an example of the contents:

```toml
[[repos]]
path = "gitopolis"
tags = ["tim"]
[repos.remotes.origin]
name = "origin"
url = "git@github.com:timabell/gitopolis.git"

[[repos]]
path = "schema-explorer"
tags = ["tim", "databases"]
[repos.remotes.origin]
name = "origin"
url = "git@github.com:timabell/schema-explorer.git"

[[repos]]
path = "database-diagram-scm"
tags = ["databases"]
[repos.remotes.origin]
name = "origin"
url = "git@github.com:timabell/database-diagram-scm.git"
```
[View as gist](https://gist.github.com/timabell/87add070a8a44db4985586efe380757d).

The TOML array format takes a little getting used to, but other than that it's pretty easy to follow and edit by hand, and it allows clean round-trips of data, and is supported in just about every programming language.

## The name

Think a [metropolis](https://en.wikipedia.org/wiki/Metropolis) of git repos.

It's a lot to type as a name, but it's nice and unique, and if you use it a lot I suggest you create a shell alias to something shorter.

## Why did I create this?

* Wanted to learn more [Rust](https://www.rust-lang.org/).
* Had a client with many microservices and teams.
* Tried [gita](https://github.com/nosarthur/gita) but found command layout hard to remember, and didn't like having to install python.
* To help others with their microservices.

More recently I've been adding more features to help with other clients, and enjoying the benefits of high-quality end-to-end tests as I increasingly work with claude code on shipping features considerably more rapidly (though not always more easily lol, crazy LLMs).

## Spread the word

If you find this useful, or just think it's cool, please do help spread the word.

- Star the repo
- Tell your friends
- Share your gitopolis config with colleagues and use it to help onboard new developers to your team
- Post on social media, youtube, reddit etc about your experiences, (good or bad), and don't forget to tag me!

## Contributing

Suggestions welcome, particularly adding your experience, problems and ideas to the issues list.

I'm happy for people to open issues that are actually just questions and support queries.

Rough internal design and ambitions can be found at [Design.md](Design.md).

PRs are appreciated though it might be best to open an issue first to discuss the design.

## Alternatives

Here's the other tools I'm aware of that have more-or-less similar capabilities

* [`git for-each-repo`](https://git-scm.com/docs/git-for-each-repo) - new built-in git command for running git commands on many repos
* [gita](https://github.com/nosarthur/gita) - requires python
* [myrepos aka "mr"](https://myrepos.branchable.com/)
* [GitKraken](https://www.gitkraken.com/blog/multi-repo-management-hurdles-and-solutions) - Commercial tool
* https://stackoverflow.com/questions/816619/managing-many-git-repositories - stackoverflow question on the same
* [gr](https://mixu.net/gr/)
* [git-repo](https://gerrit.googlesource.com/git-repo/)
* [git slave](https://gitslave.sourceforge.net/)
* [mani](https://manicli.com/) a TUI (text user interface)
* [RepoZ](https://github.com/awaescher/RepoZ) - a Windows GUI tool
