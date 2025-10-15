use crate::git::Git;
use crate::gitopolis::GitopolisError::*;
use crate::repos::{Repo, Repos};
use crate::storage::Storage;
use log::info;
use std::collections::BTreeMap;
use std::io;

pub struct Gitopolis {
	storage: Box<dyn Storage>,
	git: Box<dyn Git>,
}

#[derive(Debug)]
pub enum GitopolisError {
	GitError { message: String },
	StateError { message: String },
	GitRemoteError { message: String, remote: String },
	IoError { inner: io::Error },
}

impl GitopolisError {
	pub fn message(&self) -> String {
		match self {
			GitError { message } => message.to_string(),
			StateError { message } => message.to_string(),
			GitRemoteError { message, remote: _ } => message.to_string(),
			IoError { inner } => inner.to_string(),
		}
	}
}

impl Gitopolis {
	pub fn new(storage: Box<dyn Storage>, git: Box<dyn Git>) -> Self {
		Self { storage, git }
	}

	pub fn add(&mut self, repo_folder: String) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		let normalized_folder: String = normalize_folder(repo_folder);
		if repos.repo_index(normalized_folder.to_owned()).is_some() {
			info!("{normalized_folder} already added, ignoring.");
			return Ok(());
		}
		let remotes = self.git.read_all_remotes(normalized_folder.to_owned())?;
		repos.add(normalized_folder, remotes);
		self.save(repos)?;
		Ok(())
	}

	pub fn remove(&mut self, repo_folders: &[String]) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		repos.remove(normalize_folders(repo_folders));
		self.save(repos)
	}
	pub fn add_tag(
		&mut self,
		tag_name: &str,
		repo_folders: &[String],
	) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		repos.add_tag(tag_name, normalize_folders(repo_folders));
		self.save(repos)
	}
	pub fn remove_tag(
		&mut self,
		tag_name: &str,
		repo_folders: &[String],
	) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		repos.remove_tag(tag_name, normalize_folders(repo_folders));
		self.save(repos)
	}
	pub fn list(&self, tag_name: &Option<String>) -> Result<Vec<Repo>, GitopolisError> {
		let repos = self.load()?;
		Ok(match tag_name {
			None => repos.into_vec(),
			Some(tag) => repos
				.into_vec()
				.into_iter()
				.filter(|r| r.tags.contains(&tag.to_string()))
				.collect(),
		})
	}
	pub fn read(&self) -> Result<Repos, GitopolisError> {
		self.load()
	}
	pub fn clone(&self, repos: Vec<Repo>) {
		for repo in repos {
			// Determine which remote to use for cloning (prefer origin)
			let clone_remote_name = if repo.remotes.contains_key("origin") {
				"origin"
			} else {
				repo.remotes.keys().next().map(|s| s.as_str()).unwrap_or("")
			};

			if let Some(clone_remote) = repo.remotes.get(clone_remote_name) {
				// Clone the repo
				self.git.clone(repo.path.as_str(), &clone_remote.url);

				// Add all other remotes
				for (name, remote) in &repo.remotes {
					if name != clone_remote_name {
						self.git.add_remote(&repo.path, name, &remote.url);
					}
				}
			}
		}
	}
	pub fn tags(&self) -> Result<Vec<String>, GitopolisError> {
		let repos = self.load()?;
		let nest_of_tags: Vec<Vec<String>> = repos
			.into_vec()
			.into_iter()
			.map(|r| r.tags.into_iter().collect())
			.collect();
		let mut flat: Vec<String> = nest_of_tags.into_iter().flatten().collect();
		flat.sort();
		flat.dedup();
		Ok(flat)
	}

	fn save(&self, repos: Repos) -> Result<(), GitopolisError> {
		let state_toml = serialize(&repos)?;
		self.storage.save(state_toml);
		Ok(())
	}

	fn load(&self) -> Result<Repos, GitopolisError> {
		if !self.storage.exists() {
			return Ok(Repos::new());
		}

		let state_toml = self.storage.read();

		parse(&state_toml)
	}
}

fn serialize(repos: &Repos) -> Result<String, GitopolisError> {
	toml::to_string(&repos).map_err(|error| StateError {
		message: format!("Failed to generate toml for repo list. {error}"),
	})
}

fn parse(state_toml: &str) -> Result<Repos, GitopolisError> {
	let mut named_container: BTreeMap<String, Vec<Repo>> =
		toml::from_str(state_toml).map_err(|error| StateError {
			message: format!("Failed to parse state data as valid TOML. {error}"),
		})?;

	let repos = named_container
		.remove("repos") // [re]move this rather than taking a ref so that ownership moves with it (borrow checker)
		.expect("Failed to read 'repos' entry from state TOML");
	Ok(Repos::new_with_repos(repos))
}

fn normalize_folders(repo_folders: &[String]) -> Vec<String> {
	repo_folders
		.iter()
		.map(|f| normalize_folder(f.to_string()))
		.collect()
}

fn normalize_folder(repo_folder: String) -> String {
	repo_folder
		.trim_end_matches('/')
		.trim_end_matches('\\')
		.to_string()
}

#[test]
fn test_normalize_folders() {
	let input = vec![
		"foo".to_string(),
		"bar/".to_string(),  // *nix
		"baz\\".to_string(), // windows
	];
	let output = normalize_folders(&input);
	assert_eq!(output, vec!["foo", "bar", "baz"]);
}
