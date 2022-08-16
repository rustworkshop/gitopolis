use clap::{Parser, Subcommand};
use gitopolis::exec::exec;
use gitopolis::gitopolis::Gitopolis;
use gitopolis::list::list;
use gitopolis::storage::StorageImpl;
use log::LevelFilter;
use std::io::Write;

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

	let mut gitopolis = Gitopolis::new(Box::new(StorageImpl {
		path: ".gitopolis.toml",
	}));

	match &args.command {
		Some(Commands::Add { repo_folders }) => {
			gitopolis.add(repo_folders);
		}
		Some(Commands::Remove { repo_folders }) => {
			gitopolis.remove(repo_folders);
		}
		Some(Commands::List) => list(gitopolis.read()),
		Some(Commands::Exec { exec_args }) => {
			exec(exec_args.to_owned(), gitopolis.read());
		}
		Some(Commands::Tag {
			tag_name,
			repo_folders,
			remove,
		}) => {
			if *remove {
				gitopolis.remove_tag(tag_name, repo_folders);
			} else {
				gitopolis.add_tag(tag_name, repo_folders);
			}
		}
		None => {
			panic!("no command") // this doesn't happen because help shows instead
		}
	}
}
