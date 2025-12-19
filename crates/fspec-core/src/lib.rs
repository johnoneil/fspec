use std::collections::BTreeMap;

use std::path::Path;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse {
        line: usize,
        col: usize,
        msg: String,
    },
    Semantic {
        msg: String,
    },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub fn check_tree(root: &Path, default_severity: Severity) -> Result<Report, Error> {
    // parse .fspec
    // walk filesystem
    // classify paths
    // produce diagnostics
    Err(Error::Semantic {
        msg: "Unimplemented error".into(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Allowed,
    Ignored,
    Unaccounted,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: &'static str,     // e.g. "ambiguous_match", "reallowed_under_ignore"
    pub severity: Severity,     // usually Warning
    pub path: String,           // normalized relative path with '/'
    pub message: String,        // human-readable
    pub rule_lines: Vec<usize>, // optional: lines involved
}

#[derive(Debug)]
pub struct Report {
    // Key: normalized relative path string ("src/main.rs", "bin", ...)
    statuses: BTreeMap<String, Status>,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn is_allowed(&self, path: &str) -> bool {
        self.statuses
            .get(path)
            .is_some_and(|s| *s == Status::Allowed)
    }
    pub fn is_ignored(&self, path: &str) -> bool {
        self.statuses
            .get(path)
            .is_some_and(|s| *s == Status::Ignored)
    }
    pub fn is_unaccounted(&self, path: &str) -> bool {
        self.statuses
            .get(path)
            .is_some_and(|s| *s == Status::Unaccounted)
    }

    pub fn unaccounted_paths(&self) -> Vec<&str> {
        self.statuses
            .iter()
            .filter_map(|(p, s)| (*s == Status::Unaccounted).then_some(p.as_str()))
            .collect()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
