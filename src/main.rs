use clap::{Parser, Subcommand};

/// gitopolis, a cli tool for managnig multiple git repositories - https://github.com/timabell/gitopolis/#readme
#[derive(Parser)]
#[clap(author, version)]
struct Args {
	#[clap(subcommand)]
	command: Option<Commands>,

	#[clap(short, long)]
	working_folder: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// add one or more git repos to manage
	Add { value: Option<String> },
}

fn main() {
	let args = Args::parse();

	match &args.command {
		Some(Commands::Add { value }) => {
			println!("add command received with arg {:?}", value);
		}
		None =>{
			println!("nada");
		}
	}
}
