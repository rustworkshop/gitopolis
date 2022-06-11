use clap::{Parser, Subcommand};

/// gitopolis, a cli tool for managnig multiple git repositories - https://github.com/timabell/gitopolis/#readme
#[derive(Parser)]
#[clap(author, version, subcommand_required = true)]
struct Args {
	#[clap(subcommand)]
	command: Option<Commands>,

	#[clap(short, long)]
	working_folder: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// add one or more git repos to manage
	Add {
		#[clap(required = true)]
		repo_folder: Vec<String>
	},
}

fn main() {
	let args = Args::parse();

	match &args.working_folder {
		Some(path) => {
			println!("Working folder set to {}", path);
		}
		None => {
			 println!("Working in current folder");
		}
	}

	match &args.command {
		Some(Commands::Add { repo_folder }) => {
			println!("add command received with arg {:?}", repo_folder);
		}
		None =>{
			println!("nada");
		}
	}
}
