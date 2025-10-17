use crate::repos::Repo;
use std::env;
use std::io::{Error, Read};
use std::process::{Child, Command, ExitStatus, Stdio};

pub fn exec(exec_args: Vec<String>, repos: Vec<Repo>, oneline: bool) {
	let mut error_count = 0;
	let mut skipped_count = 0;
	for repo in &repos {
		if !exists(&repo.path) {
			if oneline {
				println!("{}\tRepo folder missing, skipped.", &repo.path);
			} else {
				println!();
				println!("🏢 {}> Repo folder missing, skipped.", &repo.path);
			}
			skipped_count += 1;
			continue;
		}
		if oneline {
			let (output, success) =
				repo_exec_oneline(&repo.path, &exec_args).expect("Failed to execute command.");
			match output {
				Some(output_text) => println!("{}\t{}", &repo.path, output_text),
				None => println!("{}\t", &repo.path),
			}
			if !success {
				error_count += 1;
			}
		} else {
			let exit_status =
				repo_exec(&repo.path, &exec_args).expect("Failed to execute command.");
			if !exit_status.success() {
				error_count += 1
			}
			println!();
		}
	}
	if error_count > 0 || skipped_count > 0 {
		if error_count > 0 {
			eprintln!("{error_count} commands exited with non-zero status code");
		}
		if skipped_count > 0 {
			eprintln!("{skipped_count} repos skipped");
		}
		std::process::exit(1);
	}
}

fn exists(repo_path: &String) -> bool {
	let mut path = env::current_dir().expect("failed to get current working directory");
	path.push(repo_path);
	path.exists() && path.is_dir()
}

fn needs_quoting(arg: &str) -> bool {
	arg.chars().any(|c| {
		c.is_whitespace()
			|| matches!(
				c,
				'|' | '&'
					| ';' | '<' | '>'
					| '(' | ')' | '$'
					| '`' | '\\' | '"'
					| '\'' | '*' | '?'
					| '[' | ']' | '{'
					| '}' | '!' | '#'
			)
	})
}

/// Escapes a string for safe use in a shell command
/// Uses POSIX shell single-quote escaping: wrap in single quotes and escape embedded single quotes
#[cfg(unix)]
fn shell_escape(arg: &str) -> String {
	// For Unix shells, we use single quotes which prevent all interpolation
	// To include a literal single quote, we end the single-quoted string,
	// add an escaped single quote, and start a new single-quoted string
	if arg.contains('\'') {
		format!("'{}'", arg.replace('\'', "'\\''"))
	} else {
		format!("'{}'", arg)
	}
}

/// Escapes a string for safe use in a Windows cmd shell
#[cfg(windows)]
fn shell_escape(arg: &str) -> String {
	// Windows cmd.exe has different escaping rules than Unix shells
	// Only quote if the argument contains special characters
	// Inside quotes, double quotes are escaped by doubling: "" not \"
	let needs_quotes = arg
		.chars()
		.any(|c| c.is_whitespace() || matches!(c, '|' | '&' | '<' | '>' | '(' | ')' | '^' | '"'));

	if needs_quotes {
		if arg.contains('"') {
			format!("\"{}\"", arg.replace('"', "\"\""))
		} else {
			format!("\"{}\"", arg)
		}
	} else {
		arg.to_string()
	}
}

fn format_args_for_display(args: &[String]) -> String {
	args.iter()
		.map(|arg| {
			if needs_quoting(arg) {
				// Use single quotes for simplicity, escape any single quotes in the string
				if arg.contains('\'') {
					// For strings containing single quotes, use double quotes and escape
					format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\""))
				} else {
					format!("'{}'", arg)
				}
			} else {
				arg.clone()
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}

fn repo_exec(path: &str, exec_args: &[String]) -> Result<ExitStatus, Error> {
	// If there's only one argument, pass it directly to allow shell syntax (pipes, redirection, etc.)
	// If there are multiple arguments, escape each one to prevent injection issues
	let command_string = if exec_args.len() == 1 {
		exec_args[0].clone()
	} else {
		exec_args
			.iter()
			.map(|arg| shell_escape(arg))
			.collect::<Vec<_>>()
			.join(" ")
	};
	println!();
	println!("🏢 {}> {}", path, format_args_for_display(exec_args));

	// Execute through shell to support piping, redirection, etc.
	#[cfg(unix)]
	let mut child_process: Child = Command::new("sh")
		.arg("-c")
		.arg(&command_string)
		.current_dir(path)
		.spawn()?;

	#[cfg(windows)]
	let mut child_process: Child = Command::new("cmd")
		.arg("/C")
		.arg(&command_string)
		.current_dir(path)
		.spawn()?;

	let exit_code = &child_process.wait()?;
	if !exit_code.success() {
		eprintln!(
			"Command exited with code {}",
			exit_code.code().expect("exit code missing")
		);
	}
	Ok(*exit_code)
}

fn repo_exec_oneline(path: &str, exec_args: &[String]) -> Result<(Option<String>, bool), Error> {
	// If there's only one argument, pass it directly to allow shell syntax (pipes, redirection, etc.)
	// If there are multiple arguments, escape each one to prevent injection issues
	let command_string = if exec_args.len() == 1 {
		exec_args[0].clone()
	} else {
		exec_args
			.iter()
			.map(|arg| shell_escape(arg))
			.collect::<Vec<_>>()
			.join(" ")
	};

	// Execute through shell to support piping, redirection, etc.
	#[cfg(unix)]
	let mut child_process: Child = Command::new("sh")
		.arg("-c")
		.arg(&command_string)
		.current_dir(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	#[cfg(windows)]
	let mut child_process: Child = Command::new("cmd")
		.arg("/C")
		.arg(&command_string)
		.current_dir(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let mut stdout = String::new();
	if let Some(mut stdout_pipe) = child_process.stdout.take() {
		stdout_pipe.read_to_string(&mut stdout)?;
	}

	let mut stderr = String::new();
	if let Some(mut stderr_pipe) = child_process.stderr.take() {
		stderr_pipe.read_to_string(&mut stderr)?;
	}

	let exit_code = child_process.wait()?;
	let success = exit_code.success();

	// Flatten multi-line output to single line by replacing newlines with spaces
	let stdout_clean = stdout.trim().replace('\n', " ");
	let stderr_clean = stderr.trim().replace('\n', " ");

	// Combine stdout and stderr, with stderr included when command fails
	let output = if !success && !stderr_clean.is_empty() {
		if stdout_clean.is_empty() {
			stderr_clean
		} else {
			format!("{} {}", stdout_clean, stderr_clean)
		}
	} else if !stdout_clean.is_empty() {
		stdout_clean
	} else {
		String::new()
	};

	if output.is_empty() {
		Ok((None, success))
	} else {
		Ok((Some(output), success))
	}
}
