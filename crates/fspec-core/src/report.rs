use std::collections::BTreeMap;

use crate::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Allowed,
    Ignored,
    Unaccounted,
}

fn canon_key(s: &str) -> String {
    let mut t = s.trim().replace('\\', "/");

    // strip leading "./"
    while t.starts_with("./") {
        t = t[2..].to_string();
    }

    // strip trailing slashes
    while t.ends_with('/') && t.len() > 1 {
        t.pop();
    }

    t
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
    pub fn set_status(&mut self, path: impl AsRef<str>, status: Status) {
        let k = canon_key(path.as_ref());
        self.statuses.insert(k, status);
    }

    pub fn status_of(&self, path: impl AsRef<str>) -> Option<Status> {
        let k = canon_key(path.as_ref());
        self.statuses.get(&k).copied()
    }

    // pub fn is_allowed(&self, path: impl AsRef<str>) -> bool {
    //     self.status_of(path).is_some_and(|s| s == Status::Allowed)
    // }

    // pub fn set_status(&mut self, path: impl Into<String>, status: Status) {
    //     self.statuses.insert(path.into(), status);
    // }

    pub fn push_diagnostic(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    pub fn is_allowed(&self, path: &str) -> bool {
        let k = canon_key(path);
        self.statuses.get(&k).is_some_and(|s| *s == Status::Allowed)
    }
    pub fn is_ignored(&self, path: &str) -> bool {
        let k = canon_key(path);
        self.statuses.get(&k).is_some_and(|s| *s == Status::Ignored)
    }
    pub fn is_unaccounted(&self, path: &str) -> bool {
        let k = canon_key(path);
        self.statuses
            .get(&k)
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
