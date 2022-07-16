use log::info;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repos {
	// todo: make inner repos private if possible
	pub repos: Vec<Repo>,
}

impl Repos {
	pub fn new() -> Repos {
		Repos { repos: Vec::new() }
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repo {
	pub path: String,
	pub tags: Vec<String>,
	// pub remotes: Vec<Remote>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Remote {
	pub name: String,
	pub url: String,
}

impl Repos {
	pub fn find_repo(&mut self, folder_name: &str) -> Option<&mut Repo> {
		if let Some(ix) = self.repo_index(folder_name) {
			return Some(&mut self.repos[ix]);
		}
		None
	}

	pub fn repo_index(&self, folder_name: &str) -> Option<usize> {
		self.repos.iter().position(|r| r.path == *folder_name)
	}

	pub fn add(&mut self, repo_folders: &Vec<String>) {
		for repo_folder in repo_folders {
			if let Some(_) = self.repo_index(repo_folder) {
				info!("{} already added, ignoring.", repo_folder);
				continue;
			}
			let repo = Repo {
				path: repo_folder.to_owned(),
				tags: Vec::new(),
				// remotes: Vec::new(),
			};
			self.repos.push(repo);
			info!("Added {}", repo_folder);
		}
	}

	pub fn remove(&mut self, repo_folders: &Vec<String>) {
		for repo_folder in repo_folders {
			let ix = self
				.repo_index(repo_folder)
				.expect(&format!("Repo '{}' not found", repo_folder));
			self.repos.remove(ix);
		}
	}
}
