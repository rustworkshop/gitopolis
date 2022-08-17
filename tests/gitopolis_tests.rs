use gitopolis::gitopolis::Gitopolis;
use gitopolis::repos::Repos;
use gitopolis::storage::Storage;

#[test]
fn add_repo() {
	let expected_toml = "[[repos]]
path = \"foo\"
tags = []
";
	let mut gitopolis = Gitopolis::new(Box::new(FakeStorage {
		exists: false,
		contents: "".to_string(),
		file_saved_callback: Box::new(|state| assert_eq!(expected_toml.to_owned(), state)),
	}));
	let mut folders = Vec::new();
	folders.push("foo".to_string());
	gitopolis.add(&folders);
}

#[test]
fn read() {
	let gitopolis = Gitopolis::new(Box::new(FakeStorage {
		exists: true,
		contents: "[[repos]]
path = \"foo\"
tags = [\"red\"]

[[repos]]
path = \"bar\"
tags = [\"red\"]

[[repos]]
path = \"baz aroony\"
tags = []"
			.to_string(),
		file_saved_callback: Box::new(|_| {}),
	}));
	let r = gitopolis.read();
	assert_eq!(3, r.repos.len())
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
