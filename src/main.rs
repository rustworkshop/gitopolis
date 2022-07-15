use clap::{Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;
use toml;

const STATE_FILENAME: &str = ".gitopolis.toml";

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
		let repo =
			find_repo(repo_folder, &mut repos).expect(&format!("Repo '{}' not found", repo_folder));
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

fn find_repo<'a>(folder_name: &str, repos: &'a mut Vec<Repo>) -> Option<&'a mut Repo> {
	if let Some(ix) = repo_index(folder_name, &repos) {
		return Some(&mut repos[ix]);
	}
	None
}

fn repo_index(folder_name: &str, repos: &Vec<Repo>) -> Option<usize> {
	repos.iter().position(|r| r.path == *folder_name)
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

#[derive(Debug, Serialize, Deserialize)]
struct Repos {
	repos: Vec<Repo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repo {
	path: String,
	tags: Vec<String>,
	// remotes: Vec<Remote>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Remote {
	name: String,
	url: String,
}

fn add_repos(repo_folders: &Vec<String>) {
	let mut repos: Vec<Repo> = load();
	for repo_folder in repo_folders {
		println!("Adding {} ...", repo_folder);
		if let Some(_) = repo_index(repo_folder, &repos) {
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
		let ix =
			repo_index(repo_folder, &repos).expect(&format!("Repo '{}' not found", repo_folder));
		repos.remove(ix);
	}
	save(repos);
}

fn save(repos: Vec<Repo>) {
	let repos_struct = Repos { repos };

	let state_toml = toml::to_string(&repos_struct).expect("Failed to generate toml for repo list");

	fs::write(STATE_FILENAME, state_toml).expect(&format!("Failed to write {}", STATE_FILENAME));
}

fn load() -> Vec<Repo> {
	if !std::path::Path::new(STATE_FILENAME).exists() {
		return Vec::new();
	}
	let state_toml = fs::read_to_string(STATE_FILENAME).expect("Failed to read state file {}");

	let mut named_container: BTreeMap<&str, Vec<Repo>> =
		toml::from_str(&state_toml).expect(&format!("Failed to parse {}", STATE_FILENAME));

	let repos = named_container
		.remove("repos") // [re]move this rather than taking a ref so that ownership moves with it (borrow checker)
		.expect(&format!("Corrupted state file {}", STATE_FILENAME));
	return repos;
}
