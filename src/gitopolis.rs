use crate::git::Git;
use crate::gitopolis::GitopolisError::*;
use crate::repos::{Repo, RepoInfo, Repos};
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
		repos.add_tag(tag_name, normalize_folders(repo_folders))?;
		self.save(repos)
	}
	pub fn remove_tag(
		&mut self,
		tag_name: &str,
		repo_folders: &[String],
	) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		repos.remove_tag(tag_name, normalize_folders(repo_folders))?;
		self.save(repos)
	}
	pub fn list(&self, tag_name: &Option<String>) -> Result<Vec<Repo>, GitopolisError> {
		let repos = self.load()?;
		let mut result = match tag_name {
			None => repos.into_vec(),
			Some(tag) => repos
				.into_vec()
				.into_iter()
				.filter(|r| r.tags.contains(&tag.to_string()))
				.collect(),
		};
		result.sort_by(|a, b| a.path.cmp(&b.path));
		Ok(result)
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

	pub fn sync_read_remotes(&mut self, tag_name: &Option<String>) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		let repo_list = self.list(tag_name)?;
		let mut error_count = 0;

		for repo in repo_list {
			match self.git.read_all_remotes(repo.path.clone()) {
				Ok(remotes) => {
					// Find the repo in the mutable repos structure and update its remotes
					if let Some(repo_mut) = repos.find_repo(repo.path.clone()) {
						repo_mut.remotes.clear();
						for (name, url) in remotes {
							repo_mut.add_remote(name, url);
						}
						info!("Updated {} with remotes from git", repo.path);
					}
				}
				Err(_) => {
					eprintln!("Warning: Could not read remotes from {}", repo.path);
					error_count += 1;
				}
			}
		}

		self.save(repos)?;

		if error_count > 0 {
			eprintln!("{error_count} repos failed to sync");
			std::process::exit(1);
		}

		Ok(())
	}

	pub fn sync_write_remotes(&self, tag_name: &Option<String>) -> Result<(), GitopolisError> {
		let repo_list = self.list(tag_name)?;
		let mut error_count = 0;

		for repo in repo_list {
			// Get current remotes from git
			let current_remotes = match self.git.read_all_remotes(repo.path.clone()) {
				Ok(remotes) => remotes,
				Err(_) => {
					eprintln!("Warning: Could not write remotes to {}", repo.path);
					error_count += 1;
					continue;
				}
			};

			// Add any missing remotes from config
			for (name, remote) in &repo.remotes {
				if !current_remotes.contains_key(name) {
					self.git.add_remote(&repo.path, name, &remote.url);
					info!("Added remote {} to {}", name, repo.path);
				}
			}
		}

		if error_count > 0 {
			eprintln!("{error_count} repos failed to sync");
			std::process::exit(1);
		}

		Ok(())
	}

	pub fn show(&self, repo_path: &str) -> Result<RepoInfo, GitopolisError> {
		let repos = self.load()?;
		let normalized_path = normalize_folder(repo_path.to_string());

		let repo = repos
			.as_vec()
			.iter()
			.find(|r| r.path == normalized_path)
			.ok_or_else(|| StateError {
				message: format!("Repo '{}' not found", normalized_path),
			})?;

		Ok(RepoInfo {
			path: repo.path.clone(),
			tags: repo.tags.clone(),
			remotes: repo.remotes.clone(),
		})
	}

	pub fn clone_and_add(
		&mut self,
		url: &str,
		target_dir: Option<&str>,
		tags: &[String],
	) -> Result<String, GitopolisError> {
		// Use target_dir if provided, otherwise extract from URL
		let folder_name = match target_dir {
			Some(dir) => dir.to_string(),
			None => extract_repo_name_from_url(url).ok_or_else(|| StateError {
				message: format!("Could not extract repository name from URL: {}", url),
			})?,
		};

		// Clone the repository
		self.git.clone(&folder_name, url);

		// Add the repository to gitopolis
		self.add(folder_name.clone())?;

		// Add tags if any were specified
		if !tags.is_empty() {
			for tag in tags {
				self.add_tag(tag.as_str(), std::slice::from_ref(&folder_name))?;
			}
		}

		Ok(folder_name)
	}

	pub fn move_repo(&mut self, old_path: &str, new_path: &str) -> Result<(), GitopolisError> {
		let mut repos = self.load()?;
		let normalized_old = normalize_folder(old_path.to_string());
		let normalized_new = normalize_folder(new_path.to_string());

		// Find the repo in the config
		let repo = repos
			.as_vec()
			.iter()
			.find(|r| r.path == normalized_old)
			.ok_or_else(|| StateError {
				message: format!("Repo '{}' not found", normalized_old),
			})?
			.clone();

		// Create parent directories if they don't exist
		if let Some(parent) = std::path::Path::new(&normalized_new).parent() {
			if !parent.as_os_str().is_empty() {
				std::fs::create_dir_all(parent).map_err(|e| IoError { inner: e })?;
			}
		}

		// Move the actual folder on the filesystem
		std::fs::rename(&normalized_old, &normalized_new).map_err(|e| IoError { inner: e })?;

		// Update the config: remove old entry and add new one with same tags/remotes
		repos.remove(vec![normalized_old]);
		repos.add_with_tags_and_remotes(normalized_new, repo.tags, repo.remotes);

		self.save(repos)?;
		Ok(())
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

/// Extracts the repository name from a git URL to determine the folder name
/// that git clone would use. Handles SSH, HTTPS URLs, and local paths.
///
/// Examples:
/// - git@github.com:user/repo.git -> repo
/// - https://github.com/user/repo.git -> repo
/// - https://github.com/user/repo -> repo
/// - https://dev.azure.com/org/project/_git/myrepo -> myrepo
/// - source_repo -> source_repo
fn extract_repo_name_from_url(url: &str) -> Option<String> {
	// Split by either / or :
	let parts: Vec<&str> = url.split(&['/', ':'][..]).collect();

	// Get the last non-empty part
	parts
		.iter()
		.rev()
		.find(|s| !s.is_empty())
		.map(|s| s.trim_end_matches(".git").to_string())
}

#[test]
fn test_extract_repo_name_from_url() {
	assert_eq!(
		extract_repo_name_from_url("git@github.com:user/repo.git"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://github.com/user/repo.git"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://github.com/user/repo"),
		Some("repo".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("git@gitlab.com:group/subgroup/project.git"),
		Some("project".to_string())
	);
	assert_eq!(
		extract_repo_name_from_url("https://dev.azure.com/org/project/_git/myrepo"),
		Some("myrepo".to_string())
	);
	// Simple local path
	assert_eq!(
		extract_repo_name_from_url("source_repo"),
		Some("source_repo".to_string())
	);
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
