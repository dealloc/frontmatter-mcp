//! Behavioral tests for `frontmatter_mcp::projection::project`: dotted
//! lookups, the present-null vs absent distinction, and request-order
//! output.

use frontmatter_mcp::projection::project;
use serde_json::{Value, json};

/// Turns a `json!` object literal into the `Map` `project` expects.
fn object(value: &Value) -> serde_json::Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// A top-level key is returned in `values`; nothing is missing.
#[test]
fn top_level_property_is_found() {
    let obj = object(&json!({"title": "Doc", "status": "draft"}));
    let (values, missing) = project(&obj, &["title".to_owned()]);
    assert_eq!(values, object(&json!({"title": "Doc"})));
    assert!(missing.is_empty());
}

/// A dotted path walks nested objects; the value is keyed by the exact
/// dotted string that was requested.
#[test]
fn nested_dotted_path_is_found() {
    let obj = object(&json!({"metadata": {"owner": "alice"}}));
    let (values, missing) = project(&obj, &["metadata.owner".to_owned()]);
    assert_eq!(values.get("metadata.owner"), Some(&json!("alice")));
    assert!(missing.is_empty());
}

/// An absent key goes into `missing` and never appears in `values`.
#[test]
fn absent_property_is_missing() {
    let obj = object(&json!({"title": "Doc"}));
    let (values, missing) = project(&obj, &["author".to_owned()]);
    assert!(values.is_empty());
    assert_eq!(missing, vec!["author"]);
}

/// A key whose value is explicitly `null` counts as found - it goes into
/// `values` as `null`, not into `missing`.
#[test]
fn present_but_null_is_found_not_missing() {
    let obj = object(&json!({"owner": null}));
    let (values, missing) = project(&obj, &["owner".to_owned()]);
    assert_eq!(values.get("owner"), Some(&Value::Null));
    assert!(missing.is_empty());
}

/// A dotted path that tries to descend through a non-object segment is
/// missing.
#[test]
fn dotted_path_through_non_object_is_missing() {
    let obj = object(&json!({"metadata": {"tags": ["a", "b"]}}));
    let (values, missing) = project(&obj, &["metadata.tags.owner".to_owned()]);
    assert!(values.is_empty());
    assert_eq!(missing, vec!["metadata.tags.owner"]);
}

/// A multi-property request partitions found and missing names, and
/// `values` follows request order.
#[test]
fn multiple_properties_partition_in_request_order() {
    let obj = object(&json!({"b": 2, "a": 1, "c": 3}));
    let (values, missing) = project(
        &obj,
        &[
            "c".to_owned(),
            "missing1".to_owned(),
            "a".to_owned(),
            "missing2".to_owned(),
        ],
    );
    let keys: Vec<&String> = values.keys().collect();
    assert_eq!(keys, vec!["c", "a"]);
    assert_eq!(missing, vec!["missing1", "missing2"]);
}
