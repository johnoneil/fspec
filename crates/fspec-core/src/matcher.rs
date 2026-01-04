use std::collections::HashMap;
use std::path::Path;

use crate::spec::{DirType, FSEntry, FSPattern, FileType, Rule, RuleKind};
use fspec_placeholder::ast::{Choice, ComponentAst, LimiterArg, Part, PlaceholderNode};
use regex::Regex;

/// Result of matching a component AST against a string
#[derive(Debug)]
struct ComponentMatch {
    /// Whether the component matched
    matched: bool,
    /// Extracted placeholder values (name -> value)
    placeholders: HashMap<String, String>,
}

fn matches_component_ast(ast: &ComponentAst, actual: &str) -> bool {
    extract_component_values(ast, actual).matched
}

fn extract_component_values(ast: &ComponentAst, actual: &str) -> ComponentMatch {
    let mut pat = String::from("^");
    let mut placeholder_indices: Vec<(String, usize)> = Vec::new(); // name -> capture group index
    let mut capture_group = 1;

    for part in &ast.parts {
        match part {
            Part::Literal(lit) => pat.push_str(&regex::escape(&lit.value)),
            Part::Star(_) => pat.push_str(".*"),
            Part::Placeholder(ph) => match &ph.node {
                PlaceholderNode::OneOf(oneof) => {
                    // Named one-of: extract the matched choice
                    if let Some(named) = &oneof.name {
                        let mut alts: Vec<String> = Vec::new();
                        for choice in &oneof.choices {
                            let s = match choice {
                                Choice::Ident { value, .. } => value,
                                Choice::Str { value, .. } => value,
                            };
                            alts.push(regex::escape(s));
                        }
                        pat.push_str("("); // capture group for named one-of
                        pat.push_str(&alts.join("|"));
                        pat.push(')');
                        placeholder_indices.push((named.name.clone(), capture_group));
                        capture_group += 1;
                    } else {
                        // Unnamed one-of: no capture
                        let mut alts: Vec<String> = Vec::new();
                        for choice in &oneof.choices {
                            let s = match choice {
                                Choice::Ident { value, .. } => value,
                                Choice::Str { value, .. } => value,
                            };
                            alts.push(regex::escape(s));
                        }
                        pat.push_str("(?:");
                        pat.push_str(&alts.join("|"));
                        pat.push(')');
                    }
                }
                PlaceholderNode::Capture(cap) => {
                    // Capture with name: extract the matched value
                    let mut cap_re = String::from(".+");

                    if let Some(lim) = &cap.limiter {
                        cap_re = limiter_to_regex(lim.name.as_str(), &lim.args);
                    }

                    pat.push_str("("); // capture group for named capture
                    pat.push_str(&cap_re);
                    pat.push(')');
                    placeholder_indices.push((cap.name.clone(), capture_group));
                    capture_group += 1;
                }
            },
        }
    }

    pat.push('$');

    // If limiter_to_regex yields invalid regex (e.g. user gave a bad re("...")),
    // treat as non-match rather than panic.
    let re = match Regex::new(&pat) {
        Ok(r) => r,
        Err(_) => {
            return ComponentMatch {
                matched: false,
                placeholders: HashMap::new(),
            };
        }
    };

    if let Some(caps) = re.captures(actual) {
        let mut placeholders = HashMap::new();
        for (name, idx) in placeholder_indices {
            if let Some(mat) = caps.get(idx) {
                placeholders.insert(name, mat.as_str().to_string());
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

fn limiter_to_regex(name: &str, args: &[LimiterArg]) -> String {
    match name {
        // ASCII case/style
        "snake_case" => r"[a-z0-9]+(?:_[a-z0-9]+)*".to_string(),
        "kebab_case" => r"[a-z0-9]+(?:-[a-z0-9]+)*".to_string(),
        "pascal_case" => r"[A-Z][a-z0-9]*(?:[A-Z][a-z0-9]*)*".to_string(),
        "upper_case" => r"[A-Z0-9]+".to_string(),
        "lower_case" => r"[a-z0-9]+".to_string(),

        // int(n): exactly n digits
        "int" => {
            if let Some(LimiterArg::Number { value, .. }) = args.get(0) {
                if let Ok(n) = value.parse::<usize>() {
                    return format!(r"[0-9]{{{}}}", n);
                }
            }
            // fallback: invalid args -> non-empty
            ".+".to_string()
        }

        // re("..."): user regex
        "re" => {
            if let Some(LimiterArg::Str { value, .. }) = args.get(0) {
                return format!(r"(?:{})", value);
            }
            ".+".to_string()
        }

        // Unicode-ish buckets (regex crate supports \p{..})
        "letters" => r"\p{L}+".to_string(),
        "numbers" => r"\p{Nd}+".to_string(),
        "alnum" => r"(?:\p{L}|\p{Nd})+".to_string(),

        // Unknown limiter for now: accept non-empty
        _ => ".+".to_string(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Terminal {
    File,
    Dir,
}

fn matches_terminal_pat(terminal: Terminal, pat: &FSEntry, actual: &str) -> bool {
    match (terminal, pat) {
        (Terminal::File, FSEntry::File(FileType::Component(component))) => {
            matches_component_ast(component, actual)
        }
        (Terminal::File, FSEntry::File(FileType::Star)) => true,

        (Terminal::Dir, FSEntry::Dir(DirType::Component(component))) => {
            matches_component_ast(component, actual)
        }
        (Terminal::Dir, FSEntry::Dir(DirType::Star)) => true,
        (Terminal::Dir, FSEntry::Dir(DirType::DoubleStar)) => false,

        _ => false,
    }
}

fn matches_dir_pat(pat: &FSEntry, actual: &str) -> bool {
    match pat {
        FSEntry::Dir(DirType::Component(component)) => matches_component_ast(component, actual),
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
                        placeholders
                            .entry(name)
                            .or_insert_with(Vec::new)
                            .push(value);
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
                        placeholders
                            .entry(name)
                            .or_insert_with(Vec::new)
                            .push(value);
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
