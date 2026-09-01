//! Behavioral tests for the property-projection path
//! (`frontmatter_mcp::tools::project_one` / `project_batch`).

mod common;

use common::fixture;
use frontmatter_mcp::tools::{project_batch, project_one};
use serde_json::json;

/// Turns a fixture path into the owned `String` the tool functions take.
fn path(name: &str) -> String {
    fixture(name).to_string_lossy().into_owned()
}

/// A requested property present in the file is returned; a requested
/// property that is absent is reported missing, per file.
#[tokio::test]
async fn projects_found_and_missing_per_file() {
    let paths = vec![path("valid-simple.md"), path("valid-nested.md")];
    let props = vec!["title".to_owned(), "author".to_owned()];
    let results = project_batch(&paths, &props).await;

    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0].values.get("title"),
        Some(&json!("Sample Document"))
    );
    assert_eq!(results[0].missing, vec!["author"]);
}

/// A file with no frontmatter reports every requested property missing and
/// returns an empty `values`.
#[tokio::test]
async fn file_without_frontmatter_reports_all_missing() {
    let props = vec!["title".to_owned(), "status".to_owned()];
    let result = project_one(&path("no-frontmatter.md"), &props).await;
    assert!(result.values.is_empty());
    assert_eq!(result.missing, props);
}

/// A dotted path projects a nested value, keyed by the exact dotted
/// string.
#[tokio::test]
async fn nested_dotted_path_is_projected() {
    let props = vec!["metadata.owner".to_owned()];
    let result = project_one(&path("valid-nested.md"), &props).await;
    assert_eq!(result.values.get("metadata.owner"), Some(&json!("alice")));
    assert!(result.missing.is_empty());
}

/// An unreadable file reports every requested property missing with an
/// empty `values`, the same as a file with no frontmatter.
#[tokio::test]
async fn unreadable_file_reports_all_missing() {
    let props = vec!["title".to_owned()];
    let result = project_one(&path("does-not-exist.md"), &props).await;
    assert!(result.values.is_empty());
    assert_eq!(result.missing, props);
}

/// Malformed YAML also reports every requested property missing - the
/// tool can't tell "unparseable" from "all absent".
#[tokio::test]
async fn malformed_yaml_reports_all_missing() {
    let props = vec!["title".to_owned()];
    let result = project_one(&path("malformed-yaml.md"), &props).await;
    assert!(result.values.is_empty());
    assert_eq!(result.missing, props);
}
