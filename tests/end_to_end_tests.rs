use assert_cmd::Command as AssertCommand;
use predicates::prelude::predicate;
use std::fs;
use std::process::Command;
use tempfile::{tempdir, TempDir};

#[test]
fn help() {
	gitopolis_executable()
		.arg("help")
		.assert()
		.success()
		.stdout(predicate::str::contains("Usage: gitopolis"));
}

#[cfg(target_os = "windows")] // only windows (cmd/powerhell) needs to have globs expanded for it, real OS's do it for you in the shell
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

	let expected_long_output = "some_git_folder\tsome_tag,another_tag\tgit://example.org/test_url
some_other_git_folder\t\tgit://example.org/test_url2
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

	let expected_stdout = "🏢 missing_git_folder> Repo folder missing, skipped.
";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "never_called"])
		.assert()
		.success()
		.stdout(expected_stdout);
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

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "ls", "non-existent"])
		.assert()
		.success()
		.stdout(expected_stdout)
		.stderr(predicate::str::contains(
			"2 commands exited with non-zero status code",
		));
}

#[test]
fn exec_invalid_command() {
	let temp = temp_folder();
	add_a_repo(&temp, "some_git_folder", "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "not-a-command"])
		.assert()
		.failure();
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
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "git", "status"])
		.assert()
		.success()
		.stdout(predicate::str::contains("nothing to commit"));
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
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--tag", "some_tag", "--", "git", "status"]) // filter exec to tag otherwise it runs on repos that don't yet exists https://github.com/timabell/gitopolis/issues/29
		.assert()
		.success()
		.stdout(predicate::str::contains("nothing to commit"));
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
		.args(vec!["init"])
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
