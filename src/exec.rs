use crate::repos::Repo;
use std::io::Error;
use std::process::Command;

pub fn exec(mut exec_args: Vec<String>, repos: Vec<Repo>) {
	let args = exec_args.split_off(1);
	let cmd = &exec_args[0]; // only cmd remaining after split_off above
	for repo in &repos {
		repo_exec(&repo.path, &cmd, &args).expect(&format!("Error running exec {}", cmd));
	}
}

fn repo_exec(path: &str, cmd: &str, args: &Vec<String>) -> Result<(), Error> {
	println!("🏢 {}> {} {}", path, cmd, args.join(" "));
	let output = Command::new(cmd).args(args).current_dir(path).output()?;

	let stdout = String::from_utf8(output.stdout).expect("Error converting stdout to string");
	let stderr = String::from_utf8(output.stderr).expect("Error converting stderr to string");
	println!("{}", stdout);
	println!("{}", stderr);
	println!();
	Ok(())
}
