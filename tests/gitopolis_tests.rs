use gitopolis::git::Git;
use gitopolis::gitopolis::Gitopolis;
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
	let mut folders = Vec::new();
	folders.push("test_repo".to_string());

	gitopolis.add(&folders);
}

#[test]
fn read() {
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
	let actual_repos = gitopolis.list(&None);

	let expected_repos = 1;
	assert_eq!(expected_repos, actual_repos.len())
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

	gitopolis.clone(gitopolis.list(&None));
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

	gitopolis.add_tag("some_tag", &vec!["test_repo".to_string()]);
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

	let result = gitopolis.tags();
	assert_eq!(3, result.len());
	assert_eq!("another_tag", result[0]);
	assert_eq!("more_tags", result[1]);
	assert_eq!("some_tag", result[2]);
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
	fn read_url(&self, _path: &str, _remote_name: &str) -> String {
		"git://example.org/test_url".to_string()
	}

	fn clone(&self, path: &str, url: &str) {
		(self.clone_callback)(path.to_owned(), url.to_owned())
	}
}
