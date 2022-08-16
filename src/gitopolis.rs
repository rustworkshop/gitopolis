use crate::repos::{Repo, Repos};
use crate::storage::Storage;
use std::collections::BTreeMap;

pub struct Gitopolis {
	storage: Box<dyn Storage>,
}

impl Gitopolis {
	pub fn new(storage: Box<dyn Storage>) -> Gitopolis {
		Gitopolis { storage }
	}

	pub fn add(&mut self, repo_folders: &Vec<String>) {
		let mut repos = self.load();
		repos.add(repo_folders);
		self.save(repos)
	}
	pub fn remove(&mut self, repo_folders: &Vec<String>) {
		let mut repos = self.load();
		repos.remove(repo_folders);
		self.save(repos)
	}
	pub fn add_tag(&mut self, tag_name: &str, repo_folders: &Vec<String>) {
		let mut repos = self.load();
		repos.add_tag(tag_name, repo_folders);
		self.save(repos)
	}
	pub fn remove_tag(&mut self, tag_name: &str, repo_folders: &Vec<String>) {
		let mut repos = self.load();
		repos.remove_tag(tag_name, repo_folders);
		self.save(repos)
	}
	pub fn read(&self) -> Repos {
		self.load()
	}

	fn save(&self, repos: Repos) {
		let state_toml = serialize(&repos);
		self.storage.save(state_toml);
	}

	fn load(&self) -> Repos {
		if !self.storage.exists() {
			return Repos::new();
		}

		let state_toml = self.storage.read();

		parse(&state_toml)
	}
}

fn serialize(repos: &Repos) -> String {
	let state_toml = toml::to_string(&repos).expect("Failed to generate toml for repo list");
	state_toml
}

fn parse(state_toml: &str) -> Repos {
	let mut named_container: BTreeMap<&str, Vec<Repo>> =
		toml::from_str(&state_toml).expect(&format!("Failed to parse {}", ".gitopolis.toml"));

	let repos = named_container
		.remove("repos") // [re]move this rather than taking a ref so that ownership moves with it (borrow checker)
		.expect(&format!("Corrupted state file {}", ".gitopolis.toml"));
	Repos { repos }
}
