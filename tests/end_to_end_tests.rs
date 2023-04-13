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
fn remove() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["remove", repo])
		.assert()
		.success();

	let actual_toml =
		fs::read_to_string(temp.path().join(".gitopolis.toml")).expect("failed to read back toml");
	assert_eq!("repos = []\n", actual_toml);
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
fn tag_remove() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["tag", "--remove", "some_tag", repo])
		.assert()
		.success();

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
fn tag_remove_short() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["tag", "-r", "some_tag", repo])
		.assert()
		.success();

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
fn list() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

	tag_repo(&temp, repo, "some_tag");
	tag_repo(&temp, repo, "another_tag");

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list"])
		.assert()
		.success()
		.stdout("some_git_folder\nsome_other_git_folder\n");
}

#[test]
fn list_long() {
	let temp = tempdir().expect("get tmp dir failed");
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");
	tag_repo(&temp, repo, "some_tag");
	tag_repo(&temp, repo, "another_tag");

	let expected_long_output = "some_git_folder\tsome_tag,another_tag\tgit://example.org/test_url\nsome_other_git_folder\t\tgit://example.org/test_url2\n";

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list", "-l"])
		.assert()
		.success()
		.stdout(expected_long_output);

	get_binary_cmd()
		.current_dir(&temp)
		.args(vec!["list", "--long"])
		.assert()
		.success()
		.stdout(expected_long_output);
}

fn tag_repo(temp: &TempDir, repo: &str, tag_name: &str) {
	get_binary_cmd()
		.current_dir(temp)
		.args(vec!["tag", tag_name, repo])
		.output()
		.expect("Failed to tag repo");
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
