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
                _ => return false,
            }
        } else {
            match (terminal, pat) {
                (Terminal::File, FSEntry::File(FileType::Lit(lit))) if lit == &actual => {}
                (Terminal::Dir, FSEntry::Dir(DirType::Lit(lit))) if lit == &actual => {}
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
