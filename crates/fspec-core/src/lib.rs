mod error;
mod parse;
mod pattern;
mod report;
mod severity;
mod spec;

pub use error::Error;
pub use report::{Diagnostic, Report, Status};
pub use severity::Severity;
pub use spec::{Component, Pattern, Rule, RuleKind, Segment};

use std::path::Path;

pub fn check_tree(root: &Path, default_severity: Severity) -> Result<Report, Error> {
    // parse .fspec
    // walk filesystem
    // classify paths
    // produce diagnostics
    Err(Error::Semantic {
        msg: "Unimplemented error".into(),
    })
}
