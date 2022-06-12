use clap::{Parser, Subcommand};
use serde_derive::Serialize;
use std::collections::BTreeMap;
use std::fs;
use toml;

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
}

fn main() {
	let args = Args::parse();

	match &args.command {
		Some(Commands::Add { repo_folders }) => add_folders(repo_folders),
		None => {
			println!("nada");
		}
	}
}

#[derive(Debug, Serialize)]
struct Repo {
	path: String,
	remotes: Vec<Remote>,
	groups: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Remote {
	name: String,
	url: String,
}

fn add_folders(repo_folders: &Vec<String>) {
	let repos: &mut Vec<Repo> = &mut Vec::new();
	for repo_folder in repo_folders {
		let repo = Repo {
			path: repo_folder.to_owned(),
			remotes: Vec::new(),
			groups: Vec::new(),
		};
		println!("Adding {} ...", repo.path);
		repos.push(repo);
	}
	save(&*repos); // &* to pass as *immutable* (dereference+reference) https://stackoverflow.com/questions/41366896/how-to-make-a-rust-mutable-reference-immutable/41367094#41367094
	println!("Done.");
}

fn save(repos: &Vec<Repo>) {
	let vec_of_maps: Vec<_> = repos
		.iter()
		.map(|r| {
			let mut repo_config = BTreeMap::new();
			repo_config.insert("path", r.path.to_owned());
			repo_config
		})
		.collect();

	let mut named_container = BTreeMap::new();
	named_container.insert("repos", vec_of_maps);

	let state_toml =
		toml::to_string(&named_container).expect("Failed to generate toml for repo list");

	let state_filename = ".gitopolis.toml";
	fs::write(state_filename, state_toml).expect(&format!("Failed to write {}", state_filename));
}
