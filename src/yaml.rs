//! Parses frontmatter YAML text into a [`serde_json::Value`], applying the
//! narrower-than-YAML-1.2-Core scalar typing this project has always used:
//! only `true`/`false`/`null` (and a few case variants) are recognized
//! implicitly, base-10 integers and floats are recognized, and anything
//! quoted or block-styled stays a string no matter what it looks like.

use saphyr_parser::{Event, Parser, ScalarStyle, StrInput};
use serde_json::{Map, Number, Value};

/// The error returned when the parsed document's root is not a mapping
/// (for example a bare scalar or a sequence).
const NON_MAPPING_ROOT_ERROR: &str = "frontmatter root must be a mapping";

/// Parses `raw` frontmatter text into a JSON object.
///
/// Empty or whitespace-only input, and a YAML document with no content
/// node (for example one that is only comments), both parse successfully
/// to an empty object. A document whose root is not a mapping, or that
/// fails to scan as YAML at all, is reported as an error message rather
/// than panicking.
///
/// # Errors
///
/// Returns the scan error's message if `raw` isn't valid YAML, or
/// [`NON_MAPPING_ROOT_ERROR`] if the parsed document's root is not a
/// mapping.
pub fn parse(raw: &str) -> Result<Value, String> {
    if raw.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    let mut parser = Parser::new_from_str(raw);
    next_event(&mut parser)?; // StreamStart
    next_event(&mut parser)?; // DocumentStart

    let root = match next_event(&mut parser)? {
        Event::DocumentEnd => Value::Object(Map::new()),
        first => {
            let node = parse_node(first, &mut parser)?;
            next_event(&mut parser)?; // DocumentEnd
            node
        }
    };

    match root {
        Value::Object(_) => Ok(root),
        _ => Err(NON_MAPPING_ROOT_ERROR.to_owned()),
    }
}

/// Pulls the next parser event, turning a scan failure or an unexpectedly
/// exhausted stream into the same `Result<_, String>` error type the rest
/// of this module uses.
///
/// # Errors
///
/// Returns the scan error's message on a scan failure, or a fixed message
/// if the stream ends where another event was expected.
fn next_event<'input>(
    parser: &mut Parser<'input, StrInput<'input>>,
) -> Result<Event<'input>, String> {
    match parser.next() {
        Some(Ok((event, _span))) => Ok(event),
        Some(Err(scan_error)) => Err(scan_error.to_string()),
        None => Err("unexpected end of YAML input".to_owned()),
    }
}

/// Converts one parser event - and, for a mapping or sequence, every event
/// up to its matching end - into a JSON value.
///
/// # Errors
///
/// Returns an error for an alias/anchor reference (not supported) or any
/// event that isn't a valid node start, propagating scan errors as they
/// occur while reading a mapping's or sequence's contents.
fn parse_node<'input>(
    event: Event<'input>,
    parser: &mut Parser<'input, StrInput<'input>>,
) -> Result<Value, String> {
    match event {
        Event::MappingStart(..) => parse_mapping(parser),
        Event::SequenceStart(..) => parse_sequence(parser),
        Event::Scalar(value, style, ..) => Ok(resolve_scalar(&value, style)),
        Event::Alias(_) => Err("anchors and aliases are not supported".to_owned()),
        other => Err(format!("unexpected YAML event: {other:?}")),
    }
}

/// Reads mapping entries until `MappingEnd`. Keys are taken as their raw
/// scalar text, never re-typed. A repeated key keeps its first position in
/// the resulting object but its value is overwritten by the later entry.
///
/// # Errors
///
/// Returns an error if a mapping key isn't a scalar, or propagates any
/// error from reading a key's or value's events.
fn parse_mapping<'input>(parser: &mut Parser<'input, StrInput<'input>>) -> Result<Value, String> {
    let mut map = Map::new();
    loop {
        let key_event = next_event(parser)?;
        if matches!(key_event, Event::MappingEnd) {
            return Ok(Value::Object(map));
        }

        let Event::Scalar(key, ..) = key_event else {
            return Err(format!("mapping keys must be scalars, found {key_event:?}"));
        };

        let value_event = next_event(parser)?;
        let value = parse_node(value_event, parser)?;
        map.insert(key.into_owned(), value);
    }
}

/// Reads sequence entries until `SequenceEnd`.
///
/// # Errors
///
/// Propagates any error from reading an item's events.
fn parse_sequence<'input>(parser: &mut Parser<'input, StrInput<'input>>) -> Result<Value, String> {
    let mut items = Vec::new();
    loop {
        let event = next_event(parser)?;
        if matches!(event, Event::SequenceEnd) {
            return Ok(Value::Array(items));
        }
        items.push(parse_node(event, parser)?);
    }
}

/// Applies this project's scalar typing rules to a plain (unquoted,
/// unblocked) scalar. Any other style - single- or double-quoted, literal
/// (`|`), or folded (`>`) - opts out of implicit typing entirely and stays
/// a string verbatim.
fn resolve_scalar(value: &str, style: ScalarStyle) -> Value {
    if style != ScalarStyle::Plain {
        return Value::String(value.to_owned());
    }

    match value {
        "" | "~" | "null" | "Null" | "NULL" => Value::Null,
        "true" | "True" | "TRUE" => Value::Bool(true),
        "false" | "False" | "FALSE" => Value::Bool(false),
        _ => value.parse::<i64>().map_or_else(
            |_| {
                value
                    .parse::<f64>()
                    .ok()
                    .and_then(Number::from_f64)
                    .map_or_else(|| Value::String(value.to_owned()), Value::Number)
            },
            |int| Value::Number(int.into()),
        ),
    }
}
