use clap::{Parser, Subcommand};
use gitopolis::exec::exec;
use gitopolis::git::GitImpl;
use gitopolis::gitopolis::Gitopolis;
use gitopolis::repos::Repo;
use gitopolis::storage::StorageImpl;
use log::LevelFilter;
use std::io::Write;

/// gitopolis, a cli tool for managing multiple git repositories - https://github.com/timabell/gitopolis - MIT licensed.
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
	/// Show list of repos gitopolis knows about. Use verbose to see tags and urls (tab separated format).
	List {
		#[arg(short, long)]
		tag_name: Option<String>,
		#[clap(short, long)]
		verbose: bool,
	},
	/// Run any shell command. E.g. `gitopolis exec -- git pull`. Double-dash separator indicates end of gitopolis's arguments and prevents arguments to your commands being interpreted by gitopolis.
	Exec {
		#[arg(short, long)]
		tag_name: Option<String>,
		exec_args: Vec<String>,
	},
	/// Add/remove repo tags. Use tags to organise repos and allow running commands against subsets of the repo list.
	Tag {
		/// Remove this tag from these repo_folders.
		#[clap(short, long)]
		remove: bool,
		#[clap(required = true)]
		tag_name: String,
		#[clap(required = true)]
		repo_folders: Vec<String>,
	},
	/// List known tags. Use verbose to list repos per tag.
	Tags {
		#[clap(short, long)]
		verbose: bool,
	},
	/// Use an existing .gitopolis.toml state file to clone any/all missing repositories.
	Clone {
		#[arg(short, long)]
		tag_name: Option<String>,
	},
}

fn main() {
	env_logger::builder()
		.format(|buf, record| writeln!(buf, "{}", record.args())) // turn off log decorations https://docs.rs/env_logger/0.9.0/env_logger/#using-a-custom-format
		.filter(None, LevelFilter::Info) // turn on log output
		.init();

	match &Args::parse().command {
		Some(Commands::Add { repo_folders }) => {
			init_gitopolis().add(repo_folders);
		}
		Some(Commands::Remove { repo_folders }) => {
			init_gitopolis().remove(repo_folders);
		}
		Some(Commands::List { tag_name, verbose }) => {
			list(init_gitopolis().list(tag_name), *verbose)
		}
		Some(Commands::Clone { tag_name }) => clone(tag_name),
		Some(Commands::Exec {
			tag_name,
			exec_args,
		}) => {
			exec(exec_args.to_owned(), init_gitopolis().list(tag_name));
		}
		Some(Commands::Tag {
			tag_name,
			repo_folders,
			remove,
		}) => {
			if *remove {
				init_gitopolis().remove_tag(tag_name, repo_folders);
			} else {
				init_gitopolis().add_tag(tag_name, repo_folders);
			}
		}
		Some(Commands::Tags { verbose }) => list_tags(*verbose),
		None => {
			panic!("no command") // this doesn't happen because help shows instead
		}
	}
}

fn clone(tag_name: &Option<String>) {
	let gitopolis = init_gitopolis();
	gitopolis.clone(gitopolis.list(tag_name))
}

fn init_gitopolis() -> Gitopolis {
	Gitopolis::new(
		Box::new(StorageImpl {
			path: ".gitopolis.toml",
		}),
		Box::new(GitImpl {}),
	)
}

fn list(repos: Vec<Repo>, verbose: bool) {
	if repos.len() == 0 {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in &repos {
		if verbose {
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

fn list_tags(verbose: bool) {
	let gitopolis = &init_gitopolis();
	if verbose {
		for tag in gitopolis.tags() {
			println!("{}", tag);
			for r in gitopolis.list(&Some(tag)) {
				println!("\t{}", r.path);
			}
			println!();
		}
	} else {
		for tag in gitopolis.tags() {
			println!("{}", tag);
		}
	}
}
