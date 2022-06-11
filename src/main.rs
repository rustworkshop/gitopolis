use clap::Parser;

/// gitopolis, a cli tool for managnig multiple git repositories - https://github.com/timabell/gitopolis/#readme
#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {

	#[clap(short, long)]
	foo: String,
}

fn main() {
	let args = Args::parse();

	println!("your arg was: {}", args.foo)
}
