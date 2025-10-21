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
	println!();
	println!("🏢 {}> {}", path, format_args_for_display(exec_args));

	// If single argument, pass directly to shell for interpretation (supports pipes, etc.)
	// If multiple arguments, pass via positional parameters to avoid quoting issues
	#[cfg(unix)]
	let mut child_process: Child = if exec_args.len() == 1 {
		Command::new("sh")
			.arg("-c")
			.arg(&exec_args[0]) // Single arg passed directly for shell interpretation
			.current_dir(path)
			.spawn()?
	} else {
		Command::new("sh")
			.arg("-c")
			.arg(r#""$@""#) // Execute all positional parameters
			.arg("--") // $0 placeholder (ignored)
			.args(exec_args) // These become $1, $2, $3, etc.
			.current_dir(path)
			.spawn()?
	};

	#[cfg(windows)]
	let mut child_process: Child = if exec_args.len() == 1 {
		Command::new("cmd")
			.arg("/C")
			.arg(&exec_args[0]) // Single arg passed directly for shell interpretation
			.current_dir(path)
			.spawn()?
	} else {
		// Windows cmd doesn't have an equivalent to sh -c "$@"
		// We need to join args with proper quoting
		let command_string = exec_args
			.iter()
			.map(|arg| {
				// Quote if contains spaces or special chars
				if arg.contains(' ') || arg.contains('"') || arg.contains('&') || arg.contains('|')
				{
					format!("\"{}\"", arg.replace('"', "\"\""))
				} else {
					arg.clone()
				}
			})
			.collect::<Vec<_>>()
			.join(" ");
		Command::new("cmd")
			.arg("/C")
			.arg(command_string)
			.current_dir(path)
			.spawn()?
	};

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
	// If single argument, pass directly to shell for interpretation (supports pipes, etc.)
	// If multiple arguments, pass via positional parameters to avoid quoting issues
	#[cfg(unix)]
	let mut child_process: Child = if exec_args.len() == 1 {
		Command::new("sh")
			.arg("-c")
			.arg(&exec_args[0]) // Single arg passed directly for shell interpretation
			.current_dir(path)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?
	} else {
		Command::new("sh")
			.arg("-c")
			.arg(r#""$@""#) // Execute all positional parameters
			.arg("--") // $0 placeholder (ignored)
			.args(exec_args) // These become $1, $2, $3, etc.
			.current_dir(path)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?
	};

	#[cfg(windows)]
	let mut child_process: Child = if exec_args.len() == 1 {
		Command::new("cmd")
			.arg("/C")
			.arg(&exec_args[0]) // Single arg passed directly for shell interpretation
			.current_dir(path)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?
	} else {
		// Windows cmd doesn't have an equivalent to sh -c "$@"
		// We need to join args with proper quoting
		let command_string = exec_args
			.iter()
			.map(|arg| {
				// Quote if contains spaces or special chars
				if arg.contains(' ') || arg.contains('"') || arg.contains('&') || arg.contains('|')
				{
					format!("\"{}\"", arg.replace('"', "\"\""))
				} else {
					arg.clone()
				}
			})
			.collect::<Vec<_>>()
			.join(" ");
		Command::new("cmd")
			.arg("/C")
			.arg(command_string)
			.current_dir(path)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?
	};

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
