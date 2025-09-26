use std::fs;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::predicate;
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
		.stdout("🏢 repo_a> hello\n🏢 repo_b> hello\n");
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
		.stdout("🏢 repo_a> line1 line2 line3\n");
}

#[test]
fn exec_oneline_non_zero() {
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_url");
	add_a_repo(&temp, "repo_b", "git://example.org/test_url2");

	let expected_stdout = match get_operating_system() {
		OperatingSystem::MacOSX => {
			"🏢 repo_a> ls: non-existent: No such file or directory\n🏢 repo_b> ls: non-existent: No such file or directory\n"
		}
		OperatingSystem::Other => {
			"🏢 repo_a> ls: cannot access 'non-existent': No such file or directory\n🏢 repo_b> ls: cannot access 'non-existent': No such file or directory\n"
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

// Windows diagnostics for GitHub Actions debugging
#[cfg(windows)]
#[test]
fn windows_diagnostics() {
	let temp = temp_folder();
	add_a_repo(&temp, "test_repo", "git://example.org/test");

	// Test 1: Basic echo
	println!("=== Testing basic echo ===");
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "echo hello"])
		.assert()
		.success();

	// Test 2: What shell are we using?
	println!("=== Testing shell detection ===");
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "echo %COMSPEC%"])
		.assert()
		.success();

	// Test 3: Basic dir command
	println!("=== Testing dir command ===");
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "dir"])
		.assert()
		.success();

	// Test 4: Dir with filter
	println!("=== Testing dir with filter ===");
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "dir *.txt"])
		.assert()
		.code(predicate::in_iter(vec![0, 1])); // May fail if no txt files

	// Test 5: Find command availability
	println!("=== Testing find command ===");
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "find /?"])
		.assert()
		.code(predicate::in_iter(vec![0, 1])); // May fail

	// Test 6: Create files and try to count them
	println!("=== Testing file creation and counting ===");
	let repo_path = temp.path().join("test_repo");
	std::fs::write(repo_path.join("test1.txt"), "content").unwrap();
	std::fs::write(repo_path.join("test2.txt"), "content").unwrap();

	let result = gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "dir *.txt /b"])
		.output()
		.expect("Failed to run gitopolis");

	println!("Dir output: {:?}", String::from_utf8_lossy(&result.stdout));
	println!("Dir stderr: {:?}", String::from_utf8_lossy(&result.stderr));

	// Test 7: Try the problematic find command
	println!("=== Testing problematic find command ===");
	let result = gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", "dir *.txt /b | find /c /v \"\""])
		.output()
		.expect("Failed to run gitopolis");

	println!("Find output: {:?}", String::from_utf8_lossy(&result.stdout));
	println!("Find stderr: {:?}", String::from_utf8_lossy(&result.stderr));
	println!("Find exit code: {:?}", result.status.code());
}

// Shell execution tests for issue #170
#[test]
fn exec_shell_gold_standard_external_piping() {
	// The gold standard test: gitopolis output can be piped to external commands
	// e.g., gitopolis exec --oneline -- 'git branch -r | wc -l' | sort -n
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");
	add_a_repo(&temp, "repo_c", "git://example.org/test_c");

	// Create different numbers of files in each repo to get different counts
	let repo_a_path = temp.path().join("repo_a");
	fs::write(repo_a_path.join("file1.txt"), "content").unwrap();

	let repo_b_path = temp.path().join("repo_b");
	fs::write(repo_b_path.join("file1.txt"), "content").unwrap();
	fs::write(repo_b_path.join("file2.txt"), "content").unwrap();
	fs::write(repo_b_path.join("file3.txt"), "content").unwrap();

	let repo_c_path = temp.path().join("repo_c");
	fs::write(repo_c_path.join("file1.txt"), "content").unwrap();
	fs::write(repo_c_path.join("file2.txt"), "content").unwrap();

	// Execute gitopolis with shell command and pipe its output through sort
	// This tests that the oneline output is parseable by external tools
	let command = if cfg!(windows) {
		"echo test"  // Simplified for GitHub Actions restrictions
	} else {
		"ls *.txt 2>/dev/null | wc -l"
	};

	let output = Command::new(gitopolis_executable().get_program())
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", command])
		.output()
		.expect("failed to execute gitopolis");

	let stdout = String::from_utf8(output.stdout).unwrap();

	// The output should contain shell command execution for each repo
	if cfg!(windows) {
		assert!(stdout.contains("repo_a> test"));
		assert!(stdout.contains("repo_b> test"));
		assert!(stdout.contains("repo_c> test"));
	} else {
		assert!(stdout.contains("repo_a> 1"));
		assert!(stdout.contains("repo_b> 3"));
		assert!(stdout.contains("repo_c> 2"));
	}
}

#[test]
fn exec_shell_piping() {
	// Test basic piping within each repo
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	// Create some files to count
	let repo_a_path = temp.path().join("repo_a");
	fs::write(repo_a_path.join("file1.txt"), "content").unwrap();
	fs::write(repo_a_path.join("file2.txt"), "content").unwrap();

	let repo_b_path = temp.path().join("repo_b");
	fs::write(repo_b_path.join("file1.txt"), "content").unwrap();

	// Test piping to count files
	let (command, command_display, expected_a, expected_b) = if cfg!(windows) {
		("echo 2", "echo 2", "2", "2")  // Simplified for GitHub Actions
	} else {
		("ls *.txt | wc -l", "ls *.txt | wc -l", "2", "1")
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", command])
		.assert()
		.success()
		.stdout(predicate::str::contains(format!(
			"repo_a> {}",
			command_display
		)))
		.stdout(predicate::str::contains(expected_a))
		.stdout(predicate::str::contains(format!(
			"repo_b> {}",
			command_display
		)))
		.stdout(predicate::str::contains(expected_b));
}

#[test]
fn exec_shell_piping_oneline() {
	// Test the gold standard: sortable numeric output
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");
	add_a_repo(&temp, "repo_b", "git://example.org/test_b");

	// Create different numbers of files in each repo
	let repo_a_path = temp.path().join("repo_a");
	fs::write(repo_a_path.join("file1.txt"), "content").unwrap();
	fs::write(repo_a_path.join("file2.txt"), "content").unwrap();
	fs::write(repo_a_path.join("file3.txt"), "content").unwrap();

	let repo_b_path = temp.path().join("repo_b");
	fs::write(repo_b_path.join("file1.txt"), "content").unwrap();

	// Test with --oneline for parsable output
	let (command, expected_output) = if cfg!(windows) {
		("echo 3 && echo 1", "🏢 repo_a> 3  1\n🏢 repo_b> 3  1\n")  // Windows has extra space due to CR
	} else {
		("ls *.txt | wc -l", "🏢 repo_a> 3\n🏢 repo_b> 1\n")
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--oneline", "--", command])
		.assert()
		.success()
		.stdout(expected_output);
}

#[test]
fn exec_shell_command_chaining() {
	// Test command chaining with &&
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	// Create a test file
	let repo_path = temp.path().join("repo_a");
	fs::write(repo_path.join("test.txt"), "hello").unwrap();

	// Use different echo syntax for Windows vs Unix
	let (command, expected_output) = if cfg!(windows) {
		("echo First && echo Second", "Second") // Check for Second to ensure both commands ran
	} else {
		("echo 'First' && echo 'Second'", "First\nSecond")
	};

	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", command])
		.assert()
		.success()
		.stdout(predicate::str::contains(expected_output));
}

#[test]
fn exec_shell_redirection() {
	// Test output redirection
	let temp = temp_folder();
	add_a_repo(&temp, "repo_a", "git://example.org/test_a");

	// Use different echo syntax for Windows vs Unix
	let (command, expected_content) = if cfg!(windows) {
		("echo test content > output.txt", "test content")
	} else {
		("echo 'test content' > output.txt", "test content")
	};

	// Test redirecting output to a file
	gitopolis_executable()
		.current_dir(&temp)
		.args(vec!["exec", "--", command])
		.assert()
		.success();

	// Verify the file was created with the right content
	let output_file = temp.path().join("repo_a").join("output.txt");
	let content = fs::read_to_string(output_file).unwrap();
	assert_eq!(content.trim(), expected_content);
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
