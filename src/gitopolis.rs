use crate::{storage, Repos};

pub struct Gitopolis {}

impl Gitopolis {
	pub(crate) fn new() -> Gitopolis {
		Gitopolis {}
	}
	pub fn add(&mut self, repo_folders: &Vec<String>) {
		let mut repos = storage::load();
		repos.add(repo_folders);
		storage::save(repos)
	}
	pub fn remove(&mut self, repo_folders: &Vec<String>) {
		let mut repos = storage::load();
		repos.remove(repo_folders);
		storage::save(repos)
	}
	pub fn add_tag(&mut self, tag_name: &str, repo_folders: &Vec<String>) {
		let mut repos = storage::load();
		repos.add_tag(tag_name, repo_folders);
		storage::save(repos)
	}
	pub fn remove_tag(&mut self, tag_name: &str, repo_folders: &Vec<String>) {
		let mut repos = storage::load();
		repos.remove_tag(tag_name, repo_folders);
		storage::save(repos)
	}
	pub(crate) fn read(&self) -> Repos {
		storage::load()
	}
}
