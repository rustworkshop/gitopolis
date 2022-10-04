use crate::repos::Repo;

pub fn list(repos: Vec<Repo>) {
	if repos.len() == 0 {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in &repos {
		println!("{}", repo.path);
	}
}
