use clap::{Parser, Subcommand};
use gitopolis::exec::exec;
use gitopolis::git::GitImpl;
use gitopolis::gitopolis::Gitopolis;
use gitopolis::repos::Repo;
use gitopolis::storage::StorageImpl;
use gitopolis::tag_filter::TagFilter;
use log::LevelFilter;
use std::io::Write;

/// A CLI tool for managing multiple git repositories
/// License: A-GPL v3.0
/// Repo: https://github.com/rustworkshop/gitopolis
#[derive(Parser)]
#[clap(author, version, subcommand_required = true, verbatim_doc_comment)]
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
		/// Filter by tags. Comma-separated tags use AND logic (e.g., "foo,bar" = foo AND bar).
		/// Multiple --tag flags use OR logic (e.g., "--tag foo,bar --tag baz" = (foo AND bar) OR baz).
		#[arg(short, long)]
		tag: Vec<String>,
		#[clap(short, long)]
		long: bool,
	},
	/// Run any shell command. E.g. `gitopolis exec -- git pull`. Double-dash separator indicates end of gitopolis's arguments and prevents arguments to your commands being interpreted by gitopolis.
	Exec {
		/// Filter by tags. Comma-separated tags use AND logic (e.g., "foo,bar" = foo AND bar).
		/// Multiple --tag flags use OR logic (e.g., "--tag foo,bar --tag baz" = (foo AND bar) OR baz).
		#[arg(short, long)]
		tag: Vec<String>,
		#[arg(long)]
		oneline: bool,
		exec_args: Vec<String>,
	},
	/// Add/remove repo tags. Use tags to organise repos and allow running commands against subsets of the repo list. Supports comma-separated tag lists (e.g., "tag1,tag2,tag3").
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
	/// Clone repository from URL and add to gitopolis, or clone all configured repos from .gitopolis.toml.
	/// This command behaves in two very different ways depending on whether a remote url was provided:
	/// If URL is provided: clones from that URL, extracts repo name, adds to gitopolis (optionally with tags).
	/// If URL is omitted: clones all repos from .gitopolis.toml (filtered by --tag if specified) skipping existing folders. (Useful for setting up new machines/developers from an existing team configuration)
	Clone {
		/// Optional git URL to clone from (e.g., git@github.com:user/repo.git or https://github.com/user/repo).
		/// If omitted, clones repos defined in .gitopolis.toml configuration
		url: Option<String>,
		/// Optional addition to URL - target directory name to clone this url into (like git clone). If omitted, extracts name from URL
		target_dir: Option<String>,
		/// When cloning from URL, all specified tags are applied to the new repo.
		/// When cloning without URL from existing config,
		/// filters repos to clone from configuration by tags; comma-separated tags use AND logic (e.g., "foo,bar" = foo AND bar),
		/// multiple --tag flags use OR logic (e.g., "--tag foo,bar --tag baz" = (foo AND bar) OR baz).
		#[arg(short, long)]
		tag: Vec<String>,
	},
	/// Sync remotes between git repositories and .gitopolis.toml configuration
	Sync {
		/// Update .gitopolis.toml from remotes in git repositories
		#[arg(long, conflicts_with = "write_remotes")]
		read_remotes: bool,
		/// Update git repositories with remotes from .gitopolis.toml
		#[arg(long, conflicts_with = "read_remotes")]
		write_remotes: bool,
		/// Filter by tags. Comma-separated tags use AND logic (e.g., "foo,bar" = foo AND bar).
		/// Multiple --tag flags use OR logic (e.g., "--tag foo,bar --tag baz" = (foo AND bar) OR baz).
		#[arg(short, long)]
		tag: Vec<String>,
	},
	/// Show detailed information about a repository including tags and remotes
	Show {
		#[clap(required = true)]
		repo_folder: String,
	},
	/// Move a repository to a new location, updating gitopolis configuration
	Move {
		#[clap(subcommand)]
		entity: MoveEntity,
	},
}

#[derive(Subcommand)]
enum MoveEntity {
	/// Move a repository to a new location
	Repo {
		/// Current path of the repository
		old_path: String,
		/// New path for the repository
		new_path: String,
	},
}

fn main() {
	env_logger::builder()
		.format(|buf, record| writeln!(buf, "{}", record.args())) // turn off log decorations https://docs.rs/env_logger/0.9.0/env_logger/#using-a-custom-format
		.filter(None, LevelFilter::Info) // turn on log output
		.init();

	match &Args::parse_from(wild::args()).command {
		Some(Commands::Add { repo_folders }) => add(repo_folders.to_owned()),
		Some(Commands::Remove { repo_folders }) => {
			init_gitopolis()
				.remove(repo_folders)
				.expect("Failed to remove repository");
		}
		Some(Commands::List {
			tag: tag_args,
			long,
		}) => {
			let filter = TagFilter::from_cli_args(tag_args);
			list(
				init_gitopolis()
					.list(&filter)
					.expect("Failed to list repositories"),
				*long,
			)
		}
		Some(Commands::Clone {
			url,
			target_dir,
			tag: tag_args,
		}) => clone(url, target_dir, tag_args),
		Some(Commands::Exec {
			tag: tag_args,
			oneline,
			exec_args,
		}) => {
			let filter = TagFilter::from_cli_args(tag_args);
			exec(
				exec_args.to_owned(),
				init_gitopolis()
					.list(&filter)
					.expect("Failed to list repositories for exec"),
				*oneline,
			);
		}
		Some(Commands::Tag {
			tag: tag_name,
			repo_folders,
			remove,
		}) => {
			let tags: Vec<&str> = tag_name.split(',').map(|s| s.trim()).collect();
			for tag in tags {
				let result = if *remove {
					init_gitopolis().remove_tag(tag, repo_folders)
				} else {
					init_gitopolis().add_tag(tag, repo_folders)
				};
				if let Err(error) = result {
					eprintln!("Error: {}", error.message());
					std::process::exit(1);
				}
			}
		}
		Some(Commands::Tags { long }) => list_tags(*long),
		Some(Commands::Sync {
			read_remotes,
			write_remotes,
			tag: tag_args,
		}) => {
			let filter = TagFilter::from_cli_args(tag_args);
			if *read_remotes {
				init_gitopolis()
					.sync_read_remotes(&filter)
					.expect("Sync read failed");
			} else if *write_remotes {
				init_gitopolis()
					.sync_write_remotes(&filter)
					.expect("Sync write failed");
			} else {
				eprintln!("Error: Must specify either --read-remotes or --write-remotes");
				std::process::exit(1);
			}
		}
		Some(Commands::Show { repo_folder }) => {
			show(repo_folder);
		}
		Some(Commands::Move { entity }) => match entity {
			MoveEntity::Repo { old_path, new_path } => {
				match init_gitopolis().move_repo(old_path, new_path) {
					Ok(_) => {
						eprintln!("Moved {} to {}", old_path, new_path);
					}
					Err(error) => {
						eprintln!("Error: {}", error.message());
						std::process::exit(1);
					}
				}
			}
		},
		None => {
			panic!("no command") // this doesn't happen because help shows instead
		}
	}
}

/// Clone repository/repositories with dual behavior depending on URL presence.
///
/// # Behavior
///
/// ## When URL is provided
/// Clones a single repository from the given URL and adds it to gitopolis.
/// All tags from tag_args are flattened and applied to the cloned repository.
/// - Example: `--tag foo,bar --tag baz` results in repo having tags: [foo, bar, baz]
///
/// ## When URL is omitted
/// Clones all repositories from .gitopolis.toml configuration that match the tag filter.
/// Uses AND/OR logic for filtering:
/// - Comma-separated tags within one --tag flag use AND logic
/// - Multiple --tag flags use OR logic
/// - Example: `--tag foo,bar --tag baz,boz` clones repos matching (foo AND bar) OR (baz AND boz)
///
/// # Arguments
///
/// * `url` - Optional git URL to clone from
/// * `target_dir` - Optional target directory name (only used when URL is provided)
/// * `tag_args` - Tag arguments for either applying (with URL) or filtering (without URL)
fn clone(url: &Option<String>, target_dir: &Option<String>, tag_args: &[String]) {
	match url {
		Some(git_url) => clone_from_url(git_url, target_dir, tag_args),
		None => {
			// Clone from .gitopolis.toml with tag filtering
			let gitopolis = init_gitopolis();
			let filter = TagFilter::from_cli_args(tag_args);
			gitopolis.clone(
				gitopolis
					.list(&filter)
					.expect("Failed to list repositories for cloning"),
			);
		}
	}
}

/// Clone a single repository from a URL and add it to gitopolis.
///
/// All tags from tag_args are flattened (comma-separated tags are split)
/// and applied to the cloned repository. No AND/OR filtering logic is used
/// since we're cloning a single repo.
///
/// # Arguments
///
/// * `git_url` - Git URL to clone from
/// * `target_dir` - Optional target directory name. If None, extracts from URL
/// * `tag_args` - Tags to apply to the cloned repo (all tags are flattened)
///
/// # Example
///
/// `--tag foo,bar --tag baz` results in repo having tags: [foo, bar, baz]
fn clone_from_url(git_url: &str, target_dir: &Option<String>, tag_args: &[String]) {
	let mut gitopolis = init_gitopolis();
	// Flatten all tags - when cloning a single repo, all tags are applied (no AND/OR logic)
	let tags: Vec<String> = tag_args
		.iter()
		.flat_map(|s| s.split(',').map(|t| t.trim().to_string()))
		.collect();
	match gitopolis.clone_and_add(git_url, target_dir.as_deref(), &tags) {
		Ok(folder_name) => {
			println!("Successfully cloned and added {}", folder_name);
		}
		Err(error) => {
			eprintln!("Error: {}", error.message());
			std::process::exit(1);
		}
	}
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
			let remotes_str = repo
				.remotes
				.iter()
				.map(|(name, remote)| format!("{}={}", name, remote.url))
				.collect::<Vec<_>>()
				.join(",");
			println!("{}\t{}\t{}", repo.path, repo.tags.join(","), remotes_str);
		} else {
			println!("{}", repo.path);
		}
	}
}

fn list_tags(long: bool) {
	let gitopolis = &init_gitopolis();
	if long {
		for tag in gitopolis.tags().expect("Failed to get tags") {
			println!("{tag}");
			let filter = TagFilter::from_cli_args(std::slice::from_ref(&tag));
			for r in gitopolis
				.list(&filter)
				.expect("Failed to list repositories for tag")
			{
				println!("\t{}", r.path);
			}
			println!();
		}
	} else {
		for tag in gitopolis.tags().expect("Failed to get tags") {
			println!("{tag}");
		}
	}
}

fn show(repo_folder: &str) {
	let gitopolis = init_gitopolis();
	match gitopolis.show(repo_folder) {
		Ok(repo_info) => {
			println!("Tags:");
			if repo_info.tags.is_empty() {
				println!("  (none)");
			} else {
				for tag in &repo_info.tags {
					println!("  {}", tag);
				}
			}
			println!();

			println!("Remotes:");
			if repo_info.remotes.is_empty() {
				println!("  (none)");
			} else {
				for (name, remote) in &repo_info.remotes {
					println!("  {}: {}", name, remote.url);
				}
			}
		}
		Err(error) => {
			eprintln!("Error: {}", error.message());
			std::process::exit(1);
		}
	}
}
