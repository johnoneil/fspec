use std::path::Path;

use crate::spec::{DirType, FSEntry, FSPattern, FileType, Rule, RuleKind};

#[derive(Clone, Copy)]
enum Terminal {
    File,
    Dir,
}

fn matches_terminal_pat(terminal: Terminal, pat: &FSEntry, actual: &str) -> bool {
    match (terminal, pat) {
        (Terminal::File, FSEntry::File(FileType::Lit(lit))) => lit == actual,
        (Terminal::File, FSEntry::File(FileType::Star)) => true,

        (Terminal::Dir, FSEntry::Dir(DirType::Lit(lit))) => lit == actual,
        (Terminal::Dir, FSEntry::Dir(DirType::Star)) => true,
        // NOTE: DoubleStar is handled in DP; it is not a valid "single dir name" terminal.
        (Terminal::Dir, FSEntry::Dir(DirType::DoubleStar)) => false,

        _ => false,
    }
}

fn matches_dir_pat(pat: &FSEntry, actual: &str) -> bool {
    match pat {
        FSEntry::Dir(DirType::Lit(lit)) => lit == actual,
        FSEntry::Dir(DirType::Star) => true,
        // NOTE: DoubleStar is handled in the DP transitions, not as a single-step match.
        FSEntry::Dir(DirType::DoubleStar) => false,
        _ => false,
    }
}

fn dp(
    pi: usize,
    si: usize,
    parts: &[FSEntry],
    path_parts: &[std::borrow::Cow<'_, str>],
    terminal: Terminal,
    memo: &mut [Vec<Option<bool>>],
) -> bool {
    if let Some(v) = memo[pi][si] {
        return v;
    }

    let res = if pi == parts.len() {
        // Consumed all pattern entries: must also consume all path parts.
        si == path_parts.len()
    } else {
        let is_last = pi + 1 == parts.len();

        if !is_last {
            match &parts[pi] {
                // '**' in directory position: either consume zero segments OR consume one and stay on '**'
                FSEntry::Dir(DirType::DoubleStar) => {
                    // Option A: '**' matches zero segments -> advance pattern
                    let a = dp(pi + 1, si, parts, path_parts, terminal, memo);
                    // Option B: '**' matches one segment -> advance path, keep pattern index
                    let b = if si < path_parts.len() {
                        dp(pi, si + 1, parts, path_parts, terminal, memo)
                    } else {
                        false
                    };
                    a || b
                }
                // Normal directory entries must consume exactly one segment
                pat => {
                    if si >= path_parts.len() {
                        false
                    } else if matches_dir_pat(pat, &path_parts[si]) {
                        dp(pi + 1, si + 1, parts, path_parts, terminal, memo)
                    } else {
                        false
                    }
                }
            }
        } else {
            // Last pattern entry: must match the terminal form (file/dir) against exactly one segment
            if si >= path_parts.len() {
                false
            } else if matches_terminal_pat(terminal, &parts[pi], &path_parts[si]) {
                dp(pi + 1, si + 1, parts, path_parts, terminal, memo)
            } else {
                false
            }
        }
    };

    memo[pi][si] = Some(res);
    res
}

fn matches_anchored_literal(rule: &Rule, path: &Path, kind: RuleKind, terminal: Terminal) -> bool {
    if rule.kind != kind {
        return false;
    }

    let parts = match &rule.pattern {
        FSPattern::Anchored(parts) => parts,
        _ => return false,
    };

    // Materialize path components once (this avoids repeated iter().count() and allows DP).
    // Using Cow<str> keeps this cheap for normal UTF-8 paths.
    let path_parts: Vec<std::borrow::Cow<'_, str>> =
        path.iter().map(|c| c.to_string_lossy()).collect();

    // Memo table: None = unknown, Some(true/false) = computed.
    // Dimensions: (parts.len()+1) x (path_parts.len()+1)
    let mut memo: Vec<Vec<Option<bool>>> = vec![vec![None; path_parts.len() + 1]; parts.len() + 1];

    dp(0, 0, parts, &path_parts, terminal, &mut memo)
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

    if parts.is_empty() {
        return false;
    }

    // Materialize path components once (like in anchored).
    let path_parts: Vec<std::borrow::Cow<'_, str>> =
        path.iter().map(|c| c.to_string_lossy()).collect();

    // Unanchored semantics (your current code): align pattern to the *end* of the path.
    // Equivalent: anchored match on the full path with an implicit leading '**' that can
    // consume any prefix, but still requires the remainder to match the suffix exactly.
    //
    // I.e. unanchored(P) == anchored([**] + P)
    let mut effective_parts: Vec<FSEntry> = Vec::with_capacity(parts.len() + 1);

    // Optional micro-normalization: avoid ** ** at the front
    if !matches!(parts.first(), Some(FSEntry::Dir(DirType::DoubleStar))) {
        effective_parts.push(FSEntry::Dir(DirType::DoubleStar));
    }
    effective_parts.extend(parts.iter().cloned());

    // Fresh memo sized to effective_parts x path_parts.
    let mut memo: Vec<Vec<Option<bool>>> =
        vec![vec![None; path_parts.len() + 1]; effective_parts.len() + 1];

    dp(0, 0, &effective_parts, &path_parts, terminal, &mut memo)
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
