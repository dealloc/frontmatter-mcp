//! Behavioral tests for `frontmatter_mcp::yaml::parse`: the deliberately
//! narrow scalar-typing rules this project uses, and the shape of the
//! resulting JSON value.

use frontmatter_mcp::yaml::parse;
use serde_json::json;

/// A flat mapping's string-valued keys parse to JSON strings.
#[test]
fn flat_mapping_parses_to_strings() {
    let value = parse("title: Sample Document\nstatus: draft").unwrap();
    assert_eq!(
        value,
        json!({"title": "Sample Document", "status": "draft"})
    );
}

/// Nested mappings and block sequences parse to nested JSON objects and
/// arrays.
#[test]
fn nested_mapping_and_block_sequence_parse() {
    let value = parse("metadata:\n  owner: alice\n  tags:\n    - draft\n    - internal").unwrap();
    assert_eq!(
        value,
        json!({"metadata": {"owner": "alice", "tags": ["draft", "internal"]}})
    );
}

/// Flow (inline) sequences parse the same as block sequences.
#[test]
fn inline_flow_sequence_parses() {
    let value = parse("tags: [draft, internal]").unwrap();
    assert_eq!(value, json!({"tags": ["draft", "internal"]}));
}

/// Plain integers and floats are recognized implicitly.
#[test]
fn plain_numbers_are_typed() {
    let value = parse("count: 42\nratio: 12.75").unwrap();
    assert_eq!(value, json!({"count": 42, "ratio": 12.75}));
    // An integer must serialize without a trailing ".0" - it's a distinct
    // JSON number kind from the float, not just a differently-valued one.
    assert_eq!(value["count"].to_string(), "42");
}

/// Every recognized casing of the plain boolean and null literals is
/// typed; every other casing is left as a string.
#[test]
fn plain_true_false_null_are_typed_by_exact_casing() {
    let value = parse(
        "a: true\nb: True\nc: TRUE\nd: false\ne: False\nf: FALSE\ng: null\nh: Null\ni: NULL\nj: ~",
    )
    .unwrap();
    assert_eq!(
        value,
        json!({
            "a": true, "b": true, "c": true,
            "d": false, "e": false, "f": false,
            "g": null, "h": null, "i": null, "j": null,
        })
    );

    let mixed_case = parse("k: TrUe").unwrap();
    assert_eq!(mixed_case, json!({"k": "TrUe"}));
}

/// YAML 1.1's `yes`/`no`/`on`/`off` truthiness is not part of this
/// project's scalar rules - they stay plain strings.
#[test]
fn yaml_1_1_booleans_stay_strings() {
    let value = parse("a: yes\nb: no\nc: on\nd: off\ne: y\nf: n").unwrap();
    assert_eq!(
        value,
        json!({"a": "yes", "b": "no", "c": "on", "d": "off", "e": "y", "f": "n"})
    );
}

/// Quoting (or block-styling) a scalar opts it out of implicit typing
/// entirely, even when its text looks like a number, bool, or null.
#[test]
fn quoted_and_block_scalars_are_always_strings() {
    let value = parse(
        "double: \"42\"\nsingle: '42'\nliteral: |\n  42\nfolded: >\n  42\nquoted_bool: \"true\"",
    )
    .unwrap();
    assert_eq!(value["double"], json!("42"));
    assert_eq!(value["single"], json!("42"));
    assert_eq!(value["literal"], json!("42\n"));
    assert_eq!(value["folded"], json!("42\n"));
    assert_eq!(value["quoted_bool"], json!("true"));
}

/// Underscored digit groups, hex/octal prefixes, and other YAML-1.1-ish
/// number spellings are not recognized - only plain base-10 `i64`/`f64`
/// parsing counts, matching `NumberStyles.Integer`/`Float` with an
/// invariant culture.
#[test]
fn non_base10_number_spellings_stay_strings() {
    let value = parse("hex: 0x2A\ngrouped: 1_000\noctal: 0o52").unwrap();
    assert_eq!(value["hex"], json!("0x2A"));
    assert_eq!(value["grouped"], json!("1_000"));
    assert_eq!(value["octal"], json!("0o52"));
}

/// Empty or whitespace-only input parses to an empty object, not an
/// error.
#[test]
fn empty_input_parses_to_empty_object() {
    assert_eq!(parse("").unwrap(), json!({}));
    assert_eq!(parse("   \n\t  ").unwrap(), json!({}));
}

/// Malformed YAML is reported as an error string; parsing never panics.
#[test]
fn malformed_yaml_is_an_error_not_a_panic() {
    let result = parse("title: \"unterminated string\nstatus droid twelve: : :");
    assert!(result.is_err());
}

/// A sequence at the document root is rejected with the exact contract
/// error message, since a projected frontmatter map wouldn't make sense
/// otherwise.
#[test]
fn sequence_root_is_rejected() {
    let result = parse("- just\n- a\n- list");
    assert_eq!(result, Err("frontmatter root must be a mapping".to_owned()));
}

/// A bare scalar at the document root is rejected the same way.
#[test]
fn bare_scalar_root_is_rejected() {
    let result = parse("just a string");
    assert_eq!(result, Err("frontmatter root must be a mapping".to_owned()));
}

/// When a key is repeated, the later value wins but the key keeps its
/// first-seen position in the object.
#[test]
fn duplicate_keys_last_value_wins() {
    let value = parse("title: First\nstatus: draft\ntitle: Second").unwrap();
    assert_eq!(value, json!({"title": "Second", "status": "draft"}));
    let keys: Vec<&String> = value.as_object().unwrap().keys().collect();
    assert_eq!(keys, vec!["title", "status"]);
}

/// Key insertion order is preserved end-to-end, including through JSON
/// serialization.
#[test]
fn key_order_is_preserved_through_serialization() {
    let value = parse("zebra: 1\napple: 2\nmango: 3").unwrap();
    let serialized = serde_json::to_string(&value).unwrap();
    assert_eq!(serialized, r#"{"zebra":1,"apple":2,"mango":3}"#);
}
