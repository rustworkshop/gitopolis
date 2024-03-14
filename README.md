# Gitopolis

Manage multiple git repositories with ease.

* 🤓 -> Run any shell or git command on multiple `git` repositories.
* 🤓 -> Re-clone all your repos on new machines.
* 🤓 -> Limit actions to custom tags.
* 🤓 -> Easy to remember and use command list (`add` / `exec` / `clone` / `tag`).
* 🤓 -> A-GPL v3 licensed labour of love ❤️.

## Installation

1. Grab the [latest release](https://github.com/timabell/gitopolis/releases/latest),
2. unzip it
3. put the binary somewhere in your `PATH`.

I suggest adding a shorter shell alias to save typing. Perhaps `gm` for git many or `gop`.

## Usage

### Initial setup
```sh
cd ~/repos/
gitopolis add *

# (or in windows command-line):
gitopolis add */
```

### Running shell / git commands in many repos
```sh
gitopolis exec -- git pull
```

### Tagging

```sh
gitopolis tag some_tag repo1 repo2
gitopolis exec -t some_tag -- git pull
```

### Re-cloning repos on a new machine

```sh
mkdir ~/repos/ && cd ~/repos/

wget https://gist.githubusercontent.com/timabell/87add070a8a44db4985586efe380757d/raw/08be5b3c38190eeed4fda0060818fa39f3c67ee3/.gitopolis.toml

gitopolis clone
```

### Using shell aliases

You can't currently use shell aliases because commands are executed directly rather than passing through a shell such as bash/zsh/fish. See <https://github.com/rustworkshop/gitopolis/issues/13>.

As a workaround you can create git aliases that run arbitrary shell commands. See <https://github.com/rustworkshop/gitopolis/issues/80#issuecomment-1803344802>

### Using chained commands

You can't chain commands together with `&&` or `||` like this `gitopolis exec -- git branch && git pull` because the shell (bash etc) will parse the `&&` before it ever gets to gitopolis.

As a workaround you can create git aliases that run arbitrary shell commands. See <https://github.com/rustworkshop/gitopolis/issues/80#issuecomment-1803344802>

### State file

Creates a single simple `.gitopolis.toml` file that you can edit, read, share with others and copy to other machines.

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

[TOML](https://toml.io/) is a well-supported config markup with parsers for many programming languages.

---

---

## The name

Think a [metropolis](https://en.wikipedia.org/wiki/Metropolis) of git repos.

It's a lot to type as a name, but it's nice and unique, and if you use it a lot I suggest you create a shell alias to something shorter.

## Why did I create this?

* Wanted to learn more [Rust](https://www.rust-lang.org/).
* Had a client with many microservices and teams.
* Tried [gita](https://github.com/nosarthur/gita) but found command layout hard to remember, and didn't like having to install python.
* To help others with their microservices.

## Social

Help others discover gitopolis:

### Twitter

[Tweet about gitopolis](http://twitter.com/share?text=Gitopolis%20is%20fab,%20check%20it%20out%20-%20made%20by%20@tim_abell&url=https://github.com/timabell/gitopolis)

### Product Hunt

Vote for gitopolis:

<a href="https://www.producthunt.com/posts/gitopolis?utm_source=badge-featured&utm_medium=badge&utm_souce=badge-gitopolis" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=392942&theme=light" alt="Gitopolis - Manage&#0032;multiple&#0032;git&#0032;repositories&#0032;with&#0032;ease&#0046; | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54" /></a>

## Contributing

Suggestions welcome, particularly adding your experience, problems and ideas to the issues list.

I'm happy for people to open issues that are actually just questions and support queries.

Rough internal design and ambitions can be found at [Design.md](Design.md).

PRs are appreciated but bear in mind I have my own plans and this is a side project for me to learn rust in a useful way, so worth talking to me before investing too much time in anything that might not fit yet. I hope to make this smoother with better CI tests etc. Start by opening an issue with your thoughts, or ping me some other way (I'm easy to find for better or worse).

### Builds

* [![build-main](https://github.com/timabell/gitopolis/actions/workflows/build-main.yml/badge.svg)](https://github.com/timabell/gitopolis/actions/workflows/build-main.yml) - Continuous integration build.
* [![build-tag](https://github.com/timabell/gitopolis/actions/workflows/build-tag.yml/badge.svg)](https://github.com/timabell/gitopolis/actions/workflows/build-tag.yml) - Release build - generates binaries for download from tagged builds.
