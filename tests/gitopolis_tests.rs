use gitopolis::git::Git;
use gitopolis::gitopolis::{Gitopolis, GitopolisError};
use gitopolis::storage::Storage;

#[test]
fn add() {
	let expected_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";
	let storage = FakeStorage::new()
		.with_file_saved_callback(|state| assert_eq!(expected_toml.to_owned(), state))
		.boxed();
	let git = FakeGit::new().boxed();
	let mut gitopolis = Gitopolis::new(storage, git);

	gitopolis.add("test_repo/".to_string()).expect("Failed");
}

#[test]
fn list() {
	let starting_state = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);
	let tag = &None;
	let actual_repos = gitopolis.list(tag).expect("TODO: panic message");

	let expected_repos = 1;
	assert_eq!(expected_repos, actual_repos.len())
}

#[test]
fn list_corrupt() {
	let starting_state = "[[NOT_A_repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);
	let tag = &None;
	let repos_result = gitopolis.list(tag);
	let actual_error = repos_result.expect_err("should error");
	let expected_error = "Failed to parse state data as valid TOML. TOML parse error at line 1, column 1\n  |\n1 | [[NOT_A_repos]]\n  | ^^^^^^^^^^^^^^^\nmissing field `remotes`\n";
	assert_eq!(expected_error, actual_error.message())
}

#[test]
fn clone() {
	// todo: test cloning more than one repo

	let starting_state = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new()
		.with_clone_callback(Box::new(|actual_path, actual_url| {
			assert_eq!(actual_path, "test_repo");
			assert_eq!(actual_url, "git://example.org/test_url");
		}))
		.boxed();

	let gitopolis = Gitopolis::new(storage, git);

	let tag = &None;
	gitopolis.clone(gitopolis.list(tag).expect("TODO: panic message"));
}

#[test]
fn tag() {
	let starting_state = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let expected_toml = "[[repos]]
path = \"test_repo\"
tags = [\"some_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.with_file_saved_callback(|state| assert_eq!(expected_toml.to_owned(), state))
		.boxed();

	let git = FakeGit::new().boxed();
	let mut gitopolis = Gitopolis::new(storage, git);

	gitopolis
		.add_tag("some_tag", &["test_repo/".to_string()])
		.expect("TODO: panic message");
}

#[test]
fn remove_tag() {
	let starting_state = "[[repos]]
path = \"test_repo\"
tags = [\"some_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let expected_toml = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.with_file_saved_callback(|state| assert_eq!(expected_toml.to_owned(), state))
		.boxed();

	let git = FakeGit::new().boxed();
	let mut gitopolis = Gitopolis::new(storage, git);

	gitopolis
		.remove_tag("some_tag", &["test_repo/".to_string()])
		.expect("TODO: panic message");
}

#[test]
fn tags() {
	let starting_state = "[[repos]]
path = \"repo1\"
tags = [\"some_tag\", \"another_tag\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"

[[repos]]
path = \"repo2\"
tags = [\"some_tag\", \"more_tags\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";
	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);

	let result = gitopolis.tags().expect("TODO: panic message");
	assert_eq!(3, result.len());
	assert_eq!("another_tag", result[0]);
	assert_eq!("more_tags", result[1]);
	assert_eq!("some_tag", result[2]);
}

#[test]
fn list_returns_repos_sorted_alphabetically() {
	let starting_state = "[[repos]]
path = \"zebra_repo\"
tags = [\"zulu\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/zebra\"

[[repos]]
path = \"alpha_repo\"
tags = [\"alpha\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/alpha\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);

	let repos = gitopolis.list(&None).expect("Failed to list repos");

	assert_eq!(2, repos.len());
	assert_eq!("alpha_repo", repos[0].path);
	assert_eq!("zebra_repo", repos[1].path);
}

#[test]
fn list_with_tag_filter_returns_sorted_repos() {
	let starting_state = "[[repos]]
path = \"zebra_repo\"
tags = [\"backend\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/zebra\"

[[repos]]
path = \"middle_repo\"
tags = [\"frontend\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/middle\"

[[repos]]
path = \"alpha_repo\"
tags = [\"backend\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/alpha\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);

	let repos = gitopolis
		.list(&Some("backend".to_string()))
		.expect("Failed to list repos");

	assert_eq!(2, repos.len());
	assert_eq!("alpha_repo", repos[0].path);
	assert_eq!("zebra_repo", repos[1].path);
}

#[test]
fn tags_long_repos_sorted_within_tag() {
	let starting_state = "[[repos]]
path = \"zebra_repo\"
tags = [\"backend\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/zebra\"

[[repos]]
path = \"alpha_repo\"
tags = [\"backend\"]

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/alpha\"\
";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.boxed();

	let git = FakeGit::new().boxed();
	let gitopolis = Gitopolis::new(storage, git);

	let tags = gitopolis.tags().expect("Failed to get tags");
	assert_eq!(1, tags.len());
	assert_eq!("backend", tags[0]);

	let repos = gitopolis
		.list(&Some("backend".to_string()))
		.expect("Failed to list repos");
	assert_eq!(2, repos.len());
	assert_eq!("alpha_repo", repos[0].path);
	assert_eq!("zebra_repo", repos[1].path);
}

#[test]
fn remove() {
	let starting_state = "[[repos]]
path = \"test_repo\"
tags = []

[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
";

	let expected_toml = "repos = []\n";

	let storage = FakeStorage::new()
		.with_contents(starting_state.to_string())
		.with_file_saved_callback(|state| assert_eq!(expected_toml.to_owned(), state))
		.boxed();

	let git = FakeGit::new().boxed();
	let mut gitopolis = Gitopolis::new(storage, git);

	gitopolis
		.remove(&["test_repo/".to_string()])
		.expect("TODO: panic message");
}

#[test]
fn remove_unknown() {
	let storage = FakeStorage::new().boxed();
	let git = FakeGit::new().boxed();
	let mut gitopolis = Gitopolis::new(storage, git);

	gitopolis
		.remove(&["non-existent-repo".to_string()])
		.expect("Failed");
}

struct FakeStorage {
	exists: bool,
	contents: String,
	file_saved_callback: Box<dyn Fn(String)>,
}

// fluent interface for building up fake storage (like the "builder pattern")
impl FakeStorage {
	fn new() -> Self {
		Self {
			exists: false,
			contents: "".to_string(),
			file_saved_callback: Box::new(|_| {}),
		}
	}

	fn with_contents(mut self, contents: String) -> Self {
		self.exists = true;
		self.contents = contents;
		self
	}

	fn with_file_saved_callback<F>(mut self, callback: F) -> Self
	where
		F: Fn(String) + 'static, // todo: would it be possible to shrink lifetime from static?
	{
		self.file_saved_callback = Box::new(callback);
		self
	}

	fn boxed(self) -> Box<dyn Storage> {
		Box::new(self)
	}
}

impl Storage for FakeStorage {
	fn exists(&self) -> bool {
		self.exists
	}

	fn save(&self, state_toml: String) {
		(self.file_saved_callback)(state_toml);
	}

	fn read(&self) -> String {
		self.contents.to_owned()
	}
}

struct FakeGit {
	clone_callback: Box<dyn Fn(String, String)>,
}

// fluent interface for building up fake git
impl FakeGit {
	fn new() -> Self {
		Self {
			clone_callback: Box::new(|_, _| {}),
		}
	}

	fn with_clone_callback(mut self, callback: Box<dyn Fn(String, String)>) -> Self {
		self.clone_callback = callback;
		self
	}

	fn boxed(self) -> Box<Self> {
		Box::new(self)
	}
}

impl Git for FakeGit {
	fn read_url(&self, _path: String, _remote_name: String) -> Result<String, GitopolisError> {
		Ok("git://example.org/test_url".to_string())
	}

	fn read_all_remotes(
		&self,
		_path: String,
	) -> Result<std::collections::BTreeMap<String, String>, GitopolisError> {
		let mut remotes = std::collections::BTreeMap::new();
		remotes.insert(
			"origin".to_string(),
			"git://example.org/test_url".to_string(),
		);
		Ok(remotes)
	}

	fn add_remote(&self, _path: &str, _remote_name: &str, _url: &str) {
		// No-op for fake implementation
	}

	fn clone(&self, path: &str, url: &str) {
		(self.clone_callback)(path.to_owned(), url.to_owned())
	}
}
