use crate::gitopolis::GitopolisError;
use crate::gitopolis::GitopolisError::{GitError, GitRemoteError};
use git2::Repository;
use std::path::Path;
use std::process::Command;

pub trait Git {
	fn read_url(&self, path: String, remote_name: String) -> Result<String, GitopolisError>;
	fn clone(&self, path: &str, url: &str);
}

pub struct GitImpl {}

impl Git for GitImpl {
	fn read_url(&self, path: String, remote_name: String) -> Result<String, GitopolisError> {
		let repository = Repository::open(&path).map_err(|error| GitError {
			message: format!("Couldn't open git repo. {}", error.message()),
		})?;
		let remote = repository
			.find_remote(remote_name.as_str())
			.map_err(|error| GitRemoteError {
				message: format!("Remote not found. {}", error.message()),
				remote: remote_name,
			})?;
		let url: String = remote.url().unwrap_or("").to_string();
		Ok(url)
	}

	fn clone(&self, path: &str, url: &str) {
		if Path::new(path).exists() {
			println!("🏢 {}> Already exists, skipped.", path);
			return;
		}
		println!("🏢 {}> Cloning {} ...", path, url);
		let output = Command::new("git")
			.args(&["clone".to_string(), url.to_string(), path.to_string()].to_vec())
			.output()
			.expect(&format!("Error running git clone"));
		let stdout = String::from_utf8(output.stdout).expect("Error converting stdout to string");
		let stderr = String::from_utf8(output.stderr).expect("Error converting stderr to string");
		println!("{}", stdout);
		println!("{}", stderr);
	}
}
