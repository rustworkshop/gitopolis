use crate::Repos;

pub fn list(repos: &Repos) {
	if repos.repos.len() == 0 {
		println!("No repos");
		std::process::exit(2);
	}
	for repo in &repos.repos {
		println!("{}", repo.path);
	}
}
