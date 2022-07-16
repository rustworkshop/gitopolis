use crate::{Repo, Repos};
use std::collections::BTreeMap;
use std::fs;
use toml;

const STATE_FILENAME: &str = ".gitopolis.toml";

pub fn save(repos: Repos) {
	let state_toml = toml::to_string(&repos).expect("Failed to generate toml for repo list");
	fs::write(STATE_FILENAME, state_toml).expect(&format!("Failed to write {}", STATE_FILENAME));
}

pub fn load() -> Repos {
	if !std::path::Path::new(STATE_FILENAME).exists() {
		return Repos::new();
	}
	let state_toml = fs::read_to_string(STATE_FILENAME).expect("Failed to read state file {}");

	let mut named_container: BTreeMap<&str, Vec<Repo>> =
		toml::from_str(&state_toml).expect(&format!("Failed to parse {}", STATE_FILENAME));

	let repos = named_container
		.remove("repos") // [re]move this rather than taking a ref so that ownership moves with it (borrow checker)
		.expect(&format!("Corrupted state file {}", STATE_FILENAME));
	return Repos { repos };
}
