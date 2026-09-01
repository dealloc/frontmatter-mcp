//! Shared helpers for integration tests. Not a test binary itself - each
//! test file that needs it declares `mod common;` (or `#[path = ...]`) to
//! pull it in, per Cargo's `tests/<name>/mod.rs` convention.

use std::path::{Path, PathBuf};

/// The absolute path to a fixture file, given its name relative to
/// `tests/fixtures/` (e.g. `"valid-simple.md"` or
/// `"glob-sample/nested/c.md"`).
pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
