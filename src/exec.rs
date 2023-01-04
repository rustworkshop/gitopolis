use crate::repos::Repo;
use std::io::Error;
use std::process::{Child, Command};

pub fn exec(mut exec_args: Vec<String>, repos: Vec<Repo>) {
	let args = exec_args.split_off(1);
	let cmd = &exec_args[0]; // only cmd remaining after split_off above
	for repo in &repos {
		repo_exec(&repo.path, &cmd, &args).expect("Failed to execute command.");
		println!();
	}
}

fn repo_exec(path: &str, cmd: &str, args: &Vec<String>) -> Result<(), Error> {
	println!("🏢 {}> {} {}", path, cmd, args.join(" "));

	// defaults to piping stdout/stderr to parent process output, so no need to specify
	let mut child_process: Child = Command::new(cmd).args(args).current_dir(path).spawn()?;

	let exit_code = &child_process.wait()?;
	if !exit_code.success() {
		eprintln!("Command exited with code {}", exit_code);
	}
	Ok(())
}
