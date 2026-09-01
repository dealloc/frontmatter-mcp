//! The tool logic, independent of the MCP transport: the `format` enum,
//! the response shapes, path resolution, and the per-file readers the
//! three tools are built from.

use std::path::Path;

use futures_util::future::join_all;
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::frontmatter::{self, Extraction};
use crate::{projection, yaml};

/// The default `maxFiles` cap applied to both an explicit `paths` list and
/// a glob's results.
pub const DEFAULT_MAX_FILES: usize = 500;

/// Supplies [`DEFAULT_MAX_FILES`] as a serde field default.
#[must_use]
pub fn default_max_files() -> usize {
    DEFAULT_MAX_FILES
}

/// Which representation of a file's frontmatter `read_frontmatter` (and
/// the batch variant) should return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub enum Format {
    /// The parsed frontmatter as a JSON object (the default).
    #[default]
    Parsed,
    /// The exact frontmatter text, unparsed.
    Raw,
    /// Both the parsed object and the raw text.
    Both,
}

impl Format {
    /// Whether this format asks for the parsed object.
    fn wants_parsed(self) -> bool {
        matches!(self, Self::Parsed | Self::Both)
    }

    /// Whether this format asks for the raw text.
    fn wants_raw(self) -> bool {
        matches!(self, Self::Raw | Self::Both)
    }
}

/// The result of reading one file's frontmatter. `raw`, `parsed`, and
/// `parseError` are serialized as explicit `null` when absent.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterResult {
    /// The path exactly as requested.
    pub path: String,
    /// Whether the file has a `---`-delimited frontmatter block.
    pub has_frontmatter: bool,
    /// The exact frontmatter text, when the format asks for it or when
    /// parsing failed and it's returned as a fallback.
    pub raw: Option<String>,
    /// The parsed frontmatter object, when the format asks for it and
    /// parsing succeeded.
    pub parsed: Option<Value>,
    /// A reader error, a YAML parse error, or an I/O error message.
    pub parse_error: Option<String>,
}

/// The result of projecting named properties out of one file's
/// frontmatter.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropertyResult {
    /// The path exactly as requested.
    pub path: String,
    /// The found properties, keyed by the exact requested name.
    pub values: Map<String, Value>,
    /// The requested names that could not be resolved, in request order.
    pub missing: Vec<String>,
}

/// Arguments to the `read_frontmatter` tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadFrontmatterArgs {
    /// Path to the markdown file, absolute or relative to the working directory.
    pub path: String,
    /// Which representation to return.
    #[serde(default)]
    pub format: Format,
}

/// Arguments to the `read_frontmatter_batch` tool. Provide exactly one of
/// `paths` or `glob`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadFrontmatterBatchArgs {
    /// Explicit list of markdown file paths.
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    /// A glob pattern (`**` recurses) resolved against the working directory.
    #[serde(default)]
    pub glob: Option<String>,
    /// Which representation to return for every file.
    #[serde(default)]
    pub format: Format,
    /// Cap on how many files to read.
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

/// Arguments to the `get_frontmatter_properties` tool. Provide exactly one
/// of `paths` or `glob`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFrontmatterPropertiesArgs {
    /// Property names to extract; dotted names (`metadata.owner`) walk nested objects.
    pub properties: Vec<String>,
    /// Explicit list of markdown file paths.
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    /// A glob pattern (`**` recurses) resolved against the working directory.
    #[serde(default)]
    pub glob: Option<String>,
    /// Cap on how many files to read.
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

/// Why [`resolve_paths`] rejected its inputs. The message is what the tool
/// surfaces to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Neither, or both, of `paths` and `glob` were provided (an empty
    /// `paths` list and a whitespace-only `glob` both count as absent).
    NotExactlyOne,
    /// The glob pattern failed to compile; the string is wax's message.
    InvalidGlob(String),
}

impl ResolveError {
    /// The message to surface to the caller.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::NotExactlyOne => "Provide exactly one of 'paths' or 'glob'.".to_owned(),
            Self::InvalidGlob(message) => message.clone(),
        }
    }
}

/// Resolves the file list for a batch tool call from exactly one of an
/// explicit `paths` list or a `glob` pattern. `max_files` caps either
/// source. `glob_base` is the directory a relative glob resolves against.
///
/// # Errors
///
/// Returns [`ResolveError::NotExactlyOne`] unless exactly one of `paths`
/// (non-empty) or `glob` (non-whitespace) is given, or
/// [`ResolveError::InvalidGlob`] if the pattern doesn't compile.
pub fn resolve_paths(
    paths: Option<&[String]>,
    glob: Option<&str>,
    max_files: usize,
    glob_base: &Path,
) -> Result<Vec<String>, ResolveError> {
    let has_paths = paths.is_some_and(|list| !list.is_empty());
    let has_glob = glob.is_some_and(|pattern| !pattern.trim().is_empty());

    if has_paths == has_glob {
        return Err(ResolveError::NotExactlyOne);
    }

    if has_paths {
        Ok(paths
            .unwrap_or_default()
            .iter()
            .take(max_files)
            .cloned()
            .collect())
    } else {
        let pattern = glob.unwrap_or_default().trim();
        crate::glob::expand(pattern, glob_base, max_files).map_err(ResolveError::InvalidGlob)
    }
}

/// Builds a [`FrontmatterResult`] from an extraction, following the
/// format's rules for which of `raw`/`parsed` to include and how parse
/// failures fall back to returning `raw`.
#[must_use]
pub fn build_result(path: &str, extraction: &Extraction, format: Format) -> FrontmatterResult {
    if !extraction.has_frontmatter {
        return FrontmatterResult {
            path: path.to_owned(),
            has_frontmatter: false,
            raw: None,
            parsed: None,
            parse_error: None,
        };
    }

    let parse_result = (extraction.error.is_none() && format.wants_parsed())
        .then(|| yaml::parse(extraction.raw.as_deref().unwrap_or_default()));

    let parse_failed = format.wants_parsed() && parse_result.as_ref().is_none_or(Result::is_err);
    let include_raw = format.wants_raw() || parse_failed;

    let parsed = parse_result
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned();
    let parse_error = extraction
        .error
        .map(str::to_owned)
        .or_else(|| parse_result.and_then(Result::err));

    FrontmatterResult {
        path: path.to_owned(),
        has_frontmatter: true,
        raw: if include_raw {
            extraction.raw.clone()
        } else {
            None
        },
        parsed,
        parse_error,
    }
}

/// Reads one file's frontmatter. An I/O failure (e.g. the file doesn't
/// exist) becomes a structured result with `parseError` set rather than
/// an error return.
pub async fn read_one(path: &str, format: Format) -> FrontmatterResult {
    match frontmatter::extract(Path::new(path)).await {
        Ok(extraction) => build_result(path, &extraction, format),
        Err(error) => FrontmatterResult {
            path: path.to_owned(),
            has_frontmatter: false,
            raw: None,
            parsed: None,
            parse_error: Some(error.to_string()),
        },
    }
}

/// Projects `properties` out of one file's frontmatter. If the file has no
/// frontmatter, hit a reader error, or failed to parse, every requested
/// property is reported missing.
pub async fn project_one(path: &str, properties: &[String]) -> PropertyResult {
    let object = match frontmatter::extract(Path::new(path)).await {
        Ok(extraction) if extraction.has_frontmatter && extraction.error.is_none() => {
            match yaml::parse(extraction.raw.as_deref().unwrap_or_default()) {
                Ok(Value::Object(map)) => Some(map),
                _ => None,
            }
        }
        _ => None,
    };

    match object {
        Some(map) => {
            let (values, missing) = projection::project(&map, properties);
            PropertyResult {
                path: path.to_owned(),
                values,
                missing,
            }
        }
        None => PropertyResult {
            path: path.to_owned(),
            values: Map::new(),
            missing: properties.to_vec(),
        },
    }
}

/// Reads many files' frontmatter concurrently, preserving input order.
pub async fn read_batch(paths: &[String], format: Format) -> Vec<FrontmatterResult> {
    join_all(paths.iter().map(|path| read_one(path, format))).await
}

/// Projects `properties` out of many files' frontmatter concurrently,
/// preserving input order.
pub async fn project_batch(paths: &[String], properties: &[String]) -> Vec<PropertyResult> {
    join_all(paths.iter().map(|path| project_one(path, properties))).await
}
