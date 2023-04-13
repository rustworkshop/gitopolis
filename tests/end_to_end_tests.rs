use assert_cmd::Command as AssertCommand;
use predicates::prelude::predicate;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::{tempdir, TempDir};

#[test]
fn help() {
	get_binary_cmd()
		.arg("help")
		.assert()
		.success()
		.stdout(predicate::str::contains("Usage: gitopolis"));
}

#[test]
fn list_empty_exit_code_2() {
	get_binary_cmd()
		.arg("list")
		.assert()
		.failure()
		.code(2)
		.stdout(predicate::str::contains("No repos"));
}

#[test]
fn add() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	init_repo(temp.path().join(repo), "git://example.org/test_url");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["add", repo])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added some_git_folder\n"));

	let actual_toml =
		fs::read_to_string(temp.path().join(".gitopolis.toml")).expect("failed to read back toml");
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
fn tag() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["tag", "some_tag", repo])
		.assert()
		.success();

	let actual_toml =
		fs::read_to_string(temp.path().join(".gitopolis.toml")).expect("failed to read back toml");
	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = [\"some_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, actual_toml);
}

#[test]
fn list() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("some_git_folder"));

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list", "-l"])
		.assert()
		.success()
		.stdout(predicate::str::contains(
			"some_git_folder\t\tgit://example.org/test_url",
		));

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list", "--long"])
		.assert()
		.success()
		.stdout(predicate::str::contains(
			"some_git_folder\t\tgit://example.org/test_url",
		));
}

fn add_a_repo(temp: &TempDir, repo: &str, remote_url: &str) {
	init_repo(temp.path().join(repo), remote_url);

	get_binary_cmd()
		.current_dir(temp)
		.args(vec!["add", repo])
		.output()
		.expect("Failed to add repo");
}

fn init_repo(path: PathBuf, remote_url: &str) {
	fs::create_dir_all(&path).expect("create repo dir failed");
	Command::new("git")
		.current_dir(&path)
		.args(vec!["init"])
		.output()
		.expect("git command failed");
	Command::new("git")
		.current_dir(&path)
		.args(vec!["config", "remote.origin.url", remote_url])
		.output()
		.expect("git command failed");
}

fn get_binary_cmd() -> AssertCommand {
	AssertCommand::cargo_bin("gitopolis").expect("failed to find binary")
}
