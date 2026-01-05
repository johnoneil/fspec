use std::collections::HashMap;
use std::path::Path;

use crate::spec::{
    CompiledComponent, DirType, FSEntry, FSPattern, FileOrDirType, FileType, Rule, RuleKind,
};

/// Result of matching a compiled component against a string
#[derive(Debug)]
struct ComponentMatch {
    /// Whether the component matched
    matched: bool,
    /// Extracted placeholder values (name -> value)
    placeholders: HashMap<String, String>,
}

fn matches_compiled_component(compiled: &CompiledComponent, actual: &str) -> bool {
    extract_component_values(compiled, actual).matched
}

/// Extract placeholder values from a matched component using the pre-compiled regex.
fn extract_component_values(compiled: &CompiledComponent, actual: &str) -> ComponentMatch {
    // Use the pre-compiled regex directly - no compilation needed!
    if let Some(caps) = compiled.regex.captures(actual) {
        let mut placeholders = HashMap::new();
        for (name, idx) in &compiled.placeholder_indices {
            if let Some(mat) = caps.get(*idx) {
                placeholders.insert(name.clone(), mat.as_str().to_string());
            }
        }
        ComponentMatch {
            matched: true,
            placeholders,
        }
    } else {
        ComponentMatch {
            matched: false,
            placeholders: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Terminal {
    File,
    Dir,
}

fn matches_terminal_pat(terminal: Terminal, pat: &FSEntry, actual: &str) -> bool {
    match (terminal, pat) {
        // FILE
        (Terminal::File, FSEntry::File(FileType::Component(c)))
        | (Terminal::File, FSEntry::Either(FileOrDirType::Component(c))) => {
            matches_compiled_component(c, actual)
        }
        (Terminal::File, FSEntry::File(FileType::Star))
        | (Terminal::File, FSEntry::Either(FileOrDirType::Star)) => true,

        // DIR
        (Terminal::Dir, FSEntry::Dir(DirType::Component(c)))
        | (Terminal::Dir, FSEntry::Either(FileOrDirType::Component(c))) => {
            matches_compiled_component(c, actual)
        }
        (Terminal::Dir, FSEntry::Dir(DirType::Star))
        | (Terminal::Dir, FSEntry::Either(FileOrDirType::Star)) => true,

        // Explicitly forbidden
        (Terminal::Dir, FSEntry::Dir(DirType::DoubleStar)) => false,

        _ => false,
    }
}

fn matches_dir_pat(pat: &FSEntry, actual: &str) -> bool {
    match pat {
        FSEntry::Dir(DirType::Component(component)) => {
            matches_compiled_component(component, actual)
        }
        FSEntry::Dir(DirType::Star) => true,
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

    // Phase 1: Check if pattern matches (existing logic)
    if !dp(0, 0, parts, &path_parts, terminal, &mut memo) {
        return false;
    }

    // Phase 2: Extract placeholder values and validate same-name constraints
    validate_placeholder_consistency(parts, &path_parts, terminal)
}

/// Extract placeholder values from a matched pattern and validate same-name consistency
fn validate_placeholder_consistency(
    parts: &[FSEntry],
    path_parts: &[std::borrow::Cow<'_, str>],
    terminal: Terminal,
) -> bool {
    // Extract all placeholder values by matching components
    let mut all_placeholders: HashMap<String, Vec<String>> = HashMap::new(); // name -> list of values

    extract_placeholders_recursive(parts, path_parts, 0, 0, terminal, &mut all_placeholders);

    // Validate: all placeholders with the same name must have the same value
    for (_name, values) in all_placeholders {
        if values.len() > 1 {
            // All values must be the same
            let first = &values[0];
            for value in values.iter().skip(1) {
                if value != first {
                    return false;
                }
            }
        }
    }

    true
}

/// Recursively extract placeholder values by matching pattern parts to path segments
fn extract_placeholders_recursive(
    parts: &[FSEntry],
    path_parts: &[std::borrow::Cow<'_, str>],
    pat_idx: usize,
    path_idx: usize,
    terminal: Terminal,
    placeholders: &mut HashMap<String, Vec<String>>,
) -> bool {
    if pat_idx >= parts.len() {
        return path_idx >= path_parts.len();
    }

    if path_idx >= path_parts.len() {
        return false;
    }

    let is_last = pat_idx + 1 == parts.len();

    match &parts[pat_idx] {
        FSEntry::Dir(DirType::DoubleStar) => {
            if is_last {
                // ** at end - doesn't match anything in anchored mode
                return false;
            }
            // ** can consume zero or more segments
            // Try zero segments first
            if extract_placeholders_recursive(
                parts,
                path_parts,
                pat_idx + 1,
                path_idx,
                terminal,
                placeholders,
            ) {
                return true;
            }
            // Try consuming one segment
            if extract_placeholders_recursive(
                parts,
                path_parts,
                pat_idx,
                path_idx + 1,
                terminal,
                placeholders,
            ) {
                return true;
            }
            false
        }
        FSEntry::Dir(DirType::Component(component)) => {
            if !is_last {
                let match_result = extract_component_values(component, &path_parts[path_idx]);
                if match_result.matched {
                    for (name, value) in match_result.placeholders {
                        placeholders.entry(name).or_default().push(value);
                    }
                    extract_placeholders_recursive(
                        parts,
                        path_parts,
                        pat_idx + 1,
                        path_idx + 1,
                        terminal,
                        placeholders,
                    )
                } else {
                    false
                }
            } else {
                false // Directory component can't be last
            }
        }
        FSEntry::Dir(DirType::Star) => {
            if !is_last {
                extract_placeholders_recursive(
                    parts,
                    path_parts,
                    pat_idx + 1,
                    path_idx + 1,
                    terminal,
                    placeholders,
                )
            } else {
                false
            }
        }
        FSEntry::File(FileType::Component(component)) => {
            if is_last && terminal == Terminal::File {
                let match_result = extract_component_values(component, &path_parts[path_idx]);
                if match_result.matched {
                    for (name, value) in match_result.placeholders {
                        placeholders.entry(name).or_default().push(value);
                    }
                    path_idx + 1 == path_parts.len()
                } else {
                    false
                }
            } else {
                false
            }
        }
        FSEntry::File(FileType::Star) => {
            if is_last && terminal == Terminal::File {
                path_idx + 1 == path_parts.len()
            } else {
                false
            }
        }
        FSEntry::Either(either) => {
            match terminal {
                Terminal::File => {
                    // Treat as File
                    match either {
                        FileOrDirType::Component(component) => {
                            if is_last {
                                let match_result =
                                    extract_component_values(component, &path_parts[path_idx]);
                                if match_result.matched {
                                    for (name, value) in match_result.placeholders {
                                        placeholders.entry(name).or_default().push(value);
                                    }
                                    path_idx + 1 == path_parts.len()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        FileOrDirType::Star => is_last && path_idx + 1 == path_parts.len(),
                    }
                }
                Terminal::Dir => {
                    // Treat as Dir
                    match either {
                        FileOrDirType::Component(component) => {
                            if !is_last {
                                let match_result =
                                    extract_component_values(component, &path_parts[path_idx]);
                                if match_result.matched {
                                    for (name, value) in match_result.placeholders {
                                        placeholders.entry(name).or_default().push(value);
                                    }
                                    extract_placeholders_recursive(
                                        parts,
                                        path_parts,
                                        pat_idx + 1,
                                        path_idx + 1,
                                        terminal,
                                        placeholders,
                                    )
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        FileOrDirType::Star => {
                            !is_last
                                && extract_placeholders_recursive(
                                    parts,
                                    path_parts,
                                    pat_idx + 1,
                                    path_idx + 1,
                                    terminal,
                                    placeholders,
                                )
                        }
                    }
                }
            }
        }
    }
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

    // Phase 1: Check if pattern matches
    if !dp(0, 0, &effective_parts, &path_parts, terminal, &mut memo) {
        return false;
    }

    // Phase 2: Extract placeholder values and validate same-name constraints
    validate_placeholder_consistency(&effective_parts, &path_parts, terminal)
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
