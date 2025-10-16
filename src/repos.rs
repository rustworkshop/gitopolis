use log::info;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Repos {
	repos: Vec<Repo>,
}

#[derive(Debug)]
pub struct RepoInfo {
	pub path: String,
	pub tags: Vec<String>,
	pub remotes: BTreeMap<String, Remote>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repo {
	pub path: String,
	pub tags: Vec<String>,
	pub remotes: BTreeMap<String, Remote>,
}

impl Repo {
	fn new(path: String) -> Self {
		Self {
			path,
			tags: vec![],
			remotes: Default::default(),
		}
	}
	pub(crate) fn add_remote(&mut self, name: String, url: String) {
		self.remotes.insert(name.clone(), Remote { name, url });
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
	pub name: String,
	pub url: String,
}

impl Repos {
	pub fn as_vec(&self) -> &Vec<Repo> {
		&self.repos
	}

	pub fn into_vec(self) -> Vec<Repo> {
		self.repos
	}

	pub fn new() -> Self {
		Default::default()
	}

	pub fn new_with_repos(repos: Vec<Repo>) -> Self {
		Repos { repos }
	}

	pub fn find_repo(&mut self, folder_name: String) -> Option<&mut Repo> {
		if let Some(ix) = self.repo_index(folder_name) {
			return Some(&mut self.repos[ix]);
		}
		None
	}

	pub fn repo_index(&self, folder_name: String) -> Option<usize> {
		self.repos.iter().position(|r| r.path == *folder_name)
	}

	pub fn add(&mut self, repo_folder: String, remotes: BTreeMap<String, String>) {
		let mut repo = Repo::new(repo_folder.clone());
		for (name, url) in remotes {
			repo.add_remote(name, url);
		}
		self.repos.push(repo);
		info!("Added {repo_folder}");
	}

	pub fn remove(&mut self, repo_folders: Vec<String>) {
		for repo_folder in repo_folders {
			match self.repo_index(repo_folder.to_owned()) {
				Some(ix) => {
					self.repos.remove(ix);
				}
				None => {
					info!("Repo already absent, skipped: {repo_folder}")
				}
			}
		}
	}

	pub fn add_tag(&mut self, tag_name: &str, repo_folders: Vec<String>) {
		self.tag(tag_name, repo_folders, false)
	}
	pub fn remove_tag(&mut self, tag_name: &str, repo_folders: Vec<String>) {
		self.tag(tag_name, repo_folders, true)
	}
	fn tag(&mut self, tag_name: &str, repo_folders: Vec<String>, remove: bool) {
		for repo_folder in repo_folders {
			let repo = self
				.find_repo(repo_folder.to_owned())
				.unwrap_or_else(|| panic!("Repo '{repo_folder}' not found"));
			if remove {
				if let Some(ix) = repo.tags.iter().position(|t| t == tag_name) {
					repo.tags.remove(ix);
				}
			} else if !repo.tags.iter().any(|s| s == &tag_name.to_string()) {
				repo.tags.push(tag_name.to_string());
			}
		}
	}
}

#[test]
fn idempotent_tag() {
	let mut repos = Repos::new();
	let path = "repo_path".to_string();
	let mut remotes = BTreeMap::new();
	remotes.insert("origin".to_string(), "url".to_string());
	repos.add(path.to_string(), remotes);
	let tag = "tag_name";
	repos.add_tag(tag, vec![path.to_owned()]);
	repos.add_tag(tag, vec![path.to_owned()]);
	let repo = repos.find_repo(path).expect("repo awol");
	assert_eq!(1, repo.tags.len());
	assert_eq!(tag, repo.tags[0]);
}
