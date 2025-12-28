use std::path::Path;

use crate::spec::{DirType, FSEntry, FSPattern, FileType, Rule, RuleKind};

#[derive(Clone, Copy)]
enum Terminal {
    File,
    Dir,
}

fn matches_anchored_literal(rule: &Rule, path: &Path, kind: RuleKind, terminal: Terminal) -> bool {
    if rule.kind != kind {
        return false;
    }

    let parts = match &rule.pattern {
        FSPattern::Anchored(parts) => parts,
        _ => return false,
    };

    // Count path components without allocating.
    if parts.len() != path.iter().count() {
        return false;
    }

    for (i, (pat, actual_os)) in parts.iter().zip(path.iter()).enumerate() {
        let is_last = i + 1 == parts.len();
        let actual = actual_os.to_string_lossy();

        if !is_last {
            match pat {
                FSEntry::Dir(DirType::Lit(lit)) if lit == &actual => {}
                FSEntry::Dir(DirType::Star) => {} // matches any single directory component
                _ => return false,
            }
        } else {
            match (terminal, pat) {
                (Terminal::File, FSEntry::File(FileType::Lit(lit))) if lit == &actual => {}
                (Terminal::File, FSEntry::File(FileType::Star)) => {} // matches any single file name

                (Terminal::Dir, FSEntry::Dir(DirType::Lit(lit))) if lit == &actual => {}
                (Terminal::Dir, FSEntry::Dir(DirType::Star)) => {} // matches any single dir name
                _ => return false,
            }
        }
    }

    true
}

fn matches_unanchored_literal(
    rule: &Rule,
    path: &Path,
    kind: RuleKind,
    terminal: Terminal,
) -> bool {
    if rule.kind != kind {
        return false;
    }

    let parts = match &rule.pattern {
        FSPattern::Unanchored(parts) => parts,
        _ => return false,
    };

    let comps: Vec<String> = path
        .iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect();
    let path_len = comps.len();
    let pat_len = parts.len();

    if pat_len == 0 || pat_len > path_len {
        return false;
    }

    // 👇 key: align pattern to the *end* of the path
    let start = path_len - pat_len;

    for (j, pat) in parts.iter().enumerate() {
        let is_last = j + 1 == pat_len;
        let actual = &comps[start + j];

        if !is_last {
            match pat {
                FSEntry::Dir(DirType::Lit(lit)) if lit == actual => {}
                FSEntry::Dir(DirType::Star) => {} // matches any single directory component
                _ => return false,
            }
        } else {
            match (terminal, pat) {
                (Terminal::File, FSEntry::File(FileType::Lit(lit))) if lit == actual => {}
                (Terminal::File, FSEntry::File(FileType::Star)) => {} // matches any single file name

                (Terminal::Dir, FSEntry::Dir(DirType::Lit(lit))) if lit == actual => {}
                (Terminal::Dir, FSEntry::Dir(DirType::Star)) => {} // matches any single dir name
                _ => return false,
            }
        }
    }

    true
}

pub(crate) fn matches_allowed_anchored_file(rule: &Rule, path: &Path) -> bool {
    matches_anchored_literal(rule, path, RuleKind::Allow, Terminal::File)
}
pub(crate) fn matches_allowed_anchored_dir(rule: &Rule, path: &Path) -> bool {
    matches_anchored_literal(rule, path, RuleKind::Allow, Terminal::Dir)
}
pub(crate) fn matches_ignored_anchored_file(rule: &Rule, path: &Path) -> bool {
    matches_anchored_literal(rule, path, RuleKind::Ignore, Terminal::File)
}
pub(crate) fn matches_ignored_anchored_dir(rule: &Rule, path: &Path) -> bool {
    matches_anchored_literal(rule, path, RuleKind::Ignore, Terminal::Dir)
}
pub(crate) fn matches_allowed_unanchored_file(rule: &Rule, path: &Path) -> bool {
    matches_unanchored_literal(rule, path, RuleKind::Allow, Terminal::File)
}
pub(crate) fn matches_allowed_unanchored_dir(rule: &Rule, path: &Path) -> bool {
    matches_unanchored_literal(rule, path, RuleKind::Allow, Terminal::Dir)
}
pub(crate) fn matches_ignored_unanchored_file(rule: &Rule, path: &Path) -> bool {
    matches_unanchored_literal(rule, path, RuleKind::Ignore, Terminal::File)
}
pub(crate) fn matches_ignored_unanchored_dir(rule: &Rule, path: &Path) -> bool {
    matches_unanchored_literal(rule, path, RuleKind::Ignore, Terminal::Dir)
}
