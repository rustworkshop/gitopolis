use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repos {
	pub repos: Vec<Repo>,
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

// todo: convert to methods

pub fn find_repo<'a>(folder_name: &str, repos: &'a mut Vec<Repo>) -> Option<&'a mut Repo> {
	if let Some(ix) = repo_index(folder_name, &repos) {
		return Some(&mut repos[ix]);
	}
	None
}

pub fn repo_index(folder_name: &str, repos: &Vec<Repo>) -> Option<usize> {
	repos.iter().position(|r| r.path == *folder_name)
}
