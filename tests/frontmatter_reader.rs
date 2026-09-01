//! Behavioral tests for `frontmatter_mcp::frontmatter::extract`: the
//! streaming scan for a `---`-delimited frontmatter block that must never
//! read the document body.

mod common;

use common::fixture;
use frontmatter_mcp::frontmatter::{TOO_LARGE_ERROR, UNTERMINATED_ERROR, extract, extract_from};
use std::fmt::Write as _;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, BufReader, ReadBuf};

/// A flat mapping's frontmatter is extracted verbatim, LF-joined, with no
/// delimiters and no trailing newline.
#[tokio::test]
async fn flat_frontmatter_is_extracted() {
    let result = extract(&fixture("valid-simple.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(
        result.raw.as_deref(),
        Some("title: Sample Document\nstatus: draft")
    );
    assert!(result.error.is_none());
}

/// Nested mappings and block sequences are captured as raw text exactly as
/// written, delimiters excluded.
#[tokio::test]
async fn nested_frontmatter_is_extracted() {
    let result = extract(&fixture("valid-nested.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(
        result.raw.as_deref(),
        Some(
            "title: Sample Document\nmetadata:\n  owner: alice\n  tags:\n    - draft\n    - internal"
        )
    );
    assert!(result.error.is_none());
}

/// An immediately-closed block (`---\n---`) has frontmatter with an empty
/// (not missing) raw text.
#[tokio::test]
async fn empty_frontmatter_block_has_empty_raw() {
    let result = extract(&fixture("valid-empty-frontmatter.md"))
        .await
        .unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(result.raw.as_deref(), Some(""));
    assert!(result.error.is_none());
}

/// A document whose first line isn't exactly `---` has no frontmatter -
/// this is a normal result, not an error.
#[tokio::test]
async fn no_opening_fence_means_no_frontmatter() {
    let result = extract(&fixture("no-frontmatter.md")).await.unwrap();
    assert!(!result.has_frontmatter);
    assert!(result.raw.is_none());
    assert!(result.error.is_none());
}

/// A 0-byte file has no frontmatter.
#[tokio::test]
async fn empty_file_means_no_frontmatter() {
    let result = extract(&fixture("empty-file.md")).await.unwrap();
    assert!(!result.has_frontmatter);
    assert!(result.raw.is_none());
}

/// A `+++`-fenced (TOML-style) document doesn't match the YAML `---`
/// delimiter, so it has no frontmatter.
#[tokio::test]
async fn toml_style_fence_means_no_frontmatter() {
    let result = extract(&fixture("non-yaml-fence.md")).await.unwrap();
    assert!(!result.has_frontmatter);
    assert!(result.raw.is_none());
}

/// A block that never closes before EOF reports `UNTERMINATED_ERROR` and
/// captures every accumulated line, including ones that look like body
/// text.
#[tokio::test]
async fn unterminated_block_is_reported_with_captured_lines() {
    let result = extract(&fixture("unterminated.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(
        result.raw.as_deref(),
        Some("title: Unterminated\nstatus: draft\n\nThis block never closes.")
    );
    assert_eq!(result.error, Some(UNTERMINATED_ERROR));
}

/// The first `---` line after the opening fence closes the block, even
/// when the body later contains a code fence with `---` lines inside it -
/// those lines are never reached.
#[tokio::test]
async fn closes_at_first_fence_even_with_dashes_in_body() {
    let result = extract(&fixture("dashes-in-body.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(result.raw.as_deref(), Some("title: Has code fence in body"));
    assert!(result.error.is_none());
}

/// A UTF-8 byte-order-mark before the opening `---` is stripped, not
/// treated as part of the line.
#[tokio::test]
async fn bom_before_fence_is_stripped() {
    let result = extract(&fixture("bom-utf8.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(
        result.raw.as_deref(),
        Some("title: BOM Document\nstatus: draft")
    );
    assert!(result.error.is_none());
}

/// CRLF line endings are normalized to bare LF in the extracted raw text,
/// identically to the LF-only equivalent document.
#[tokio::test]
async fn crlf_line_endings_are_normalized() {
    let result = extract(&fixture("crlf-line-endings.md")).await.unwrap();
    assert!(result.has_frontmatter);
    assert_eq!(
        result.raw.as_deref(),
        Some("title: CRLF Document\nstatus: draft")
    );
    assert!(result.error.is_none());
}

/// Opening `extract` on a path that doesn't exist is an I/O error, not a
/// structured "no frontmatter" result.
#[tokio::test]
async fn missing_file_is_an_io_error() {
    let result = extract(&fixture("does-not-exist.md")).await;
    assert!(result.is_err());
}

/// A frontmatter block that accumulates 1000 lines without closing is
/// reported as too large, bounded to the first 1000 lines - it must not
/// keep reading indefinitely.
#[tokio::test]
async fn overlong_unterminated_block_is_bounded() {
    let mut content = String::from("---\n");
    for i in 0..50_000 {
        let _ = writeln!(content, "line{i}");
    }
    let result = extract_from(BufReader::new(Cursor::new(content.into_bytes()))).await;
    assert!(result.has_frontmatter);
    assert_eq!(result.error, Some(TOO_LARGE_ERROR));
    assert_eq!(result.raw.unwrap().lines().count(), 1000);
}

/// Wraps an in-memory reader and records how many bytes actually pass
/// through it, so a test can assert a bounded read deterministically
/// rather than by wall-clock timing.
struct CountingReader {
    /// The underlying in-memory data.
    inner: Cursor<Vec<u8>>,
    /// Running total of bytes delivered through `poll_read`.
    read: Arc<AtomicUsize>,
}

impl AsyncRead for CountingReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        if result.is_ready() {
            let delivered = buf.filled().len() - before;
            self.read.fetch_add(delivered, Ordering::SeqCst);
        }
        result
    }
}

/// A multi-megabyte document body, after a small frontmatter block, is
/// never buffered - extraction must stop reading at the closing fence.
#[tokio::test]
async fn huge_body_is_never_fully_read() {
    let mut content = String::from("---\ntitle: Small header\n---\n");
    content.push_str(&"x\n".repeat(5 * 1024 * 1024));

    let read = Arc::new(AtomicUsize::new(0));
    let reader = CountingReader {
        inner: Cursor::new(content.into_bytes()),
        read: Arc::clone(&read),
    };

    let result = extract_from(BufReader::new(reader)).await;

    assert!(result.has_frontmatter);
    assert_eq!(result.raw.as_deref(), Some("title: Small header"));
    assert!(result.error.is_none());

    let bytes_read = read.load(Ordering::SeqCst);
    assert!(
        bytes_read < 65536,
        "expected a bounded read, but {bytes_read} bytes were pulled from the body"
    );
}
