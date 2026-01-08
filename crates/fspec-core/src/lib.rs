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

pub fn check_tree(root: &Path, settings: &MatchSettings) -> Result<Report, Error> {
    check_tree_with_spec(root, None, settings)
}

/// If `spec_path` is `Some`, that path is used. Otherwise defaults to `{root}/.fspec`.
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
            msg: format!(".fspec not found at {}", fspec_path.display()),
        });
    }

    // Optional but usually helpful: fail early if it's not a file.
    if !fspec_path.is_file() {
        return Err(Error::Semantic {
            msg: format!("spec path is not a file: {}", fspec_path.display()),
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
