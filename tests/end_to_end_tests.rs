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

#[test]
fn list_empty_exit_code_2() {
	gitopolis_executable()
		.arg("list")
		.assert()
		.failure()
		.code(2)
		.stdout(predicate::str::contains("No repos"));
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

	let expected_toml = "[[repos]]
path = \"some_git_folder\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	assert_eq!(expected_toml, read_gitopolis_state_toml(&temp));
}

#[test]
fn remove() {
	let temp = temp_folder();
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["remove", repo])
		.assert()
		.success();

	assert_eq!("repos = []\n", read_gitopolis_state_toml(&temp));
}

#[test]
fn tag() {
	let temp = temp_folder();
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "some_tag", repo])
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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "--remove", "some_tag", repo])
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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["tag", "-r", "some_tag", repo])
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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");
	tag_repo(&temp, repo2, "some_tag");
	tag_repo(&temp, repo2, "another_tag");

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");
	tag_repo(&temp, repo2, "some_tag");
	tag_repo(&temp, repo2, "another_tag");

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");
	tag_repo(&temp, repo2, "some_tag");
	tag_repo(&temp, repo2, "another_tag");

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
fn list() {
	let temp = temp_folder();
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

	tag_repo(&temp, repo, "some_tag");
	tag_repo(&temp, repo, "another_tag");

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");
	tag_repo(&temp, repo, "some_tag");
	tag_repo(&temp, repo, "another_tag");

	let expected_long_output = "some_git_folder\tsome_tag,another_tag\tgit://example.org/test_url\nsome_other_git_folder\t\tgit://example.org/test_url2\n";

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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

	let expected_stdout = "🏢 some_git_folder> git config remote.origin.url
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
fn exec_tag() {
	let temp = temp_folder();
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

	let expected_stdout = "🏢 some_git_folder> git config remote.origin.url
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
	let repo = "some_git_folder";
	add_a_repo(&temp, repo, "git://example.org/test_url");
	tag_repo(&temp, repo, "some_tag");
	let repo2 = "some_other_git_folder";
	add_a_repo(&temp, repo2, "git://example.org/test_url2");

	let expected_stdout = "🏢 some_git_folder> git config remote.origin.url
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
fn clone() {
	let temp = temp_folder();
	let repo = "source_repo";
	create_local_repo(&temp, repo);
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

	// check repo is valid by running a command on it
	let expected_exec_stdout = "🏢 some_git_folder> git status
On branch master

No commits yet

nothing to commit (create/copy files and use \"git add\" to track)

";

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", "git", "status"])
		.assert()
		.success()
		.stdout(expected_exec_stdout);
}

fn create_local_repo(temp: &TempDir, repo: &str) {
	create_git_repo(&temp, repo, "git://example.org/test_url");
}

fn tag_repo(temp: &TempDir, repo: &str, tag_name: &str) {
	gitopolis_executable()
		.current_dir(temp)
		.args(vec!["tag", tag_name, repo])
		.output()
		.expect("Failed to tag repo");
}

fn add_a_repo(temp: &TempDir, repo: &str, remote_url: &str) {
	create_git_repo(temp, repo, remote_url);

	gitopolis_executable()
		.current_dir(temp)
		.args(vec!["add", repo])
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
