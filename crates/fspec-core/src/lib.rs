mod error;
mod parse;
mod pattern;
mod report;
mod severity;
mod spec;

use parse::parse_fspec;
use std::fs;
use std::path::{Path, PathBuf};

pub use error::Error;
pub use report::{Diagnostic, Report, Status};
pub use severity::Severity;
pub use spec::{Component, Pattern, Rule, RuleKind, Segment};

#[derive(Debug, Default)]
pub struct ClassifiedPaths {
    pub allowed_files: Vec<PathBuf>,
    pub ignored_files: Vec<PathBuf>,
    pub unclassified_files: Vec<PathBuf>,

    pub allowed_dirs: Vec<PathBuf>,
    pub ignored_dirs: Vec<PathBuf>,
    pub unclassified_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Ignore,
    Allow,
    Unclassified,
}

/// Walk the filesystem rooted at `root`, classifying each entry by the first matching rule.
/// Directories that are classified as Ignore are not descended into.
pub fn walk_and_classify(root: &Path, rules: &[Rule]) -> Result<ClassifiedPaths, Error> {
    let mut out = ClassifiedPaths::default();
    walk_dir(root, root, rules, &mut out)?;
    Ok(out)
}

fn walk_dir(
    root: &Path,
    dir: &Path,
    rules: &[Rule],
    out: &mut ClassifiedPaths,
) -> Result<(), Error> {
    let rd = fs::read_dir(dir).map_err(|e| Error::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for ent in rd {
        let ent = ent.map_err(|e| Error::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = ent.path();
        let md = ent.metadata().map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;

        // Compute rel path for matching.
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue, // should not happen
        };

        // Skip the .fspec file itself if you want.
        // if rel == Path::new(".fspec") { continue; }

        let is_dir = md.is_dir();
        let decision = decide(rel, is_dir, rules);

        if is_dir {
            match decision {
                Decision::Ignore => out.ignored_dirs.push(rel.to_path_buf()),
                Decision::Allow => out.allowed_dirs.push(rel.to_path_buf()),
                Decision::Unclassified => out.unclassified_dirs.push(rel.to_path_buf()),
            }

            // IMPORTANT: ignore dirs prune traversal
            if decision != Decision::Ignore {
                walk_dir(root, &path, rules, out)?;
            }
        } else {
            match decision {
                Decision::Ignore => out.ignored_files.push(rel.to_path_buf()),
                Decision::Allow => out.allowed_files.push(rel.to_path_buf()),
                Decision::Unclassified => out.unclassified_files.push(rel.to_path_buf()),
            }
        }
    }

    Ok(())
}

fn decide(rel: &Path, is_dir: bool, rules: &[Rule]) -> Decision {
    for rule in rules {
        if pattern_matches_path(&rule.pattern, rel, is_dir) {
            return match rule.kind {
                RuleKind::Ignore => Decision::Ignore,
                RuleKind::Allow => Decision::Allow,
            };
        }
    }
    Decision::Unclassified
}

/// Convert a relative path into string components for matching.
/// NOTE: This treats non-UTF8 components as non-matchable (you can adjust policy).
fn rel_components(rel: &Path) -> Option<Vec<String>> {
    let mut v = Vec::new();
    for c in rel.components() {
        let s = c.as_os_str().to_str()?.to_string();
        v.push(s);
    }
    Some(v)
}

fn pattern_matches_path(pat: &Pattern, rel: &Path, is_dir: bool) -> bool {
    let path = match rel_components(rel) {
        Some(v) => v,
        None => return false,
    };

    match pat {
        Pattern::Anchored(comps) => matches_components(comps, &path, is_dir, /*start*/ 0),
        Pattern::Unanchored(comps) => {
            // Try matching starting at any directory boundary.
            for start in 0..=path.len() {
                if matches_components(comps, &path, is_dir, start) {
                    return true;
                }
            }
            false
        }
    }
}

fn matches_components(comps: &[Component], path: &[String], is_dir: bool, start: usize) -> bool {
    // Quick “type” gate: pattern ending in Dir should only match dirs;
    // pattern ending in Entry should only match non-dirs.
    if let Some(last) = comps.last() {
        match last {
            Component::Dir(_) if !is_dir => return false,
            Component::Entry(_) if is_dir => return false,
            _ => {}
        }
    }

    // Extract segments from components (Dir/Entry both consume one path element)
    let segs: Vec<&Segment> = comps
        .iter()
        .map(|c| match c {
            Component::Dir(s) => s,
            Component::Entry(s) => s,
        })
        .collect();

    glob_match_segs(&segs, path, start)
}

/// Match pattern segments against path components with `*` and `**`.
/// - Lit must equal component exactly
/// - Star matches exactly one component
/// - DoubleStar matches zero or more components
fn glob_match_segs(segs: &[&Segment], path: &[String], start: usize) -> bool {
    fn rec(segs: &[&Segment], path: &[String], i: usize, j: usize) -> bool {
        if i == segs.len() {
            return j == path.len();
        }

        match segs[i] {
            Segment::Lit(lit) => {
                if j < path.len() && path[j] == *lit {
                    rec(segs, path, i + 1, j + 1)
                } else {
                    false
                }
            }
            Segment::Star => {
                if j < path.len() {
                    rec(segs, path, i + 1, j + 1)
                } else {
                    false
                }
            }
            Segment::DoubleStar => {
                // ** matches zero or more components
                // try “consume none”
                if rec(segs, path, i + 1, j) {
                    return true;
                }
                // or consume one and stay on **
                if j < path.len() && rec(segs, path, i, j + 1) {
                    return true;
                }
                false
            }
        }
    }

    // start matching at `start` within the path; allow remaining prefix before start.
    if start > path.len() {
        return false;
    }
    rec(segs, path, 0, start)
}

pub fn check_tree(root: &Path, default_severity: Severity) -> Result<Report, Error> {
    // --- parse .fspec ---
    let fspec_path: PathBuf = root.join(".fspec");

    if !fspec_path.exists() {
        return Err(Error::Semantic {
            msg: format!(".fspec not found at {}", fspec_path.display()),
        });
    }

    let contents = fs::read_to_string(&fspec_path).map_err(|e| Error::Io {
        path: fspec_path.clone(),
        source: e,
    })?;

    let spec_rules = parse_fspec(&contents)?;

    // DEBUG: remove later.
    eprintln!("{:#?}", spec_rules);

    // - walk filesystem
    // - classify paths
    let classified = walk_and_classify(root, &spec_rules)?;

    eprintln!("{:#?}", classified);

    let mut report = Report::default();

    // Optionally: don’t ever report on root .fspec as "unaccounted"
    // (either skip it here, or skip it during the walk)
    // report.set_status(".fspec", Status::Allowed);

    for p in classified
        .ignored_dirs
        .iter()
        .chain(classified.ignored_files.iter())
    {
        report.set_status(p.to_string_lossy().replace('\\', "/"), Status::Ignored);
    }

    for p in classified
        .unclassified_dirs
        .iter()
        .chain(classified.unclassified_files.iter())
    {
        // optionally skip ".fspec"
        if p.as_os_str() == ".fspec" {
            continue;
        }
        report.set_status(p.to_string_lossy().replace('\\', "/"), Status::Unaccounted);
    }

    for p in classified
        .allowed_dirs
        .iter()
        .chain(classified.allowed_files.iter())
    {
        report.set_status(p.to_string_lossy().replace('\\', "/"), Status::Allowed);
    }

    Ok(report)

    // TEMPORARY: return early until next stages exist
    // Err(Error::Semantic {
    //     msg: "Unimplemented error".into(),
    // })
}
