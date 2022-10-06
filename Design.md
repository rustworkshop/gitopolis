# Design

Mostly for me to think about how it should be.

## Code design

* [main](src/main.rs) - basically a bridge between a console and the actual logic.
  * Defines command-line interface (i.e. subcommands, arguments etc).
  * Injects real dependencies into gitopolis module
* [lib](src/lib.rs) - top of re-usable gitopolis module, pulls in all modules it needs (everything except main basically)
* [gitopolis](src/gitopolis.rs) - all the logic of this tool.
  * injected dependencies: 
    * state [storage](src/storage.rs) trait
    * [git operations](src/git.rs) trait
    * stdout - not injected yet but, this needs to change to return programatically useful responses instead of piles of strings in byte form
* [exec](src/exec.rs) - run arbitrary commands in list of paths/repos
  * currently a separate thing managed by main, needs to be controlled by gitopolis.rs instead
  * writes to stdout, not streamed, also needs to change
* [repos](src/repos.rs) - models for encapsulating state of repo(s) with methods for changing state
  * needs a bit of tlc, currently exposes its `Vec<Repo>` internals, but otherwise seems sound
* [list](src/list.rs) - probably needs to go away, just writes repo list or message to stdout. Listing functionality needs expanding.

## Testing

* [test.sh](test.sh) - hacky shell based test, will go away when have a better end-to-end rust test.
* End-to-end, uses all the code for real, what's mocked and simulated tbc but should be testing maximum breadth of code. This is the one to give confidence that a PR hasn't completely broken it in a way the more granular tests don't catch, e.g. someone breaking the arg parser.
* [gitopolis_tests](tests/gitopolis_tests.rs) - tests just the main logic in the library, with mocked storage, git (& stdout?). Faster and more granular than end-to-end.


## Output

Currently using logging (`info!` macro etc) with custom formatting. Not sure this is wise.


