use clap::{Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
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
	List,
}

fn main() {
	let args = Args::parse();

	match &args.command {
		Some(Commands::Add { repo_folders }) => add_folders(repo_folders),
		Some(Commands::List) => list(),
		None => {
			println!("nada");
		}
	}
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

#[derive(Debug, Deserialize, Serialize)]
struct Repo {
	path: String,
	// remotes: Vec<Remote>,
	// groups: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Remote {
	name: String,
	url: String,
}

fn add_folders(repo_folders: &Vec<String>) {
	let repos: &mut Vec<Repo> = &mut Vec::new();
	for repo_folder in repo_folders {
		let repo = Repo {
			path: repo_folder.to_owned(),
			// remotes: Vec::new(),
			// groups: Vec::new(),
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

	fs::write(STATE_FILENAME, state_toml).expect(&format!("Failed to write {}", STATE_FILENAME));
}

fn load() -> Vec<Repo> {
	if !std::path::Path::new(STATE_FILENAME).exists() {
		return Vec::new();
	}
	let state_toml = fs::read_to_string(STATE_FILENAME).expect("Failed to read state file {}");
	let named_container: BTreeMap<String, Vec<BTreeMap<String, String>>> =
		toml::from_str(&state_toml).expect(&format!("Failed to parse {}", STATE_FILENAME));
	let vec_of_maps = named_container["repos"].to_owned();
	let repos = vec_of_maps
		.iter()
		.map(|r| Repo {
			path: r["path"].to_owned(),
		})
		.collect();
	return repos;
}
