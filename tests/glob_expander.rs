//! Behavioral tests for `frontmatter_mcp::glob::expand`: `*` vs `**`
//! depth, file-only matching, ordinal sorting, and the `max_files` cap.

mod common;

use common::fixture;
use frontmatter_mcp::glob::expand;

/// `*.md` matches only the top level of the base directory, not nested
/// directories, and never a non-`.md` file.
#[test]
fn single_star_matches_top_level_only() {
    let base = fixture("glob-sample");
    let matches = expand("*.md", &base, 500).unwrap();

    let names: Vec<String> = matches
        .iter()
        .map(|p| p.rsplit(['/', '\\']).next().unwrap().to_owned())
        .collect();
    assert_eq!(names, vec!["a.md", "b.md"]);
}

/// `**/*.md` recurses into nested directories.
#[test]
fn double_star_recurses() {
    let base = fixture("glob-sample");
    let matches = expand("**/*.md", &base, 500).unwrap();

    let names: Vec<String> = matches
        .iter()
        .map(|p| p.rsplit(['/', '\\']).next().unwrap().to_owned())
        .collect();
    assert_eq!(names, vec!["a.md", "b.md", "c.md"]);
}

/// Results are absolute paths, sorted by byte order.
#[test]
fn results_are_absolute_and_ordinal_sorted() {
    let base = fixture("glob-sample");
    let matches = expand("**/*.md", &base, 500).unwrap();

    assert!(
        matches
            .iter()
            .all(|p| std::path::Path::new(p).is_absolute())
    );
    let mut sorted = matches.clone();
    sorted.sort_unstable();
    assert_eq!(matches, sorted);
}

/// `max_files` caps the number of results.
#[test]
fn max_files_caps_results() {
    let base = fixture("glob-sample");
    let matches = expand("**/*.md", &base, 1).unwrap();
    assert_eq!(matches.len(), 1);
}

/// A pattern that matches nothing yields an empty list, not an error.
#[test]
fn no_matches_is_empty() {
    let base = fixture("glob-sample");
    let matches = expand("*.nonexistent", &base, 500).unwrap();
    assert!(matches.is_empty());
}

/// A syntactically invalid glob is an error.
#[test]
fn invalid_pattern_is_an_error() {
    let base = fixture("glob-sample");
    assert!(expand("a/**b", &base, 500).is_err());
}
