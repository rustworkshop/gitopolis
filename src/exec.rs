use crate::repos::Repo;
use std::env;
use std::io::{BufRead, BufReader, Error, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;

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

/// Formats command arguments for display with intelligent quoting.
///
/// This function reconstructs a shell command string from parsed arguments,
/// adding quotes where needed to make the output readable and copy-pasteable.
///
/// # Why We Need Heuristics
///
/// By the time we receive the arguments here, the shell has already parsed
/// the user's input and consumed the original quotes. For example:
/// - User types: `gitopolis exec -- git log --since="One Week"`
/// - Shell parses this into separate args: `["git", "log", "--since=One Week"]`
/// - We receive: The value "One Week" with no information about original quoting
///
/// Since we've lost the original quoting style, we use heuristics to reconstruct
/// a readable command that would execute correctly if copy-pasted.
///
/// # Strategy
///
/// - Arguments without special chars: display as-is
/// - Arguments with spaces/special chars: wrap in single quotes
/// - Arguments matching `--flag=value` pattern: quote only the value portion
///   for better readability (e.g., `--since='One Week'` vs `'--since=One Week'`)
/// - Values containing single quotes: use double quotes with escaping
fn format_args_for_display(args: &[String]) -> String {
	args.iter()
		.map(|arg| {
			if needs_quoting(arg) {
				// Check if this is a --flag=value or -flag=value pattern
				if arg.starts_with('-') && arg.contains('=') {
					format_key_value_for_display(arg)
				} else {
					// Not a flag=value pattern, quote the whole thing
					quote_arg(arg)
				}
			} else {
				arg.clone()
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}

/// Formats a --key=value argument, quoting only the value portion if needed.
///
/// For example: `--since=One Week` becomes `--since='One Week'`
fn format_key_value_for_display(arg: &str) -> String {
	let eq_pos = arg.find('=').expect("arg must contain '='");
	let (key, value) = arg.split_at(eq_pos);
	let value = &value[1..]; // Skip the '=' character

	// Quote only the value portion if it needs quoting
	if needs_quoting(value) {
		if value.contains('\'') {
			// For values containing single quotes, use double quotes and escape
			format!(
				"{}=\"{}\"",
				key,
				value.replace('\\', "\\\\").replace('"', "\\\"")
			)
		} else {
			format!("{}='{}'", key, value)
		}
	} else {
		// Value doesn't need quoting, return as-is
		arg.to_string()
	}
}

fn quote_arg(arg: &str) -> String {
	// Use single quotes for simplicity, escape any single quotes in the string
	if arg.contains('\'') {
		// For strings containing single quotes, use double quotes and escape
		format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\""))
	} else {
		format!("'{}'", arg)
	}
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
			.stdin(Stdio::null()) // Prevent interactive prompts/pagers
			.stdout(Stdio::piped()) // Prevent TTY detection for pagers
			.stderr(Stdio::piped())
			.spawn()?
	} else {
		Command::new("sh")
			.arg("-c")
			.arg(r#""$@""#) // Execute all positional parameters
			.arg("--") // $0 placeholder (ignored)
			.args(exec_args) // These become $1, $2, $3, etc.
			.current_dir(path)
			.stdin(Stdio::null()) // Prevent interactive prompts/pagers
			.stdout(Stdio::piped()) // Prevent TTY detection for pagers
			.stderr(Stdio::piped())
			.spawn()?
	};

	#[cfg(windows)]
	let mut child_process: Child = if exec_args.len() == 1 {
		Command::new("cmd")
			.arg("/C")
			.arg(&exec_args[0]) // Single arg passed directly for shell interpretation
			.current_dir(path)
			.stdin(Stdio::null()) // Prevent interactive prompts/pagers
			.stdout(Stdio::piped()) // Prevent TTY detection for pagers
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
			.stdin(Stdio::null()) // Prevent interactive prompts/pagers
			.stdout(Stdio::piped()) // Prevent TTY detection for pagers
			.stderr(Stdio::piped())
			.spawn()?
	};

	// Stream stdout and stderr in real-time using threads
	let stdout = child_process
		.stdout
		.take()
		.expect("Failed to capture stdout");
	let stderr = child_process
		.stderr
		.take()
		.expect("Failed to capture stderr");

	let stdout_thread = thread::spawn(move || {
		let reader = BufReader::new(stdout);
		for line in reader.lines().map_while(Result::ok) {
			println!("{}", line);
		}
	});

	let stderr_thread = thread::spawn(move || {
		let reader = BufReader::new(stderr);
		for line in reader.lines().map_while(Result::ok) {
			eprintln!("{}", line);
		}
	});

	let exit_code = child_process.wait()?;

	// Wait for output threads to finish
	let _ = stdout_thread.join();
	let _ = stderr_thread.join();

	if !exit_code.success() {
		eprintln!(
			"Command exited with code {}",
			exit_code.code().expect("exit code missing")
		);
	}
	Ok(exit_code)
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
			.stdin(Stdio::null())
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
			.stdin(Stdio::null())
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
			.stdin(Stdio::null())
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
			.stdin(Stdio::null())
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_format_args_no_quoting_needed() {
		let args = vec!["git".to_string(), "status".to_string()];
		assert_eq!(format_args_for_display(&args), "git status");
	}

	#[test]
	fn test_format_args_simple_quoting() {
		let args = vec!["echo".to_string(), "hello world".to_string()];
		assert_eq!(format_args_for_display(&args), "echo 'hello world'");
	}

	#[test]
	fn test_format_args_flag_with_value() {
		let args = vec![
			"git".to_string(),
			"log".to_string(),
			"-n".to_string(),
			"5".to_string(),
			"--since=One Week".to_string(),
		];
		assert_eq!(
			format_args_for_display(&args),
			"git log -n 5 --since='One Week'"
		);
	}

	#[test]
	fn test_format_args_flag_with_value_containing_single_quote() {
		let args = vec![
			"git".to_string(),
			"commit".to_string(),
			"-m=Don't panic".to_string(),
		];
		assert_eq!(
			format_args_for_display(&args),
			"git commit -m=\"Don't panic\""
		);
	}

	#[test]
	fn test_format_args_non_flag_key_value() {
		// If it doesn't start with -, quote the whole thing
		let args = vec!["foo=bar baz".to_string()];
		assert_eq!(format_args_for_display(&args), "'foo=bar baz'");
	}

	#[test]
	fn test_format_args_flag_with_no_spaces() {
		let args = vec!["--since=yesterday".to_string()];
		assert_eq!(format_args_for_display(&args), "--since=yesterday");
	}

	#[test]
	fn test_format_args_mixed() {
		let args = vec![
			"git".to_string(),
			"log".to_string(),
			"--author=John Doe".to_string(),
			"--format=%H".to_string(),
			"-n".to_string(),
			"10".to_string(),
		];
		assert_eq!(
			format_args_for_display(&args),
			"git log --author='John Doe' --format=%H -n 10"
		);
	}

	#[test]
	fn test_format_args_with_special_chars() {
		let args = vec!["echo".to_string(), "hello|world".to_string()];
		assert_eq!(format_args_for_display(&args), "echo 'hello|world'");
	}

	#[test]
	fn test_format_args_double_dash_flag() {
		let args = vec!["grep".to_string(), "--exclude=*.tmp files".to_string()];
		assert_eq!(
			format_args_for_display(&args),
			"grep --exclude='*.tmp files'"
		);
	}
}
