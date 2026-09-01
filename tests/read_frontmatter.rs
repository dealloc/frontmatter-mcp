//! Behavioral tests for the single-file read path
//! (`frontmatter_mcp::tools::read_one` / `build_result`): the `format`
//! rules and how parse failures fall back to returning `raw`.

mod common;

use common::fixture;
use frontmatter_mcp::frontmatter::UNTERMINATED_ERROR;
use frontmatter_mcp::tools::{Format, read_one};
use serde_json::json;

/// Turns a fixture path into the owned `String` the tool functions take.
fn path(name: &str) -> String {
    fixture(name).to_string_lossy().into_owned()
}

/// The default format returns the parsed object and nothing else.
#[tokio::test]
async fn default_format_returns_parsed_only() {
    let result = read_one(&path("valid-simple.md"), Format::Parsed).await;
    assert!(result.has_frontmatter);
    assert_eq!(
        result.parsed,
        Some(json!({"title": "Sample Document", "status": "draft"}))
    );
    assert!(result.raw.is_none());
    assert!(result.parse_error.is_none());
}

/// `Raw` returns the exact frontmatter text and does not parse.
#[tokio::test]
async fn raw_format_returns_text_only() {
    let result = read_one(&path("valid-simple.md"), Format::Raw).await;
    assert_eq!(
        result.raw.as_deref(),
        Some("title: Sample Document\nstatus: draft")
    );
    assert!(result.parsed.is_none());
    assert!(result.parse_error.is_none());
}

/// `Both` returns the text and the parsed object.
#[tokio::test]
async fn both_format_returns_text_and_parsed() {
    let result = read_one(&path("valid-simple.md"), Format::Both).await;
    assert_eq!(
        result.raw.as_deref(),
        Some("title: Sample Document\nstatus: draft")
    );
    assert_eq!(
        result.parsed,
        Some(json!({"title": "Sample Document", "status": "draft"}))
    );
}

/// A file with no frontmatter is a normal result with every optional
/// field absent.
#[tokio::test]
async fn no_frontmatter_leaves_all_fields_absent() {
    let result = read_one(&path("no-frontmatter.md"), Format::Parsed).await;
    assert!(!result.has_frontmatter);
    assert!(result.raw.is_none());
    assert!(result.parsed.is_none());
    assert!(result.parse_error.is_none());
}

/// Malformed YAML: `parsed` is absent, the raw text comes back as a
/// fallback, and `parseError` carries the message.
#[tokio::test]
async fn malformed_yaml_falls_back_to_raw_with_error() {
    let result = read_one(&path("malformed-yaml.md"), Format::Parsed).await;
    assert!(result.has_frontmatter);
    assert!(result.parsed.is_none());
    assert!(result.raw.is_some());
    assert!(result.parse_error.is_some());
}

/// An unterminated block: `parsed` absent, `raw` present, `parseError` is
/// the reader's exact message.
#[tokio::test]
async fn unterminated_block_reports_reader_error() {
    let result = read_one(&path("unterminated.md"), Format::Parsed).await;
    assert!(result.has_frontmatter);
    assert!(result.parsed.is_none());
    assert!(result.raw.is_some());
    assert_eq!(result.parse_error.as_deref(), Some(UNTERMINATED_ERROR));
}

/// A nested document parses through to nested JSON.
#[tokio::test]
async fn nested_document_parses() {
    let result = read_one(&path("valid-nested.md"), Format::Parsed).await;
    let parsed = result.parsed.unwrap();
    assert_eq!(parsed["metadata"]["owner"], json!("alice"));
}

/// A path that doesn't exist becomes a structured result with
/// `parseError` set, not an error return.
#[tokio::test]
async fn missing_file_becomes_structured_error() {
    let result = read_one(&path("does-not-exist.md"), Format::Parsed).await;
    assert!(!result.has_frontmatter);
    assert!(result.parse_error.is_some());
}
