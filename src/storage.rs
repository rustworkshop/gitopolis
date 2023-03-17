use std::fs;

// Abstract away storage to allow testing via dependency injection
pub trait Storage {
	fn exists(&self) -> bool;
	fn save(&self, state_toml: String);
	fn read(&self) -> String;
}

// The struct used in production code
pub struct StorageImpl<'a> {
	pub path: &'a str,
}

impl Storage for Box<dyn Storage + 'static> {
	fn exists(&self) -> bool {
		self.as_ref().exists()
	}

	fn save(&self, state_toml: String) {
		self.as_ref().save(state_toml)
	}

	fn read(&self) -> String {
		self.as_ref().read()
	}
}

// The implementation used in production code
impl<'a> Storage for StorageImpl<'a> {
	fn exists(&self) -> bool {
		std::path::Path::new(self.path).exists()
	}

	fn save(&self, state_toml: String) {
		fs::write(self.path, state_toml)
			.unwrap_or_else(|_| panic!("Failed to write {}", self.path));
	}

	fn read(&self) -> String {
		fs::read_to_string(self.path).expect("Failed to read state file {}")
	}
}
