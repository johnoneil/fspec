use std::collections::BTreeMap;

use crate::Severity;

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

#[derive(Debug, Default)]
pub struct Report {
    // Key: normalized relative path string ("src/main.rs", "bin", ...)
    statuses: BTreeMap<String, Status>,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    // pub fn new() -> Self {
    //     Self {
    //         statuses: BTreeMap::new(),
    //         diagnostics: Vec::new(),
    //     }
    // }

    pub fn set_status(&mut self, path: impl Into<String>, status: Status) {
        self.statuses.insert(path.into(), status);
    }

    pub fn push_diagnostic(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

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
