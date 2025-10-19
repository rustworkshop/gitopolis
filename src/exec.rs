use crate::repos::Repo;
use std::env;
use std::io::{Error, Read};
use std::process::{Command, ExitStatus, Stdio};

enum ShellType {
	PosixLike, // bash, zsh, sh, etc. - supports positional parameters
	#[cfg(windows)]
	Windows, // cmd, pwsh - needs special quoting
}

struct Shell {
	shell_type: ShellType,
	/// Shell executable and arguments, e.g. ["bash", "-c"] or ["cmd", "/s", "/c"]
	shell_invocation: Vec<String>,
}

/// Detects the appropriate shell to use and returns the shell with type and invocation
/// Implements ADR-0003: Full subshell support
/// Note: TTY/interactive handling is deferred to issue #209
fn detect_shell() -> Shell {
	// Check for SHELL environment variable first
	if let Ok(shell) = env::var("SHELL") {
		return Shell {
			shell_type: ShellType::PosixLike,
			shell_invocation: vec![shell, "-c".to_string()],
		};
	}

	// Windows-specific fallbacks
	#[cfg(windows)]
	{
		// Check if we're in PowerShell environment
		if env::var("PSModulePath").is_ok() {
			return Shell {
				shell_type: ShellType::Windows,
				shell_invocation: vec!["pwsh".to_string(), "-NoLogo".to_string(), "-c".to_string()],
			};
		}
		// Default to cmd on Windows
		return Shell {
			shell_type: ShellType::Windows,
			shell_invocation: vec!["cmd".to_string(), "/s".to_string(), "/c".to_string()],
		};
	}

	// POSIX fallback to /bin/sh
	#[cfg(not(windows))]
	{
		Shell {
			shell_type: ShellType::PosixLike,
			shell_invocation: vec!["/bin/sh".to_string(), "-c".to_string()],
		}
	}
}

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

/// Builds a Command for executing args in the detected shell
fn build_shell_command(shell: &Shell, exec_args: &[String], path: &str) -> Command {
	let shell_cmd = &shell.shell_invocation;
	let mut cmd = match shell.shell_type {
		ShellType::PosixLike => {
			let mut cmd = Command::new(&shell_cmd[0]);
			for arg in &shell_cmd[1..shell_cmd.len() - 1] {
				cmd.arg(arg);
			}
			cmd.arg(&shell_cmd[shell_cmd.len() - 1]);

			if exec_args.len() == 1 {
				cmd.arg(&exec_args[0]); // Single arg passed directly for shell interpretation
			} else {
				// Unix-like shells: use positional parameters
				cmd.arg(r#""$@""#); // Execute all positional parameters
				cmd.arg("--"); // $0 placeholder (ignored)
				cmd.args(exec_args); // These become $1, $2, $3, etc.
			}
			cmd
		}
		#[cfg(windows)]
		ShellType::Windows => {
			let mut cmd = Command::new(&shell_cmd[0]);
			for arg in &shell_cmd[1..shell_cmd.len() - 1] {
				cmd.arg(arg);
			}
			cmd.arg(&shell_cmd[shell_cmd.len() - 1]);

			if exec_args.len() == 1 {
				cmd.arg(&exec_args[0]); // Single arg passed directly for shell interpretation
			} else {
				// Windows cmd/pwsh doesn't have an equivalent to sh -c "$@"
				// We need to join args with proper quoting
				let command_string = exec_args
					.iter()
					.map(|arg| {
						// Quote if contains spaces or special chars
						if arg.contains(' ')
							|| arg.contains('"') || arg.contains('&')
							|| arg.contains('|')
						{
							format!("\"{}\"", arg.replace('"', "\"\""))
						} else {
							arg.clone()
						}
					})
					.collect::<Vec<_>>()
					.join(" ");
				cmd.arg(command_string);
			}
			cmd
		}
	};
	cmd.current_dir(path);
	cmd
}

fn repo_exec(path: &str, exec_args: &[String]) -> Result<ExitStatus, Error> {
	println!();
	println!("🏢 {}> {}", path, format_args_for_display(exec_args));

	let shell = detect_shell();
	let mut cmd = build_shell_command(&shell, exec_args, path);
	let mut child_process = cmd.spawn()?;

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
	let shell = detect_shell();
	let mut cmd = build_shell_command(&shell, exec_args, path);
	cmd.stdout(Stdio::piped());
	cmd.stderr(Stdio::piped());
	let mut child_process = cmd.spawn()?;

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
