use std::fs;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::{predicate, PredicateBooleanExt};
use tempfile::{tempdir, TempDir};

#[test]
fn help() {
	gitopolis_executable()
		.arg("help")
		.assert()
		.success()
		.stdout(predicate::str::contains("Usage: gitopolis"));
}

// only windows (cmd/powerhell) needs to have globs expanded for it, real OS's do it for you in the shell
#[cfg(target_os = "windows")]
#[test]
fn add_glob() {
	// Linux has shell globbing built in, but that's not available for windows/cmd so "add *" is passed
	// in without being expanded, resulting in an error instead of adding everything.
	// https://github.com/rustworkshop/gitopolis/issues/122
	let temp = temp_folder();
	create_git_repo(&temp, "first_git_folder", "git://example.org/test_url");
	create_git_repo(&temp, "second_git_folder", "git://example.org/test_url2");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "*"])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added second_git_folder"));

	let expected_toml = "[[repos]]
path = \"first_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"

[[repos]]
path = \"second_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url2\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn add() {
	let temp = temp_folder();
	create_git_repo(&temp, "some_git_folder", "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "some_git_folder"])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added some_git_folder\n"));

	create_git_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "some_other_git_folder"])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added some_other_git_folder\n"));

	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"

[[repos]]
path = \"some_other_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url2\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn remove() {
	let temp = temp_folder();
	add_a_repo(&temp, "some_git_folder", "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["remove", "some_git_folder"])
		.assert()
		.success();

	assert_eq!("repos = []\n", read_gitopolis_state_toml(&temp));
}

#[test]
fn list_errors_when_no_config() {
	gitopolis_executable()
		.arg("list")
		.assert()
		.failure()
		.code(2)
		.stdout(predicate::str::contains("No repos"));
}

#[test]
fn list() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout("some_git_folder\nsome_other_git_folder\n");
}

#[test]
fn list_tag() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list", "--tag", "some_tag"])
		.assert()
		.success()
		.stdout("some_git_folder\n");
}

#[test]
fn list_tag_abbreviated() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list", "-t", "some_tag"])
		.assert()
		.success()
		.stdout("some_git_folder\n");
}

#[test]
fn list_long() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	let expected_long_output =
		"some_git_folder\tsome_tag,another_tag\torigin=git://example.org/test_url
some_other_git_folder\t\torigin=git://example.org/test_url2
";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list", "-l"])
		.assert()
		.success()
		.stdout(expected_long_output);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list", "--long"])
		.assert()
		.success()
		.stdout(expected_long_output);
}

#[test]
fn exec() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	let expected_stdout = "
🏢 some_git_folder> git config remote.origin.url
git://example.org/test_url


🏢 some_other_git_folder> git config remote.origin.url
git://example.org/test_url2

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "git", "config", "remote.origin.url"])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn exec_missing() {
	let temp = temp_folder();

	let initial_state_toml = "[[repos]]
path = \"missing_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"example_url\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	let expected_stdout = "
🏢 missing_git_folder> Repo folder missing, skipped.
";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "never_called"])
		.assert()
		.failure()
		.code(1)
		.stdout(expected_stdout)
		.stderr("1 repos skipped\n");
}

#[test]
fn exec_tag() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	let expected_stdout = "
🏢 some_git_folder> git config remote.origin.url
git://example.org/test_url

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"--tag",
			"some_tag",
			"--",
			"git",
			"config",
			"remote.origin.url",
		])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn exec_tag_abbreviated() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag", "another_tag"],
	);
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	let expected_stdout = "
🏢 some_git_folder> git config remote.origin.url
git://example.org/test_url

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"-t",
			"some_tag",
			"--",
			"git",
			"config",
			"remote.origin.url",
		])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn exec_non_zero() {
	let temp = temp_folder();
	add_a_repo(&temp, "some_git_folder", "git://example.org/test_url");
	add_a_repo(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
	);

	let expected_stdout = "
🏢 some_git_folder> ls non-existent


🏢 some_other_git_folder> ls non-existent

";
	let expected_stderr = match get_operating_system() {
		OperatingSystem::MacOSX => {
			"ls: non-existent: No such file or directory
Command exited with code 1
ls: non-existent: No such file or directory
Command exited with code 1
2 commands exited with non-zero status code
"
		}
		OperatingSystem::Other => {
			"ls: cannot access \'non-existent\': No such file or directory
Command exited with code 2
ls: cannot access \'non-existent\': No such file or directory
Command exited with code 2
2 commands exited with non-zero status code
"
		}
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "ls", "non-existent"])
		.assert()
		.failure()
		.code(1)
		.stdout(expected_stdout)
		.stderr(expected_stderr);
}

#[test]
fn exec_invalid_command() {
	let temp = temp_folder();
	add_a_repo(&temp, "some_git_folder", "git://example.org/test_url");

	// With shell execution, invalid commands are handled by the shell
	// Gitopolis should exit with failure when shell commands fail
	let expected_error = if cfg!(windows) {
		"not recognized as an internal or external command"
	} else {
		"not found"
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "not-a-command"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains("not-a-command"))
		.stderr(predicate::str::contains(expected_error))
		.stderr(predicate::str::contains(
			"1 commands exited with non-zero status code",
		));
}

#[test]
fn exec_oneline() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_url");
	add_a_repo(&temp, "repo_b", "git://example.org/test_url2");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "echo", "hello"])
		.assert()
		.success()
		.stdout("repo_a\thello\nrepo_b\thello\n");
}

#[test]
fn exec_oneline_multiline_output() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_url");

	// Create a test file with multiple lines in the repo
	let repo_path = temp.path().join("repo_a");
	fs::write(repo_path.join("test.txt"), "line1\nline2\nline3").unwrap();

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "cat", "test.txt"])
		.assert()
		.success()
		.stdout("repo_a\tline1 line2 line3\n");
}

#[test]
fn exec_oneline_non_zero() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_url");
	add_a_repo(&temp, "repo_b", "git://example.org/test_url2");

	let expected_stdout = match get_operating_system() {
		OperatingSystem::MacOSX => {
			"repo_a\tls: non-existent: No such file or directory\nrepo_b\tls: non-existent: No such file or directory\n"
		}
		OperatingSystem::Other => {
			"repo_a\tls: cannot access 'non-existent': No such file or directory\nrepo_b\tls: cannot access 'non-existent': No such file or directory\n"
		}
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "ls", "non-existent"])
		.assert()
		.failure()
		.code(1)
		.stdout(expected_stdout)
		.stderr("2 commands exited with non-zero status code\n");
}

#[test]
fn exec_oneline_missing() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_url");

	let initial_state_toml = "[[repos]]
path = \"missing_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"example_url\"

[[repos]]
path = \"repo_a\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "echo", "hello"])
		.assert()
		.failure()
		.code(1)
		.stdout("missing_repo\tRepo folder missing, skipped.\nrepo_a\thello\n")
		.stderr("1 repos skipped\n");
}

#[test]
fn tag() {
	let temp = temp_folder();
	add_a_repo(&temp, "some_git_folder", "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "some_tag", "some_git_folder"])
		.assert()
		.success();

	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = [\"some_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn tag_remove() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag"],
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "--remove", "some_tag", "some_git_folder"])
		.assert()
		.success();

	let actual_toml = read_gitopolis_state_toml(&temp);
	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, actual_toml);
}

#[test]
fn tag_remove_abbreviated() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag"],
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "-r", "some_tag", "some_git_folder"])
		.assert()
		.success();

	let actual_toml = read_gitopolis_state_toml(&temp);
	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, actual_toml);
}

#[test]
fn tags() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag"],
	);
	add_a_repo_with_tags(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
		vec!["some_tag", "another_tag"],
	);

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tags"])
		.assert()
		.success()
		.stdout("another_tag\nsome_tag\n");
}

#[test]
fn tags_long() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag"],
	);
	add_a_repo_with_tags(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
		vec!["some_tag", "another_tag"],
	);

	let expected_stdout = "another_tag
	some_other_git_folder

some_tag
	some_git_folder
	some_other_git_folder

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tags", "--long"])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn tags_long_abbreviated() {
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"some_git_folder",
		"git://example.org/test_url",
		vec!["some_tag"],
	);
	add_a_repo_with_tags(
		&temp,
		"some_other_git_folder",
		"git://example.org/test_url2",
		vec!["some_tag", "another_tag"],
	);

	let expected_stdout = "another_tag
	some_other_git_folder

some_tag
	some_git_folder
	some_other_git_folder

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tags", "-l"])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn clone() {
	let temp = temp_folder();
	create_local_repo(&temp, "source_repo");
	let initial_state_toml = "[[repos]]
path = \"some_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"source_repo\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	let expected_clone_stdout = "🏢 some_git_folder> Cloning source_repo ...

Cloning into \'some_git_folder\'...
warning: You appear to have cloned an empty repository.
done.

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone"])
		.assert()
		.success()
		.stdout(expected_clone_stdout);

	// check repo has been successfully cloned by running a git command on it via exec
	let expected_exec_stdout = match get_operating_system() {
		OperatingSystem::MacOSX => {
			"
🏢 some_git_folder> git status
On branch main

No commits yet

nothing to commit

"
		}
		OperatingSystem::Other => {
			"
🏢 some_git_folder> git status
On branch main

No commits yet

nothing to commit (create/copy files and use \"git add\" to track)

"
		}
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "git", "status"])
		.assert()
		.success()
		.stdout(expected_exec_stdout);
}

#[test]
fn clone_tag() {
	let temp = temp_folder();
	create_local_repo(&temp, "source_repo");
	let initial_state_toml = "[[repos]]
path = \"some_git_folder\"
tags = [\"some_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"source_repo\"

[[repos]]
path = \"some_other_git_folder\"
tags = [\"some_other_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"nonexistent_source_repo\"

[[repos]]
path = \"yet_other_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"nonexistent_source_repo\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	let expected_clone_stdout = "🏢 some_git_folder> Cloning source_repo ...

Cloning into \'some_git_folder\'...
warning: You appear to have cloned an empty repository.
done.

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone", "--tag", "some_tag"])
		.assert()
		.success()
		.stdout(expected_clone_stdout);

	// check repo has been successfully cloned by running a git command on it via exec
	let expected_exec_stdout = match get_operating_system() {
		OperatingSystem::MacOSX => {
			"
🏢 some_git_folder> git status
On branch main

No commits yet

nothing to commit

"
		}
		OperatingSystem::Other => {
			"
🏢 some_git_folder> git status
On branch main

No commits yet

nothing to commit (create/copy files and use \"git add\" to track)

"
		}
	};
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--tag", "some_tag", "--", "git", "status"]) // filter exec to tag otherwise it runs on repos that don't yet exists https://github.com/timabell/gitopolis/issues/29
		.assert()
		.success()
		.stdout(expected_exec_stdout);
}

fn create_local_repo(temp: &TempDir, repo_name: &str) {
	create_git_repo(temp, repo_name, "git://example.org/test_url");
}

fn tag_repo(temp: &TempDir, repo_name: &str, tag_name: &str) {
	gitopolis_executable()
		.current_dir(temp)
		.args(vec!["tag", tag_name, repo_name])
		.output()
		.expect("Failed to tag repo");
}

fn add_a_repo_with_tags(temp: &TempDir, repo_name: &str, remote_url: &str, tags: Vec<&str>) {
	add_a_repo(temp, repo_name, remote_url);

	tags.into_iter().for_each(|tag| {
		tag_repo(temp, repo_name, tag);
	});
}

fn add_a_repo(temp: &TempDir, repo_name: &str, remote_url: &str) {
	create_git_repo(temp, repo_name, remote_url);

	gitopolis_executable()
		.current_dir(temp)
		.args(vec!["add", repo_name])
		.output()
		.expect("Failed to add repo");
}

fn create_git_repo(temp: &TempDir, repo_name: &str, remote_url: &str) {
	let path = &temp.path().join(repo_name);
	fs::create_dir_all(path).expect("create repo dir failed");

	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git command failed");

	Command::new("git")
		.current_dir(path)
		.args(vec!["config", "remote.origin.url", remote_url])
		.output()
		.expect("git command failed");
}

fn gitopolis_executable() -> AssertCommand {
	AssertCommand::cargo_bin("gitopolis").expect("failed to find binary")
}

fn write_gitopolis_state_toml(temp: &TempDir, initial_state_toml: &str) {
	fs::write(temp.path().join(".gitopolis.toml"), initial_state_toml)
		.expect("failed to write initial state toml");
}
fn read_gitopolis_state_toml(temp: &TempDir) -> String {
	fs::read_to_string(temp.path().join(".gitopolis.toml")).expect("failed to read back toml")
}
fn temp_folder() -> TempDir {
	tempdir().expect("get tmp dir failed")
}

#[allow(dead_code)] // each value only used on one OS so get dead code warning on others
enum OperatingSystem {
	MacOSX,
	Other,
}

#[cfg(target_os = "macos")]
fn get_operating_system() -> OperatingSystem {
	OperatingSystem::MacOSX
}

#[cfg(not(target_os = "macos"))]
fn get_operating_system() -> OperatingSystem {
	OperatingSystem::Other
}

#[test]
fn exec_command_oneline_with_piping() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "echo hello | sort"]) // Use echo piped to sort which works on both platforms
		.assert()
		.success()
		.stdout(predicate::str::contains("repo_a\thello"))
		.stdout(predicate::str::contains("repo_b\thello"));
}

#[test]
fn exec_shell_piping() {
	// Test that shell pipes work within each repository (non-oneline)
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "echo test output | sort"])
		.assert()
		.success()
		.stdout(predicate::str::contains(
			"🏢 repo_a> 'echo test output | sort'",
		))
		.stdout(predicate::str::contains("test output"))
		.stdout(predicate::str::contains(
			"🏢 repo_b> 'echo test output | sort'",
		));
}

#[test]
fn exec_shell_quoted_args() {
	// Test that quoted arguments work properly with shell execution
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "echo 'hello world'"])
		.assert()
		.success()
		.stdout(predicate::str::contains("hello world"));
}

#[test]
fn add_multiple_remotes() {
	let temp = temp_folder();
	let path = &temp.path().join("test_repo");
	fs::create_dir_all(path).expect("create repo dir failed");

	// Initialize git repo
	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	// Add multiple remotes
	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/origin_url",
		])
		.output()
		.expect("git remote add origin failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"upstream",
			"git://example.org/upstream_url",
		])
		.output()
		.expect("git remote add upstream failed");

	// Run gitopolis add
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "test_repo"])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added test_repo\n"));

	// Verify TOML contains both remotes
	let expected_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/origin_url\"

[repos.remotes.upstream]
name = \"upstream\"
url = \"git://example.org/upstream_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn list_long_multiple_remotes() {
	let temp = temp_folder();
	let path = &temp.path().join("test_repo");
	fs::create_dir_all(path).expect("create repo dir failed");

	// Initialize git repo
	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	// Add multiple remotes
	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/origin_url",
		])
		.output()
		.expect("git remote add origin failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"upstream",
			"git://example.org/upstream_url",
		])
		.output()
		.expect("git remote add upstream failed");

	// Run gitopolis add
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "test_repo"])
		.output()
		.expect("Failed to add repo");

	// Test list --long shows both remotes
	let expected_output = "test_repo\t\torigin=git://example.org/origin_url,upstream=git://example.org/upstream_url\n";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list", "--long"])
		.assert()
		.success()
		.stdout(expected_output);
}

#[test]
fn clone_multiple_remotes() {
	let temp = temp_folder();

	// Create source repos
	create_local_repo(&temp, "source_origin");
	create_local_repo(&temp, "source_upstream");

	// Create state with multiple remotes
	let initial_state_toml = "[[repos]]
path = \"cloned_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"source_origin\"

[repos.remotes.upstream]
name = \"upstream\"
url = \"source_upstream\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run clone
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone"])
		.assert()
		.success();

	// Verify both remotes exist in cloned repo
	let cloned_path = temp.path().join("cloned_repo");

	let origin_output = Command::new("git")
		.current_dir(&cloned_path)
		.args(vec!["config", "remote.origin.url"])
		.output()
		.expect("git config failed");
	assert!(origin_output.status.success());
	let origin_url = String::from_utf8(origin_output.stdout)
		.expect("utf8 conversion failed")
		.trim()
		.to_string();
	assert!(origin_url.ends_with("source_origin"));

	let upstream_output = Command::new("git")
		.current_dir(&cloned_path)
		.args(vec!["config", "remote.upstream.url"])
		.output()
		.expect("git config failed");
	assert!(upstream_output.status.success());
	let upstream_url = String::from_utf8(upstream_output.stdout)
		.expect("utf8 conversion failed")
		.trim()
		.to_string();
	assert!(upstream_url.ends_with("source_upstream"));
}

#[test]
fn sync_read_remotes() {
	let temp = temp_folder();
	let path = &temp.path().join("test_repo");
	fs::create_dir_all(path).expect("create repo dir failed");

	// Initialize git repo with multiple remotes
	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/origin_url",
		])
		.output()
		.expect("git remote add origin failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"upstream",
			"git://example.org/upstream_url",
		])
		.output()
		.expect("git remote add upstream failed");

	// Create initial gitopolis state with just the repo path (no remotes)
	let initial_state_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes]
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --read-remotes
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--read-remotes"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Updated test_repo with remotes from git",
		));

	// Verify TOML now contains both remotes from git
	let expected_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/origin_url\"

[repos.remotes.upstream]
name = \"upstream\"
url = \"git://example.org/upstream_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn sync_write_remotes() {
	let temp = temp_folder();
	let path = &temp.path().join("test_repo");
	fs::create_dir_all(path).expect("create repo dir failed");

	// Initialize git repo with just origin
	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/origin_url",
		])
		.output()
		.expect("git remote add origin failed");

	// Create gitopolis state with additional remotes
	let initial_state_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/origin_url\"

[repos.remotes.upstream]
name = \"upstream\"
url = \"git://example.org/upstream_url\"

[repos.remotes.fork]
name = \"fork\"
url = \"git://example.org/fork_url\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --write-remotes
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--write-remotes"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Added remote upstream to test_repo",
		))
		.stderr(predicate::str::contains("Added remote fork to test_repo"));

	// Verify git now has all remotes
	let upstream_output = Command::new("git")
		.current_dir(path)
		.args(vec!["config", "remote.upstream.url"])
		.output()
		.expect("git config failed");
	assert!(upstream_output.status.success());
	let upstream_url = String::from_utf8(upstream_output.stdout)
		.expect("utf8 conversion failed")
		.trim()
		.to_string();
	assert_eq!("git://example.org/upstream_url", upstream_url);

	let fork_output = Command::new("git")
		.current_dir(path)
		.args(vec!["config", "remote.fork.url"])
		.output()
		.expect("git config failed");
	assert!(fork_output.status.success());
	let fork_url = String::from_utf8(fork_output.stdout)
		.expect("utf8 conversion failed")
		.trim()
		.to_string();
	assert_eq!("git://example.org/fork_url", fork_url);
}

#[test]
fn sync_read_remotes_with_tag() {
	let temp = temp_folder();

	// Create first repo with tag
	let path1 = &temp.path().join("tagged_repo");
	fs::create_dir_all(path1).expect("create repo dir failed");
	Command::new("git")
		.current_dir(path1)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");
	Command::new("git")
		.current_dir(path1)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/tagged_url",
		])
		.output()
		.expect("git remote add failed");

	// Create second repo without tag
	let path2 = &temp.path().join("untagged_repo");
	fs::create_dir_all(path2).expect("create repo dir failed");
	Command::new("git")
		.current_dir(path2)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");
	Command::new("git")
		.current_dir(path2)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/untagged_url",
		])
		.output()
		.expect("git remote add failed");

	// Create initial state with tags
	let initial_state_toml = "[[repos]]
path = \"tagged_repo\"
tags = [\"sync-tag\"]

[repos.remotes]

[[repos]]
path = \"untagged_repo\"
tags = []

[repos.remotes]
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --read-remotes with tag
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--read-remotes", "--tag", "sync-tag"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Updated tagged_repo with remotes from git",
		))
		.stderr(predicate::str::contains("untagged_repo").not());

	// Verify only tagged repo was updated
	let toml = read_gitopolis_state_toml(&temp);
	assert!(toml.contains("git://example.org/tagged_url"));
	assert!(!toml.contains("git://example.org/untagged_url"));
}

#[test]
fn sync_write_remotes_with_tag() {
	let temp = temp_folder();

	// Create first repo with tag
	let path1 = &temp.path().join("tagged_repo");
	fs::create_dir_all(path1).expect("create repo dir failed");
	Command::new("git")
		.current_dir(path1)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	// Create second repo without tag
	let path2 = &temp.path().join("untagged_repo");
	fs::create_dir_all(path2).expect("create repo dir failed");
	Command::new("git")
		.current_dir(path2)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	// Create state with remotes to add
	let initial_state_toml = "[[repos]]
path = \"tagged_repo\"
tags = [\"sync-tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/tagged_origin\"

[repos.remotes.upstream]
name = \"upstream\"
url = \"git://example.org/tagged_upstream\"

[[repos]]
path = \"untagged_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/untagged_origin\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --write-remotes with tag
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--write-remotes", "--tag", "sync-tag"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Added remote origin to tagged_repo",
		))
		.stderr(predicate::str::contains(
			"Added remote upstream to tagged_repo",
		))
		.stderr(predicate::str::contains("untagged_repo").not());

	// Verify only tagged repo got remotes
	let output1 = Command::new("git")
		.current_dir(path1)
		.args(vec!["remote", "-v"])
		.output()
		.expect("git remote failed");
	let remotes1 = String::from_utf8(output1.stdout).expect("utf8 conversion failed");
	assert!(remotes1.contains("origin"));
	assert!(remotes1.contains("upstream"));

	let output2 = Command::new("git")
		.current_dir(path2)
		.args(vec!["remote", "-v"])
		.output()
		.expect("git remote failed");
	let remotes2 = String::from_utf8(output2.stdout).expect("utf8 conversion failed");
	assert!(remotes2.is_empty());
}

#[test]
fn sync_read_remotes_missing_repo() {
	let temp = temp_folder();

	// Create initial state with a missing repo
	let initial_state_toml = "[[repos]]
path = \"missing_repo\"
tags = []

[repos.remotes]
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --read-remotes
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--read-remotes"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Warning: Could not read remotes from missing_repo",
		))
		.stderr(predicate::str::contains("1 repos failed to sync"));
}

#[test]
fn sync_write_remotes_missing_repo() {
	let temp = temp_folder();

	// Create initial state with a missing repo
	let initial_state_toml = "[[repos]]
path = \"missing_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	write_gitopolis_state_toml(&temp, initial_state_toml);

	// Run sync --write-remotes
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["sync", "--write-remotes"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Warning: Could not write remotes to missing_repo",
		))
		.stderr(predicate::str::contains("1 repos failed to sync"));
}

#[test]
fn show() {
	let temp = temp_folder();
	let path = &temp.path().join("test_repo");
	fs::create_dir_all(path).expect("create repo dir failed");

	// Initialize git repo
	Command::new("git")
		.current_dir(path)
		.args(vec!["init", "--initial-branch", "main"])
		.output()
		.expect("git init failed");

	// Add multiple remotes
	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"origin",
			"git://example.org/origin_url",
		])
		.output()
		.expect("git remote add origin failed");

	Command::new("git")
		.current_dir(path)
		.args(vec![
			"remote",
			"add",
			"upstream",
			"git://example.org/upstream_url",
		])
		.output()
		.expect("git remote add upstream failed");

	// Add repo to gitopolis and tag it
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["add", "test_repo"])
		.output()
		.expect("Failed to add repo");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "backend", "test_repo"])
		.output()
		.expect("Failed to tag repo");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "rust", "test_repo"])
		.output()
		.expect("Failed to tag repo");

	// Run show command
	let expected_output = "Tags:
  backend
  rust

Remotes:
  origin: git://example.org/origin_url
  upstream: git://example.org/upstream_url
";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["show", "test_repo"])
		.assert()
		.success()
		.stdout(expected_output);
}

#[test]
fn show_repo_not_found() {
	let temp = temp_folder();

	// Try to show a repo that doesn't exist
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["show", "nonexistent_repo"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Repo 'nonexistent_repo' not found",
		));
}

#[test]
fn tag_repo_not_found() {
	let temp = temp_folder();
	add_a_repo(&temp, "existing_repo", "git://example.org/test_url");

	// Try to tag a repo that doesn't exist
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "some_tag", "nonexistent_repo"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Repo 'nonexistent_repo' not found",
		));
}

#[test]
fn tag_remove_repo_not_found() {
	let temp = temp_folder();
	add_a_repo(&temp, "existing_repo", "git://example.org/test_url");

	// Try to remove tag from a repo that doesn't exist
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "--remove", "some_tag", "nonexistent_repo"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Repo 'nonexistent_repo' not found",
		));
}

#[test]
fn exec_displays_quoted_args() {
	// Test that exec command display adds quotes for arguments with spaces
	// Issue: https://github.com/rustworkshop/gitopolis/issues/86
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "echo", "oh no"])
		.assert()
		.success()
		.stdout(predicate::str::contains("🏢 repo_a> echo 'oh no'"));
}

#[test]
fn clone_with_url() {
	// Test cloning a repository from a URL and automatically adding it to gitopolis
	// Issue: https://github.com/rustworkshop/gitopolis/issues/193
	let temp = temp_folder();

	// Create a source repo in a subdirectory
	let sources_dir = temp.path().join("_sources");
	fs::create_dir(&sources_dir).unwrap();
	let source_temp = TempDir::new_in(&sources_dir).unwrap();
	create_local_repo(&source_temp, "myrepo");
	let source_path = source_temp
		.path()
		.join("myrepo")
		.to_str()
		.unwrap()
		.to_string();

	// Run clone with URL - clone into temp directory
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone", &source_path])
		.assert()
		.success()
		.stdout(predicate::str::contains("🏢 myrepo> Cloning"))
		.stdout(predicate::str::contains("Cloning into 'myrepo'"));

	// Verify repo was added to .gitopolis.toml
	let toml = read_gitopolis_state_toml(&temp);
	assert!(toml.contains("path = \"myrepo\""));

	// Verify repo is listed
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("myrepo"));
}

#[test]
fn clone_with_url_extracts_folder_name() {
	// Test that clone extracts the correct folder name from various URL formats
	// Issue: https://github.com/rustworkshop/gitopolis/issues/193
	let temp = temp_folder();

	// Create a source repo with .git in the name to test extraction
	let sources_dir = temp.path().join("_sources");
	fs::create_dir(&sources_dir).unwrap();
	let source_temp = TempDir::new_in(&sources_dir).unwrap();
	// Create repo named "myrepo.git" to simulate typical git URL structure
	create_local_repo(&source_temp, "myrepo.git");
	let source_path = source_temp
		.path()
		.join("myrepo.git")
		.to_str()
		.unwrap()
		.to_string();

	// Clone - should extract "myrepo" without the .git extension
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone", &source_path])
		.assert()
		.success();

	// Verify folder name is without .git
	let toml = read_gitopolis_state_toml(&temp);
	assert!(toml.contains("path = \"myrepo\""));

	// Verify the cloned repo exists
	assert!(temp.path().join("myrepo").exists());
}

#[test]
fn clone_with_url_and_target_dir() {
	// Test cloning with a custom target directory, like git clone does
	// Issue: https://github.com/rustworkshop/gitopolis/issues/193
	let temp = temp_folder();

	// Create a source repo in a subdirectory
	let sources_dir = temp.path().join("_sources");
	fs::create_dir(&sources_dir).unwrap();
	let source_temp = TempDir::new_in(&sources_dir).unwrap();
	create_local_repo(&source_temp, "upstream_repo");
	let source_path = source_temp
		.path()
		.join("upstream_repo")
		.to_str()
		.unwrap()
		.to_string();

	// Clone with custom target directory
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["clone", &source_path, "my_custom_name"])
		.assert()
		.success()
		.stdout(predicate::str::contains("🏢 my_custom_name> Cloning"));

	// Verify repo was added with the custom name
	let toml = read_gitopolis_state_toml(&temp);
	assert!(toml.contains("path = \"my_custom_name\""));

	// Verify the cloned repo exists with the custom name
	assert!(temp.path().join("my_custom_name").exists());

	// Verify repo is listed with custom name
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout("my_custom_name\n");
}

#[test]
fn move_repo() {
	// Test basic move operation
	// Issue: https://github.com/rustworkshop/gitopolis/issues/157
	let temp = temp_folder();
	add_a_repo(&temp, "old_location", "git://example.org/test_url");

	// Move the repo
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["move", "repo", "old_location", "new_location"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Moved old_location to new_location",
		));

	// Verify old location doesn't exist
	assert!(!temp.path().join("old_location").exists());

	// Verify new location exists
	assert!(temp.path().join("new_location").exists());

	// Verify config is updated
	let expected_toml = "[[repos]]
path = \"new_location\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));

	// Verify repo is listed with new name
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout("new_location\n");
}

#[test]
fn move_repo_with_nested_path() {
	// Test move operation with nested source and target paths, creating parent directories
	// Issue: https://github.com/rustworkshop/gitopolis/issues/157
	let temp = temp_folder();
	add_a_repo(&temp, "services/backend", "git://example.org/test_url");

	// Move to nested path (parent directories don't exist yet)
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["move", "repo", "services/backend", "apps/auth"])
		.assert()
		.success()
		.stderr(predicate::str::contains(
			"Moved services/backend to apps/auth",
		));

	// Verify old location doesn't exist
	assert!(!temp.path().join("services/backend").exists());

	// Verify new location exists
	assert!(temp.path().join("apps/auth").exists());

	// Verify new parent directory was created
	assert!(temp.path().join("apps").exists());

	// Verify config is updated
	let expected_toml = "[[repos]]
path = \"apps/auth\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn move_repo_preserves_tags() {
	// Test that move preserves tags
	// Issue: https://github.com/rustworkshop/gitopolis/issues/157
	let temp = temp_folder();
	add_a_repo_with_tags(
		&temp,
		"tagged_repo",
		"git://example.org/test_url",
		vec!["backend", "rust"],
	);

	// Move the repo
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["move", "repo", "tagged_repo", "new_tagged_repo"])
		.assert()
		.success();

	// Verify tags are preserved
	let expected_toml = "[[repos]]
path = \"new_tagged_repo\"
tags = [\"backend\", \"rust\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn move_repo_not_found() {
	// Test that move fails when repo doesn't exist
	// Issue: https://github.com/rustworkshop/gitopolis/issues/157
	let temp = temp_folder();
	add_a_repo(&temp, "existing_repo", "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["move", "repo", "nonexistent_repo", "new_location"])
		.assert()
		.failure()
		.code(1)
		.stderr(predicate::str::contains(
			"Repo 'nonexistent_repo' not found",
		));
}

#[test]
fn exec_with_special_chars() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	// Test with all special characters that need quoting: whitespace, quotes, and shell metacharacters
	// From needs_quoting(): | & ; < > ( ) $ ` \ " ' * ? [ ] { } ! #
	// Windows cmd.exe echo outputs escaped quotes for the outer quotes
	let expected_stdout = if cfg!(windows) {
		r#"
🏢 repo_a> echo "test \" ' | & ; < > ( ) $ ` \\ * ? [ ] { } ! # chars"
\"test \"\" ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars\"

"#
	} else {
		r#"
🏢 repo_a> echo "test \" ' | & ; < > ( ) $ ` \\ * ? [ ] { } ! # chars"
test " ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars

"#
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"--",
			"echo",
			r#"test " ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars"#,
		])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn exec_single_arg_with_quotes_and_pipes() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", r#"echo "foo | bar" | grep bar"#])
		.assert()
		.success()
		.stdout(predicate::str::contains("foo | bar"));
}

#[test]
fn exec_single_arg_literal_pipe_characters() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", r#"echo "this | is | not | a | pipe""#])
		.assert()
		.success()
		.stdout(predicate::str::contains("this | is | not | a | pipe"));
}

#[test]
fn exec_multiple_args_with_single_quotes() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	// Windows cmd.exe echo behaves differently - it prints the quotes we add
	// Unix shells strip quotes before passing to echo
	let expected_output = if cfg!(windows) {
		"\"argument with 'quotes'\""
	} else {
		"argument with 'quotes'"
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "echo", "argument with 'quotes'"])
		.assert()
		.success()
		.stdout(predicate::str::contains(expected_output));
}

#[test]
fn exec_oneline_with_special_chars() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	// Test with all special characters: | & ; < > ( ) $ ` \ " ' * ? [ ] { } ! #
	// Windows cmd.exe echo behaves differently - it escapes the outer quotes with backslashes
	let expected_stdout = if cfg!(windows) {
		r#"repo_a	\"test \"\" ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars\"
repo_b	\"test \"\" ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars\"
"#
	} else {
		r#"repo_a	test " ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars
repo_b	test " ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars
"#
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"--oneline",
			"--",
			"echo",
			r#"test " ' | & ; < > ( ) $ ` \ * ? [ ] { } ! # chars"#,
		])
		.assert()
		.success()
		.stdout(expected_stdout);
}

#[test]
fn exec_oneline_single_arg_with_pipes() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"--oneline",
			"--",
			r#"echo "hey there" | wc -w"#,
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("repo_a\t2"))
		.stdout(predicate::str::contains("repo_b\t2"));
}

#[test]
fn exec_oneline_multiple_args_with_single_quotes() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	// Windows cmd.exe echo behaves differently - it prints the quotes we add
	// Note: Windows also escapes the quotes with backslashes in the output
	let expected_stdout = if cfg!(windows) {
		r#"repo_a	\"argument with 'quotes'\"
"#
	} else {
		"repo_a\targument with 'quotes'\n"
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec![
			"exec",
			"--oneline",
			"--",
			"echo",
			"argument with 'quotes'",
		])
		.assert()
		.success()
		.stdout(expected_stdout);
}
