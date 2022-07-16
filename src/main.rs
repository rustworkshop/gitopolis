use clap::{Parser, Subcommand};
use std::process::Command;
mod repos;
use repos::*;
mod storage;
use log::LevelFilter;
use std::io::Write;
use storage::*;

/// gitopolis, a cli tool for managnig multiple git repositories - https://github.com/timabell/gitopolis
#[derive(Parser)]
#[clap(author, version, subcommand_required = true)]
struct Args {
	#[clap(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	/// add one or more git repos to manage
	Add {
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	Remove {
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	List,
	Exec {
		exec_args: Vec<String>,
	},
	Tag {
		/// Remove this tag from these repo_folders
		#[clap(short, long)]
		remove: bool,
		#[clap(required = true)]
		tag_name: String,
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
}

fn main() {
	env_logger::builder()
		.format(|buf, record| writeln!(buf, "{}", record.args())) // turn off log decorations https://docs.rs/env_logger/0.9.0/env_logger/#using-a-custom-format
		.filter(None, LevelFilter::Info) // turn on log output
		.init();

	let args = Args::parse();
	let mut repos = load();

	match &args.command {
		Some(Commands::Add { repo_folders }) => {
			repos.add(repo_folders);
			save(repos)
		}
		Some(Commands::Remove { repo_folders }) => {
			repos.remove(repo_folders);
			save(repos)
		}
		Some(Commands::List) => list(&repos),
		Some(Commands::Exec { exec_args }) => {
			exec(exec_args.to_owned(), &repos);
			save(repos)
		}
		Some(Commands::Tag {
			tag_name,
			repo_folders,
			remove,
		}) => {
			if *remove {
				repos.remove_tag(tag_name, repo_folders);
			} else {
				repos.add_tag(tag_name, repo_folders);
			}
			save(repos)
		}
		None => {
			panic!("no command") // this doesn't happen because help shows instead
		}
	}
}

fn exec(mut exec_args: Vec<String>, repos: &Repos) {
	let args = exec_args.split_off(1);
	let cmd = &exec_args[0]; // only cmd remaining after split_off above
	for repo in &repos.repos {
		repo_exec(&repo.path, &cmd, &args);
	}
}

fn repo_exec(path: &str, cmd: &str, args: &Vec<String>) {
	println!("🌲 {}> {} {:?}", path, cmd, args);
	let output = Command::new(cmd)
		.args(args)
		.current_dir(path)
		.output()
		.expect(&format!("Error running exec {}", cmd));

	let stdout = String::from_utf8(output.stdout).expect("Error converting stdout to string");
	println!("{}", stdout);
	println!();
}

fn list(repos: &Repos) {
	if repos.repos.len() == 0 {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in &repos.repos {
		println!("{}", repo.path);
	}
}
