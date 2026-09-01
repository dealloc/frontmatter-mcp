//! Behavioral tests for the batch read path
//! (`frontmatter_mcp::tools::resolve_paths` / `read_batch`): the
//! exactly-one-of validation, ordering, `maxFiles`, and per-file error
//! isolation.

mod common;

use common::fixture;
use frontmatter_mcp::tools::{Format, ResolveError, read_batch, resolve_paths};
use std::path::Path;

/// Turns a fixture path into the owned `String` the tool functions take.
fn path(name: &str) -> String {
    fixture(name).to_string_lossy().into_owned()
}

/// Explicit paths are read in the given order, and each result echoes the
/// requested path string.
#[tokio::test]
async fn explicit_paths_are_read_in_order() {
    let paths = vec![path("valid-nested.md"), path("valid-simple.md")];
    let resolved = resolve_paths(Some(&paths), None, 500, Path::new(".")).unwrap();
    let results = read_batch(&resolved, Format::Parsed).await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].path, paths[0]);
    assert_eq!(results[1].path, paths[1]);
}

/// A glob resolves against the given base directory.
#[tokio::test]
async fn glob_resolves_against_base_dir() {
    let base = fixture("glob-sample");
    let resolved = resolve_paths(None, Some("*.md"), 500, &base).unwrap();
    assert_eq!(resolved.len(), 2);
}

/// Supplying both `paths` and `glob` is rejected.
#[test]
fn both_paths_and_glob_is_rejected() {
    let paths = vec![path("valid-simple.md")];
    let error = resolve_paths(Some(&paths), Some("*.md"), 500, Path::new(".")).unwrap_err();
    assert_eq!(error, ResolveError::NotExactlyOne);
    assert_eq!(error.message(), "Provide exactly one of 'paths' or 'glob'.");
}

/// Supplying neither is rejected.
#[test]
fn neither_paths_nor_glob_is_rejected() {
    let error = resolve_paths(None, None, 500, Path::new(".")).unwrap_err();
    assert_eq!(error, ResolveError::NotExactlyOne);
}

/// An empty `paths` array counts as "not provided" and is rejected.
#[test]
fn empty_paths_array_is_rejected() {
    let empty: Vec<String> = Vec::new();
    let error = resolve_paths(Some(&empty), None, 500, Path::new(".")).unwrap_err();
    assert_eq!(error, ResolveError::NotExactlyOne);
}

/// A whitespace-only glob counts as "not provided" and is rejected.
#[test]
fn whitespace_glob_is_rejected() {
    let error = resolve_paths(None, Some("   "), 500, Path::new(".")).unwrap_err();
    assert_eq!(error, ResolveError::NotExactlyOne);
}

/// `maxFiles` caps an explicit `paths` list, not just glob results.
#[test]
fn max_files_caps_explicit_paths() {
    let paths = vec![
        path("valid-simple.md"),
        path("valid-nested.md"),
        path("no-frontmatter.md"),
    ];
    let resolved = resolve_paths(Some(&paths), None, 2, Path::new(".")).unwrap();
    assert_eq!(resolved.len(), 2);
}

/// One unreadable path in a batch produces a `parseError` on that entry
/// only; the others succeed and order is preserved.
#[tokio::test]
async fn one_bad_path_does_not_fail_the_batch() {
    let paths = vec![
        path("valid-simple.md"),
        path("does-not-exist.md"),
        path("valid-nested.md"),
    ];
    let results = read_batch(&paths, Format::Parsed).await;

    assert_eq!(results.len(), 3);
    assert!(results[0].parse_error.is_none());
    assert!(results[1].parse_error.is_some());
    assert!(!results[1].has_frontmatter);
    assert!(results[2].parse_error.is_none());
    assert_eq!(results[1].path, paths[1]);
}
