use crate::spec::Severity;
use crate::walk::WalkOutput;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Allowed,
    Ignored,
    Unaccounted,
}

fn canon_key(s: &str) -> String {
    let mut t = s.trim().replace('\\', "/");

    // strip leading "./" and "/"
    t = t
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string();

    // strip trailing slashes
    while t.ends_with('/') && t.len() > 1 {
        t.pop();
    }

    // On case-insensitive file systems (like macOS), normalize to lowercase
    // so that paths that differ only in case are treated as the same.
    // This ensures that if a file is matched with one case, it's also
    // considered matched when queried with a different case.
    #[cfg(target_os = "macos")]
    {
        t = t.to_lowercase();
    }
    // On Windows, also normalize to lowercase for case-insensitive matching
    #[cfg(target_os = "windows")]
    {
        t = t.to_lowercase();
    }

    t
}

/// A diagnostic message about a path or rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Diagnostic code (e.g., "ambiguous_match", "reallowed_under_ignore")
    pub code: &'static str,
    /// Severity level of the diagnostic
    pub severity: Severity,
    /// Normalized relative path (using '/' as separator)
    pub path: String,
    /// Human-readable message describing the issue
    pub message: String,
    /// Line numbers in the `.fspec` file that are involved (if applicable)
    pub rule_lines: Vec<usize>,
}

/// A report containing the results of validating a directory tree against an `.fspec` file.
///
/// The report contains:
/// - Status information for each path (allowed, ignored, or unaccounted)
/// - Diagnostic messages about potential issues
#[derive(Debug, Default)]
pub struct Report {
    // Key: normalized relative path string ("src/main.rs", "bin", ...)
    statuses: BTreeMap<String, Status>,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn from_walk_output(walk: &WalkOutput) -> Report {
        let mut report = Report::default();

        for p in &walk.allowed_files {
            report.set_status(p.to_string_lossy().as_ref(), Status::Allowed);
        }
        for p in &walk.allowed_dirs {
            report.set_status(p.to_string_lossy().as_ref(), Status::Allowed);
        }
        for p in &walk.ignored_files {
            report.set_status(p.to_string_lossy().as_ref(), Status::Ignored);
        }
        for p in &walk.ignored_dirs {
            report.set_status(p.to_string_lossy().as_ref(), Status::Ignored);
        }
        for p in &walk.unaccounted_files {
            report.set_status(p.to_string_lossy().as_ref(), Status::Unaccounted);
        }
        for p in &walk.unaccounted_dirs {
            report.set_status(p.to_string_lossy().as_ref(), Status::Unaccounted);
        }

        report
    }
    pub fn set_status(&mut self, path: impl AsRef<str>, status: Status) {
        let k = canon_key(path.as_ref());
        self.statuses.insert(k, status);
    }

    pub fn status_of(&self, path: impl AsRef<str>) -> Option<Status> {
        let k = canon_key(path.as_ref());
        self.statuses.get(&k).copied()
    }

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
