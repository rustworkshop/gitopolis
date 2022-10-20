use crate::git::Git;
use crate::repos::{Repo, Repos};
use crate::storage::Storage;
use log::info;
use std::collections::BTreeMap;

pub struct Gitopolis<TGit, TStorage> {
	storage: TStorage,
	git: TGit,
}

impl<TGit, TStorage> Gitopolis<TGit, TStorage>
where
	TGit: Git,
	TStorage: Storage,
{
	pub fn new(storage: TStorage, git: TGit) -> Gitopolis<TGit, TStorage> {
		Gitopolis { storage, git }
	}

	pub fn add(&mut self, repo_folders: &Vec<String>) {
		let mut repos = self.load();
		for repo_folder in repo_folders {
			if let Some(_) = repos.repo_index(repo_folder) {
				info!("{} already added, ignoring.", repo_folder);
				continue;
			}
			// todo: read all remotes, not just origin https://github.com/timabell/gitopolis/i
			let remote_name = "origin";
			let url = self.git.read_url(&repo_folder, remote_name);
			repos.add(repo_folder, url, remote_name);
		}
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
	pub fn list(&self, tag_name: &Option<String>) -> Vec<Repo> {
		let repos = self.load();
		match tag_name {
			None => repos.repos,
			Some(tag) => repos
				.repos
				.into_iter()
				.filter(|r| r.tags.contains(&tag.to_string()))
				.collect(),
		}
	}
	pub fn read(&self) -> Repos {
		self.load()
	}
	pub fn clone(&self, repos: Vec<Repo>) {
		for repo in repos {
			// todo: multiple remote support
			let url = &repo.remotes["origin"].url;
			self.git.clone(repo.path.as_str(), url);
		}
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
