//! Streaming extraction of the YAML frontmatter block from a markdown
//! file. The document body - everything after the closing `---` - is
//! never read.

use std::path::Path;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};

/// Reader error when the frontmatter block never closes before EOF.
pub const UNTERMINATED_ERROR: &str = "unterminated frontmatter block";

/// Reader error when the frontmatter block exceeds the maximum line count
/// without closing.
pub const TOO_LARGE_ERROR: &str = "frontmatter block exceeds maximum size";

/// The maximum number of lines a frontmatter block may contain before
/// extraction gives up and reports [`TOO_LARGE_ERROR`].
const MAX_FRONTMATTER_LINES: usize = 1000;

/// The result of scanning a markdown file for a `---`-delimited
/// frontmatter block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extraction {
    /// Whether the file's first line is exactly `---`.
    pub has_frontmatter: bool,
    /// The frontmatter text between the delimiters, LF-joined with no
    /// trailing newline. `None` when `has_frontmatter` is `false`.
    pub raw: Option<String>,
    /// Set when the block never closes ([`UNTERMINATED_ERROR`]) or grows
    /// past the maximum line count ([`TOO_LARGE_ERROR`]).
    pub error: Option<&'static str>,
}

/// Opens `path` and extracts its frontmatter block, if any. The document
/// body is never read from disk.
///
/// # Errors
///
/// Returns an error if `path` cannot be opened (e.g. it doesn't exist).
pub async fn extract(path: &Path) -> std::io::Result<Extraction> {
    let file = tokio::fs::File::open(path).await?;
    Ok(extract_from(BufReader::new(file)).await)
}

/// The core extraction algorithm, generic over any buffered async reader.
/// Exposed (rather than kept private) so the "the body is never read"
/// invariant can be tested against a reader that tracks how much of its
/// input was actually consumed.
pub async fn extract_from<R: AsyncBufRead + Unpin>(mut reader: R) -> Extraction {
    let mut first_line = String::new();
    let first_read = reader.read_line(&mut first_line).await;
    strip_line_ending(&mut first_line);

    if !matches!(first_read, Ok(n) if n > 0) || strip_bom(&first_line) != "---" {
        return Extraction {
            has_frontmatter: false,
            raw: None,
            error: None,
        };
    }

    let mut lines: Vec<String> = Vec::new();
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).await;

        let Ok(bytes_read) = read else {
            return unterminated(&lines);
        };
        if bytes_read == 0 {
            return unterminated(&lines);
        }

        strip_line_ending(&mut line);
        if line == "---" {
            return Extraction {
                has_frontmatter: true,
                raw: Some(lines.join("\n")),
                error: None,
            };
        }

        lines.push(line);
        if lines.len() >= MAX_FRONTMATTER_LINES {
            return Extraction {
                has_frontmatter: true,
                raw: Some(lines.join("\n")),
                error: Some(TOO_LARGE_ERROR),
            };
        }
    }
}

/// Builds the "never closed" result from whatever lines were accumulated
/// before EOF (or a read error) was reached.
fn unterminated(lines: &[String]) -> Extraction {
    Extraction {
        has_frontmatter: true,
        raw: Some(lines.join("\n")),
        error: Some(UNTERMINATED_ERROR),
    }
}

/// Removes a trailing `\n` and, if present, a preceding `\r` - normalizing
/// both LF and CRLF line endings to bare content.
fn strip_line_ending(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

/// Strips a leading UTF-8 byte-order-mark character, if present.
fn strip_bom(line: &str) -> &str {
    line.strip_prefix('\u{FEFF}').unwrap_or(line)
}
