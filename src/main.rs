use clap::{Parser, Subcommand};
use gitopolis::exec::exec;
use gitopolis::git::GitImpl;
use gitopolis::gitopolis::Gitopolis;
use gitopolis::repos::Repo;
use gitopolis::storage::StorageImpl;
use log::LevelFilter;
use std::io::Write;

/// gitopolis, a cli tool for managing multiple git repositories - https://github.com/timabell/gitopolis - A-GPL v3.0 licensed.
#[derive(Parser)]
#[clap(author, version, subcommand_required = true)]
struct Args {
	#[clap(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	/// Add one or more git repos to manage.
	Add {
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	/// Remove one or more git repos from gitopolis's list. Leaves actual repo on filesystem alone.
	Remove {
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	/// Show list of repos gitopolis knows about. Use "long" to see tags and urls (tab separated format).
	List {
		#[arg(short, long)]
		tag: Option<String>,
		#[clap(short, long)]
		long: bool,
	},
	/// Run any shell command. E.g. `gitopolis exec -- git pull`. Double-dash separator indicates end of gitopolis's arguments and prevents arguments to your commands being interpreted by gitopolis.
	Exec {
		#[arg(short, long)]
		tag: Option<String>,
		exec_args: Vec<String>,
	},
	/// Add/remove repo tags. Use tags to organise repos and allow running commands against subsets of the repo list.
	Tag {
		/// Remove this tag from these repo_folders.
		#[clap(short, long)]
		remove: bool,
		#[clap(required = true)]
		tag: String,
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	/// List known tags. Use "long" to list repos per tag.
	Tags {
		#[clap(short, long)]
		long: bool,
	},
	/// Use an existing .gitopolis.toml state file to clone any/all missing repositories.
	Clone {
		#[arg(short, long)]
		tag: Option<String>,
	},
}

fn main() {
	env_logger::builder()
		.format(|buf, record| writeln!(buf, "{}", record.args())) // turn off log decorations https://docs.rs/env_logger/0.9.0/env_logger/#using-a-custom-format
		.filter(None, LevelFilter::Info) // turn on log output
		.init();

	match &Args::parse().command {
		Some(Commands::Add { repo_folders }) => add(repo_folders.to_owned()),
		Some(Commands::Remove { repo_folders }) => {
			init_gitopolis()
				.remove(repo_folders)
				.expect("TODO: panic message");
		}
		Some(Commands::List {
			tag: tag_name,
			long,
		}) => list(
			init_gitopolis()
				.list(tag_name)
				.expect("TODO: panic message"),
			*long,
		),
		Some(Commands::Clone { tag: tag_name }) => clone(tag_name),
		Some(Commands::Exec {
			tag: tag_name,
			exec_args,
		}) => {
			exec(
				exec_args.to_owned(),
				init_gitopolis()
					.list(tag_name)
					.expect("TODO: panic message"),
			);
		}
		Some(Commands::Tag {
			tag: tag_name,
			repo_folders,
			remove,
		}) => {
			if *remove {
				init_gitopolis()
					.remove_tag(tag_name, repo_folders)
					.expect("TODO: panic message");
			} else {
				init_gitopolis()
					.add_tag(tag_name, repo_folders)
					.expect("TODO: panic message");
			}
		}
		Some(Commands::Tags { long }) => list_tags(*long),
		None => {
			panic!("no command") // this doesn't happen because help shows instead
		}
	}
}

fn clone(tag_name: &Option<String>) {
	let gitopolis = init_gitopolis();
	gitopolis.clone(gitopolis.list(tag_name).expect("TODO: panic message"))
}

const STATE_FILE: &str = ".gitopolis.toml";

fn init_gitopolis() -> Gitopolis {
	Gitopolis::new(
		Box::new(StorageImpl { path: STATE_FILE }),
		Box::new(GitImpl {}),
	)
}

fn add(repo_folders: Vec<String>) {
	for repo_folder in repo_folders {
		init_gitopolis().add(repo_folder).expect("Add failed");
	}
}

fn list(repos: Vec<Repo>, long: bool) {
	if repos.is_empty() {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in &repos {
		if long {
			println!(
				"{}\t{}\t{}",
				repo.path,
				repo.tags.join(","),
				repo.remotes["origin"].url
			);
		} else {
			println!("{}", repo.path);
		}
	}
}

fn list_tags(long: bool) {
	let gitopolis = &init_gitopolis();
	if long {
		for tag in gitopolis.tags().expect("TODO: panic message") {
			println!("{}", tag);
			for r in gitopolis.list(&Some(tag)).expect("TODO: panic message") {
				println!("\t{}", r.path);
			}
			println!();
		}
	} else {
		for tag in gitopolis.tags().expect("TODO: panic message") {
			println!("{}", tag);
		}
	}
}
