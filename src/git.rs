use std::path::Path;
use std::process::Command;

pub trait Git {
	fn read_url(&self, path: &str, remote_name: &str) -> String;
	fn clone(&self, path: &str, url: &str);
}

pub struct GitImpl {}

impl Git for GitImpl {
	/// hacky call to external git command to get url of origin
	fn read_url(&self, path: &str, remote_name: &str) -> String {
		repo_capture_exec(
			&path,
			"git",
			&["config".to_string(), format!("remote.{}.url", remote_name)].to_vec(),
		)
		.trim()
		.to_owned()
	}

	fn clone(&self, path: &str, url: &str) {
		if Path::new(path).exists() {
			println!("🌲 {}> Already exists, skipped.", path);
			return;
		}
		println!("🌲 {}> Cloning {} ...", path, url);
		let output = Command::new("git")
			.args(&["clone".to_string(), url.to_string(), path.to_string()].to_vec())
			.output()
			.expect(&format!("Error running git clone"));
		let stdout = String::from_utf8(output.stdout).expect("Error converting stdout to string");
		let stderr = String::from_utf8(output.stderr).expect("Error converting stdout to string");
		println!("{}", stdout);
		println!("{}", stderr);
	}
}

/// Run a command and capture the output for use internally
fn repo_capture_exec(path: &str, cmd: &str, args: &Vec<String>) -> String {
	let output = Command::new(cmd)
		.args(args)
		.current_dir(path)
		.output()
		.expect(&format!(
			"Error running external command {} {:?} in folder {}",
			cmd, args, path
		));

	String::from_utf8(output.stdout).expect("Error converting stdout to string")
}
