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
	pub fn push(&mut self, repo: Repo) {
		self.repos.push(repo)
	}
	pub fn remove(&mut self, index: usize) {
		self.repos.remove(index);
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
}
