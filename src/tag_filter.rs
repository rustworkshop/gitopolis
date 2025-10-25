/// A filter for repositories based on tag matching with AND/OR logic.
///
/// Tag filtering works as follows:
/// - Each inner Vec represents tags that must ALL be present (AND logic)
/// - Different inner Vecs are ORed together
/// - Empty filter matches all repos
///
/// # Examples
///
/// ```
/// use gitopolis::tag_filter::TagFilter;
///
/// // Match all repos (no filter)
/// let filter = TagFilter::all();
///
/// // Match repos with BOTH "foo" AND "bar"
/// let filter = TagFilter::from_cli_args(&["foo,bar".to_string()]);
///
/// // Match repos with (foo AND bar) OR (baz AND boz)
/// let filter = TagFilter::from_cli_args(&["foo,bar".to_string(), "baz,boz".to_string()]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFilter {
	tag_groups: Vec<Vec<String>>,
}

impl TagFilter {
	/// Create a filter that matches all repositories (no filtering)
	pub fn all() -> Self {
		Self {
			tag_groups: Vec::new(),
		}
	}

	/// Create a filter from CLI tag arguments.
	/// Each argument can contain comma-separated tags (AND logic).
	/// Multiple arguments are ORed together.
	pub fn from_cli_args(tag_args: &[String]) -> Self {
		if tag_args.is_empty() {
			return Self::all();
		}

		let tag_groups = tag_args
			.iter()
			.map(|tag_str| {
				tag_str
					.split(',')
					.map(|s| s.trim().to_string())
					.collect::<Vec<String>>()
			})
			.collect();

		Self { tag_groups }
	}

	/// Check if this filter matches a repository with the given tags.
	pub fn matches(&self, repo_tags: &[String]) -> bool {
		if self.tag_groups.is_empty() {
			// No filter, match everything
			return true;
		}

		// Check if repo matches ANY of the tag groups (OR)
		self.tag_groups.iter().any(|tag_group| {
			// Check if repo has ALL tags in this group (AND)
			tag_group.iter().all(|tag| repo_tags.contains(tag))
		})
	}

	/// Returns true if this is an "all" filter (no filtering)
	pub fn is_all(&self) -> bool {
		self.tag_groups.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn all_filter_matches_everything() {
		let filter = TagFilter::all();
		assert!(filter.matches(&[]));
		assert!(filter.matches(&["foo".to_string()]));
		assert!(filter.matches(&["foo".to_string(), "bar".to_string()]));
		assert!(filter.is_all());
	}

	#[test]
	fn empty_cli_args_creates_all_filter() {
		let filter = TagFilter::from_cli_args(&[]);
		assert!(filter.is_all());
		assert!(filter.matches(&["anything".to_string()]));
	}

	#[test]
	fn single_tag_matches_repos_with_that_tag() {
		let filter = TagFilter::from_cli_args(&["foo".to_string()]);
		assert!(filter.matches(&["foo".to_string()]));
		assert!(filter.matches(&["foo".to_string(), "bar".to_string()]));
		assert!(!filter.matches(&["bar".to_string()]));
		assert!(!filter.matches(&[]));
	}

	#[test]
	fn comma_separated_tags_require_all_and_logic() {
		let filter = TagFilter::from_cli_args(&["foo,bar".to_string()]);
		assert!(filter.matches(&["foo".to_string(), "bar".to_string()]));
		assert!(filter.matches(&["foo".to_string(), "bar".to_string(), "baz".to_string()]));
		assert!(!filter.matches(&["foo".to_string()]));
		assert!(!filter.matches(&["bar".to_string()]));
		assert!(!filter.matches(&[]));
	}

	#[test]
	fn multiple_tag_groups_use_or_logic() {
		let filter = TagFilter::from_cli_args(&["foo,bar".to_string(), "baz,boz".to_string()]);
		// Matches (foo AND bar)
		assert!(filter.matches(&["foo".to_string(), "bar".to_string()]));
		// Matches (baz AND boz)
		assert!(filter.matches(&["baz".to_string(), "boz".to_string()]));
		// Matches both groups
		assert!(filter.matches(&[
			"foo".to_string(),
			"bar".to_string(),
			"baz".to_string(),
			"boz".to_string()
		]));
		// Doesn't match - only has foo (not foo AND bar)
		assert!(!filter.matches(&["foo".to_string()]));
		// Doesn't match - only has baz (not baz AND boz)
		assert!(!filter.matches(&["baz".to_string()]));
	}

	#[test]
	fn whitespace_is_trimmed() {
		let filter = TagFilter::from_cli_args(&[" foo , bar ".to_string()]);
		assert!(filter.matches(&["foo".to_string(), "bar".to_string()]));
	}
}
