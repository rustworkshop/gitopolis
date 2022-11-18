use assert_cmd::Command as AssertCommand;
use predicates::prelude::predicate;
use std::env::set_current_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

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
	init_repo(temp.path().join(repo));
	set_current_dir(temp.path()).expect("chdir failed");

	get_binary_cmd()
		.args(vec!["add", repo])
		.assert()
		.success()
		.stderr(predicate::str::contains("Added some_git_folder\n"));
}

fn init_repo(path: PathBuf) {
	fs::create_dir_all(&path).expect("create repo dir failed");
	set_current_dir(path).expect("chdir failed");
	Command::new("git")
		.args(vec!["init"])
		.output()
		.expect("git command failed");
}

fn get_binary_cmd() -> AssertCommand {
	AssertCommand::cargo_bin("gitopolis").expect("failed to find binary")
}
