use gitopolis::gitopolis::Gitopolis;
use gitopolis::storage::Storage;

#[test]
fn read() {
	let g = Gitopolis::new(Box::new(FakeStorage {}));
	let r = g.read();
	assert_eq!(3, r.repos.len())
}

struct FakeStorage {}

impl Storage for FakeStorage {
	fn exists(&self) -> bool {
		true
	}

	fn save(&self, _state_toml: String) {
		todo!()
	}

	fn read(&self) -> String {
		"[[repos]]
path = \"foo\"
tags = [\"red\"]

[[repos]]
path = \"bar\"
tags = [\"red\"]

[[repos]]
path = \"baz aroony\"
tags = []"
			.to_string()
	}
}
