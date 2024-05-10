use crate::repos::Repo;
use std::env;
use std::io::Error;
use std::process::{Child, Command, ExitStatus};

pub fn exec(mut exec_args: Vec<String>, repos: Vec<Repo>) {
	let args = exec_args.split_off(1);
	let cmd = &exec_args[0]; // only cmd remaining after split_off above
	let mut error_count = 0;
	for repo in &repos {
		if !exists(&repo.path) {
			println!("🏢 {}> Repo folder missing, skipped.", &repo.path);
			return;
		}
		let exit_status = repo_exec(&repo.path, cmd, &args).expect("Failed to execute command.");
		if !exit_status.success() {
			error_count += 1
		}
		println!();
	}
	if error_count > 0 {
		eprintln!("{} commands exited with non-zero status code", error_count);
	}
}

fn exists(repo_path: &String) -> bool {
	let mut path = env::current_dir().expect("failed to get current working directory");
	path.push(repo_path);
	path.exists() && path.is_dir()
}

fn repo_exec(path: &str, cmd: &str, args: &Vec<String>) -> Result<ExitStatus, Error> {
	println!();
	println!("🏢 {}> {} {}", path, cmd, args.join(" "));

	// defaults to piping stdout/stderr to parent process output, so no need to specify
	let mut child_process: Child = Command::new(cmd).args(args).current_dir(path).spawn()?;

	let exit_code = &child_process.wait()?;
	if !exit_code.success() {
		eprintln!("Command exited with code {}", exit_code.code().expect("exit code missing"));
	}
	Ok(*exit_code)
}
