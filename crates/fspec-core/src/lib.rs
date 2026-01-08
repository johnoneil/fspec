mod compile;
mod error;
mod matcher;
mod parse;
mod pattern;
mod report;
mod spec;
mod walk;

use parse::parse_fspec;
use std::fs;
use std::path::{Path, PathBuf};

pub use error::Error;
pub use report::Report;
pub use spec::{DirType, FSEntry, FSPattern, FileType, MatchSettings, Rule, RuleKind, Severity};
pub use walk::{WalkCtx, WalkOutput};

/// Check a directory tree against an `.fspec` file located at `{root}/.fspec`.
///
/// This is a convenience function that calls `check_tree_with_spec` with `spec_path = None`.
///
/// # Errors
///
/// Returns an error if:
/// - The `.fspec` file is not found at `{root}/.fspec`
/// - The `.fspec` file cannot be read
/// - The `.fspec` file contains invalid syntax
/// - An I/O error occurs while walking the directory tree
pub fn check_tree(root: &Path, settings: &MatchSettings) -> Result<Report, Error> {
    check_tree_with_spec(root, None, settings)
}

/// Check a directory tree against an `.fspec` file.
///
/// If `spec_path` is `Some`, that path is used. Otherwise defaults to `{root}/.fspec`.
///
/// # Arguments
///
/// * `root` - The root directory to check
/// * `spec_path` - Optional path to the `.fspec` file. If `None`, looks for `.fspec` at the root.
/// * `settings` - Matching settings that control behavior (e.g., file vs directory matching)
///
/// # Returns
///
/// A `Report` containing the validation results, or an `Error` if something went wrong.
///
/// # Errors
///
/// Returns an error if:
/// - The `.fspec` file is not found
/// - The `.fspec` file cannot be read
/// - The `.fspec` file contains invalid syntax
/// - An I/O error occurs while walking the directory tree
pub fn check_tree_with_spec(
    root: &Path,
    spec_path: Option<&Path>,
    settings: &MatchSettings,
) -> Result<Report, Error> {
    // --- parse .fspec ---
    let fspec_path: PathBuf = match spec_path {
        Some(p) => p.to_path_buf(),
        None => root.join(".fspec"),
    };

    if !fspec_path.exists() {
        return Err(Error::Semantic {
            msg: format!(
                ".fspec file not found at {}\nHint: Create an .fspec file in the root directory or specify a different path with --spec",
                fspec_path.display()
            ),
        });
    }

    // Optional but usually helpful: fail early if it's not a file.
    if !fspec_path.is_file() {
        return Err(Error::Semantic {
            msg: format!(
                "Spec path is not a file: {}\nHint: The .fspec file must be a regular file, not a directory",
                fspec_path.display()
            ),
        });
    }

    let contents = fs::read_to_string(&fspec_path).map_err(|e| Error::Io {
        path: fspec_path.clone(),
        source: e,
    })?;

    let spec_rules = parse_fspec(&contents, settings)?;
    let walk_output = walk::walk_tree(root, &spec_rules)?;
    let report = Report::from_walk_output(&walk_output);

    Ok(report)
}
