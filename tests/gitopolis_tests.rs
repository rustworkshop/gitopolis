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
	let mut gitopolis = Gitopolis::new(
		Box::new(FakeStorage {
			exists: false,
			contents: "".to_string(),
			file_saved_callback: Box::new(|state| assert_eq!(expected_toml.to_owned(), state)),
		}),
		Box::new(FakeGit {}),
	);
	let mut folders = Vec::new();
	folders.push("test_repo".to_string());
	gitopolis.add(&folders);
}

#[test]
fn read() {
	let gitopolis = Gitopolis::new(
		Box::new(FakeStorage {
			exists: true,
			contents: "[[repos]]
path = \"test_repo\"
tags = []
[repos.remotes.origin]
name = \"origin\"
url = \"git://example.org/test_url\"\
"
			.to_string(),
			file_saved_callback: Box::new(|_| {}),
		}),
		Box::new(FakeGit {}),
	);
	let r = gitopolis.read();
	assert_eq!(1, r.repos.len())
}

struct FakeStorage {
	exists: bool,
	contents: String,
	file_saved_callback: Box<dyn Fn(String)>,
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

struct FakeGit {}

impl Git for FakeGit {
	fn read_url(&self, _path: &str, _remote_name: &str) -> String {
		"git://example.org/test_url".to_string()
	}
}
