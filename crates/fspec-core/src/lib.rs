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

    // TODO: Verbose/debug mode
    //println!("{:#?}", contents);

    let spec_rules = parse_fspec(&contents, settings)?;

    // TODO: verbose/debug mode
    //println!("{:#?}", spec_rules);

    let walk_output = walk::walk_tree(root, &spec_rules)?;

    // TODO: verbose/debug mode
    //println!("{:#?}", walk_output);

    let report = Report::from_walk_output(&walk_output);

    Ok(report)
}
