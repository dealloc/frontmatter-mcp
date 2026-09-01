//! Expands a glob pattern to a sorted, de-duplicated list of matching
//! file paths. `*` matches within a single directory level; `**` is
//! required to recurse.

use std::path::Path;
use wax::Glob;
use wax::walk::Entry as _;

/// Expands `pattern` against `base_dir`, returning matching **file** paths
/// (directories are excluded) as absolute path strings, sorted by byte
/// order and truncated to `max_files`.
///
/// # Errors
///
/// Returns the build error's message if `pattern` is not a valid glob.
pub fn expand(pattern: &str, base_dir: &Path, max_files: usize) -> Result<Vec<String>, String> {
    let glob = Glob::new(pattern).map_err(|error| error.to_string())?;

    let mut paths: Vec<String> = glob
        .walk(base_dir)
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path().to_string_lossy().into_owned())
        .collect();

    paths.sort_unstable();
    paths.dedup();
    paths.truncate(max_files);
    Ok(paths)
}
