//! Projects a set of named (optionally dotted) properties out of a parsed
//! frontmatter object, partitioning them into found values and missing
//! names.

use serde_json::{Map, Value};

/// Looks up each name in `properties` within `object` and returns
/// `(values, missing)`:
///
/// - `values` maps each found name - keyed by the exact requested string,
///   dotted form included - to its value. A name that resolves to an
///   explicit `null` counts as found.
/// - `missing` lists, in request order, the names that could not be
///   resolved: an absent key, or a dotted path that passes through a
///   non-object.
#[must_use]
pub fn project(
    object: &Map<String, Value>,
    properties: &[String],
) -> (Map<String, Value>, Vec<String>) {
    let mut values = Map::new();
    let mut missing = Vec::new();

    for property in properties {
        match get_dotted(object, property) {
            Some(found) => {
                values.insert(property.clone(), found.clone());
            }
            None => missing.push(property.clone()),
        }
    }

    (values, missing)
}

/// Resolves a dotted path (`"a.b.c"`) against `object`, walking nested
/// objects one segment at a time. Returns `None` if any segment is absent
/// or if the path tries to descend through a non-object.
fn get_dotted<'a>(object: &'a Map<String, Value>, dotted: &str) -> Option<&'a Value> {
    let mut segments = dotted.split('.');
    let mut current = object.get(segments.next()?)?;
    for segment in segments {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}
