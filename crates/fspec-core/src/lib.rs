mod error;
mod matcher;
mod parse;
mod pattern;
mod report;
mod severity;
mod spec;
mod walk;

use parse::parse_fspec;
use report::Report;
use std::fs;
use std::path::{Path, PathBuf};

pub use error::Error;
pub use severity::Severity;
pub use spec::{DirType, FSEntry, FSPattern, FileType, Rule, RuleKind};
pub use walk::{WalkCtx, WalkOutput};

pub fn check_tree(root: &Path, _default_severity: Severity) -> Result<Report, Error> {
    // --- parse .fspec ---
    let fspec_path: PathBuf = root.join(".fspec");

    if !fspec_path.exists() {
        return Err(Error::Semantic {
            msg: format!(".fspec not found at {}", fspec_path.display()),
        });
    }

    let contents = fs::read_to_string(&fspec_path).map_err(|e| Error::Io {
        path: fspec_path.clone(),
        source: e,
    })?;

    let spec_rules = parse_fspec(&contents)?;

    let walk_output = walk::walk_tree(root, &spec_rules)?;

    let report = Report::from_walk_output(&walk_output);

    Ok(report)
}
