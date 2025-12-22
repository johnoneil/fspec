mod error;
mod parse;
mod pattern;
mod report;
mod severity;
mod spec;

use parse::parse_fspec;
use std::fs;
use std::path::{Path, PathBuf};

pub use error::Error;
pub use report::{Diagnostic, Report, Status};
pub use severity::Severity;
pub use spec::{Component, Pattern, Rule, RuleKind, Segment};

pub fn check_tree(root: &Path, default_severity: Severity) -> Result<Report, Error> {
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

    let spec = parse_fspec(&contents)?;

    // DEBUG: remove later.
    eprintln!("{:#?}", spec);

    // later:
    // - walk filesystem
    // - classify paths
    // - produce diagnostics

    // TEMPORARY: return early until next stages exist
    Err(Error::Semantic {
        msg: "Unimplemented error".into(),
    })
}
