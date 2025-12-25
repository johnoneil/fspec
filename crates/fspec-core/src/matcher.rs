use std::path::Path;

use crate::spec::{DirType, FSEntry, FSPattern, FileType, Rule, RuleKind};

pub fn matches_anchored_file(rule: &Rule, path: &Path) -> bool {
    // 1. Must be an allow rule
    if rule.kind != RuleKind::Allow {
        return false;
    }

    // 2. Must be an anchored pattern
    let parts = match &rule.pattern {
        FSPattern::Anchored(parts) => parts,
        _ => return false,
    };

    // 3. Split path into components
    let path_parts: Vec<std::borrow::Cow<'_, str>> =
        path.iter().map(|p| p.to_string_lossy()).collect();

    // 4. Must have same number of components
    if parts.len() != path_parts.len() {
        return false;
    }

    // 5. Match each component
    for (i, (pat, actual)) in parts.iter().zip(path_parts.iter()).enumerate() {
        let is_last = i == parts.len() - 1;

        match (pat, is_last) {
            // Directory components (must be literal dirs)
            (FSEntry::Dir(DirType::Lit(lit)), false) => {
                if lit != actual {
                    return false;
                }
            }

            // Final component must be a *file*
            (FSEntry::File(FileType::Lit(lit)), true) => {
                if lit != actual {
                    return false;
                }
            }

            // Anything else (wrong type, glob, etc.) — not supported yet
            _ => return false,
        }
    }

    true
}

pub fn matches_anchored_dir(rule: &Rule, path: &Path) -> bool {
    // 1. Must be an allow rule
    if rule.kind != RuleKind::Allow {
        return false;
    }

    // 2. Must be an anchored pattern
    let parts = match &rule.pattern {
        FSPattern::Anchored(parts) => parts,
        _ => return false,
    };

    // 3. Split path into components
    let path_parts: Vec<std::borrow::Cow<'_, str>> =
        path.iter().map(|p| p.to_string_lossy()).collect();

    // 4. Must have same number of components
    if parts.len() != path_parts.len() {
        return false;
    }

    // 5. Match each component
    for (i, (pat, actual)) in parts.iter().zip(path_parts.iter()).enumerate() {
        let is_last = i == parts.len() - 1;

        match (pat, is_last) {
            // Directory components (must be literal dirs)
            (FSEntry::Dir(DirType::Lit(lit)), false) => {
                if lit != actual {
                    return false;
                }
            }

            // Final component must be a *dir*
            (FSEntry::Dir(DirType::Lit(lit)), true) => {
                if lit != actual {
                    return false;
                }
            }

            // Anything else (wrong type, glob, etc.) — not supported yet
            _ => return false,
        }
    }

    true
}
