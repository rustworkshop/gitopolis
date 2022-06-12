use clap::{Parser, Subcommand};

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

struct Repo {
	folder: String,
}

fn add_folders(repo_folders: &Vec<String>) {
	let repos: &mut Vec<Repo> = &mut Vec::new();
	for repo_folder in repo_folders {
		let repo = Repo {
			folder: repo_folder.to_owned(),
		};
		println!("Adding {} ...", repo.folder);
		repos.push(repo);
	}
	save(&*repos); // &* to pass as *immutable* (dereference+reference) https://stackoverflow.com/questions/41366896/how-to-make-a-rust-mutable-reference-immutable/41367094#41367094
}

fn save(repos: &Vec<Repo>) {
	println!("Saving {} repos...", repos.len())
}
