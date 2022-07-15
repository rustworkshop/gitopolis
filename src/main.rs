use clap::{Parser, Subcommand};
use std::process::Command;
mod repos;
use repos::*;
mod storage;
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
	let args = Args::parse();

	match &args.command {
		Some(Commands::Add { repo_folders }) => add_repos(repo_folders),
		Some(Commands::Remove { repo_folders }) => remove_repos(repo_folders),
		Some(Commands::List) => list(),
		Some(Commands::Exec { exec_args }) => exec(exec_args),
		Some(Commands::Tag {
			tag_name,
			repo_folders,
			remove,
		}) => tag_folders(tag_name, repo_folders, &remove),
		None => {
			println!("nada");
		}
	}
}

fn tag_folders(tag_name: &str, repo_folders: &Vec<String>, remove: &bool) {
	let mut repos = load();
	for repo_folder in repo_folders {
		let repo = repos
			.find_repo(repo_folder)
			.expect(&format!("Repo '{}' not found", repo_folder));
		if *remove {
			if let Some(ix) = repo.tags.iter().position(|t| t == tag_name) {
				repo.tags.remove(ix);
			}
		} else {
			repo.tags.push(tag_name.to_string());
		}
	}
	save(repos);
}

fn exec(exec_args: &Vec<String>) {
	let args_copy: &mut Vec<String> = &mut exec_args.to_owned();
	let args = args_copy.split_off(1);
	let cmd = &args_copy[0]; // only cmd remaining after split_off above
	for repo in load() {
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

fn list() {
	let repos: Vec<Repo> = load();
	if repos.len() == 0 {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in repos {
		println!("{}", repo.path);
	}
}

fn add_repos(repo_folders: &Vec<String>) {
	let mut repos: Vec<Repo> = load();
	for repo_folder in repo_folders {
		println!("Adding {} ...", repo_folder);
		if let Some(_) = repos.repo_index(repo_folder) {
			println!("{} already added, ignoring.", repo_folder);
			continue;
		}
		let repo = Repo {
			path: repo_folder.to_owned(),
			tags: Vec::new(),
			// remotes: Vec::new(),
		};
		repos.push(repo);
	}
	save(repos);
	println!("Done.");
}

fn remove_repos(repo_folders: &Vec<String>) {
	let mut repos = load();
	for repo_folder in repo_folders {
		let ix = repos
			.repo_index(repo_folder)
			.expect(&format!("Repo '{}' not found", repo_folder));
		repos.remove(ix);
	}
	save(repos);
}
