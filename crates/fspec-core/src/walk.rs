use crate::matcher::matches_allowed_anchored_dir;
use crate::matcher::matches_allowed_anchored_file;
use crate::matcher::matches_allowed_unanchored_dir;
use crate::matcher::matches_allowed_unanchored_file;
use crate::matcher::matches_ignored_anchored_dir;
use crate::matcher::matches_ignored_anchored_file;
use crate::matcher::matches_ignored_unanchored_dir;
use crate::matcher::matches_ignored_unanchored_file;
use crate::spec::RuleKind;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Rule};

#[derive(Debug, Clone, Default)]
pub struct WalkOutput {
    pub allowed_files: HashSet<PathBuf>,
    pub allowed_dirs: HashSet<PathBuf>,
    pub ignored_files: HashSet<PathBuf>,
    pub ignored_dirs: HashSet<PathBuf>,
    pub unaccounted_files: HashSet<PathBuf>,
    pub unaccounted_dirs: HashSet<PathBuf>,
}

impl WalkOutput {
    fn allow_with_ancestors(&mut self, path: PathBuf, is_file: bool) {
        // 1) allow the path itself
        if is_file {
            self.allowed_files.insert(path.clone());
            self.unaccounted_files.remove(&path);
            self.ignored_files.remove(&path);
        } else {
            self.allowed_dirs.insert(path.clone());
            self.unaccounted_dirs.remove(&path);
            self.ignored_dirs.remove(&path);
        }

        // 2) walk ancestors (dirs only)
        let mut cur = path.parent();

        while let Some(dir) = cur {
            // 🔧 Stop before the empty relative root ("")
            if dir.as_os_str().is_empty() {
                break;
            }

            let pb = dir.to_path_buf();

            self.allowed_dirs.insert(pb.clone());
            self.unaccounted_dirs.remove(&pb);
            self.ignored_dirs.remove(&pb);

            cur = dir.parent();
        }
    }

    pub fn mark_unaccounted_dir(&mut self, path: PathBuf) {
        // Don't mark if already justified
        if self.allowed_dirs.contains(&path) || self.ignored_dirs.contains(&path) {
            return;
        }

        self.unaccounted_dirs.insert(path);
    }

    pub fn mark_unaccounted_file(&mut self, path: PathBuf) {
        if self.allowed_files.contains(&path) || self.ignored_files.contains(&path) {
            return;
        }

        self.unaccounted_files.insert(path);
    }

    pub fn mark_ignored_dir(&mut self, path: PathBuf) {
        self.ignored_dirs.insert(path.clone());
        self.unaccounted_dirs.remove(&path);
    }

    pub fn mark_ignored_file(&mut self, path: PathBuf) {
        self.ignored_files.insert(path.clone());
        self.unaccounted_files.remove(&path);
    }
}

/// Per-directory traversal context.
///
/// This is intentionally "empty" today, but shaped so it can evolve into
/// the Option A model:
/// - "live candidates" (rules that can still match in this subtree)
/// - "effective decisions" inherited from parents (e.g., subtree ignored)
#[derive(Debug, Clone)]
pub struct WalkCtx {
    /// Root of the check (where .fspec lives).
    pub root: PathBuf,

    /// Current path relative to root ("" means root itself).
    pub rel: PathBuf,

    /// Depth from root. Useful for debug indentation.
    pub depth: usize,

    /// The returned list of allowed/ignored/unaccounted for dirs+files
    pub walk_output: WalkOutput,

    /// Placeholder for Option A: the set of rule indices still "in play" in this subtree.
    /// Today we just carry it forward unchanged.
    pub live_rule_idxs: Vec<usize>,

    /// Placeholder for Option A: an inherited "subtree status".
    /// For example, later you might store "ignored by rule #N unless overridden".
    pub inherited: InheritedState,
}

#[derive(Debug, Clone)]
pub enum InheritedState {
    None,
    // We're in an ignored subtree, ignored by rule at rule_index
    SubtreeIgnored { rule_idx: usize },
    // etc.
}

pub fn walk_tree(root: &Path, rules: &[Rule]) -> Result<WalkOutput, Error> {
    let mut ctx = WalkCtx {
        root: root.to_path_buf(),
        rel: PathBuf::new(),
        depth: 0,
        walk_output: WalkOutput::default(),
        live_rule_idxs: (0..rules.len()).collect(),
        inherited: InheritedState::None,
    };

    walk_dir(&mut ctx, rules)?;

    Ok(ctx.walk_output)
}

/// Walk a directory with a mutable context representing "where we are".
///
/// Today we only print debug info and recurse.
/// Later, this is where you will:
/// - refine ctx.live_rule_idxs based on which rules can still match below
/// - compute effective decisions for this directory (dir-only allow, ignore subtree, etc.)
fn walk_dir(ctx: &mut WalkCtx, rules: &[Rule]) -> Result<(), Error> {
    // Build the absolute path we are currently at.
    let abs = ctx.root.join(&ctx.rel);

    // Debug: entering directory
    debug_enter(ctx, &abs);

    let rd = fs::read_dir(&abs).map_err(|e| Error::Io {
        path: abs.clone(),
        source: e,
    })?;

    // Collect and sort entries for deterministic traversal output (helps goldens later).
    let mut entries: Vec<fs::DirEntry> = Vec::new();
    for ent in rd {
        let ent = ent.map_err(|e| Error::Io {
            path: abs.clone(),
            source: e,
        })?;
        entries.push(ent);
    }
    entries.sort_by_key(|e| e.file_name());

    for ent in entries {
        let name = ent.file_name();
        let name = name.to_string_lossy();

        // Skip the spec file itself (optional, but usually desired).
        if ctx.rel.as_os_str().is_empty() && name == ".fspec" {
            continue;
        }

        let ty = ent.file_type().map_err(|e| Error::Io {
            path: ent.path(),
            source: e,
        })?;

        if ty.is_dir() {
            // --- Option A hook: advance state for this child directory ---
            //
            // Eventually:
            // let child_ctx = advance_ctx_for_dir(ctx, rules, &name);
            //
            // For now we simply descend, carrying state unchanged.

            let saved_rel = ctx.rel.clone();
            let saved_depth = ctx.depth;
            let saved_live = ctx.live_rule_idxs.clone();
            let saved_inh = ctx.inherited.clone();

            ctx.rel.push(name.as_ref());
            ctx.depth += 1;

            let rel_path = ctx.rel.clone();

            match classify_entry_last_wins(ctx, rules, &rel_path, EntryKind::Dir) {
                Verdict::Allow { .. } => ctx
                    .walk_output
                    .allow_with_ancestors(rel_path.clone(), false),
                Verdict::Unaccounted => ctx.walk_output.mark_unaccounted_dir(rel_path),
                Verdict::Ignore { rule_idx } => {
                    ctx.walk_output.mark_ignored_dir(rel_path.clone());
                    // we just ignored a directory. set the inherited context flag.
                    ctx.inherited = InheritedState::SubtreeIgnored { rule_idx };
                }
                Verdict::IgnoredByInheritance { .. } => {
                    ctx.walk_output.mark_ignored_dir(rel_path);
                }
            }

            // Recurse
            walk_dir(ctx, rules)?;

            // Restore context (so we can continue siblings)
            ctx.rel = saved_rel;
            ctx.depth = saved_depth;
            ctx.live_rule_idxs = saved_live;
            ctx.inherited = saved_inh;
        } else if ty.is_file() {
            let rel_path = ctx.rel.join(name.as_ref());

            match classify_entry_last_wins(ctx, rules, &rel_path, EntryKind::File) {
                Verdict::Allow { .. } => {
                    ctx.walk_output.allow_with_ancestors(rel_path.clone(), true)
                }
                Verdict::Unaccounted => ctx.walk_output.mark_unaccounted_file(rel_path),
                Verdict::Ignore { .. } => {
                    ctx.walk_output.mark_ignored_file(rel_path.clone());
                }
                Verdict::IgnoredByInheritance { .. } => {
                    ctx.walk_output.mark_ignored_file(rel_path);
                }
            }
        } else {
            // symlink / fifo / socket / etc.
            // For now, silently skip (could be reported in future)
        }
    }

    debug_exit(ctx, &abs);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum EntryKind {
    File,
    Dir,
}

#[derive(Debug, Clone, Copy)]
enum Verdict {
    Allow { rule_idx: usize }, // rule_idx reserved for future diagnostics
    Ignore { rule_idx: usize },
    IgnoredByInheritance { rule_idx: usize }, // rule_idx reserved for future diagnostics
    Unaccounted,
}

fn classify_entry_last_wins(
    ctx: &WalkCtx,
    rules: &[Rule],
    rel_path: &Path,
    kind: EntryKind,
) -> Verdict {
    // 1) last rule wins: scan from bottom to top over live rules
    // This must happen BEFORE checking inheritance, so that later rules can
    // override inherited ignore state (e.g., "ignore /bin/" then "allow /bin/allowed.txt")
    for &rule_idx in ctx.live_rule_idxs.iter().rev() {
        let r = &rules[rule_idx];

        // 2) extensible dispatch over rule kind + pattern kind + entry kind
        //
        // Today: only "allow + anchored + (dir|file)" exists.
        // Tomorrow: add ignore, unanchored, etc, in this same match ladder.

        match (r.kind, kind) {
            (RuleKind::Allow, EntryKind::Dir) => {
                if matches_allowed_anchored_dir(r, rel_path) {
                    return Verdict::Allow { rule_idx };
                }
                if matches_allowed_unanchored_dir(r, rel_path) {
                    return Verdict::Allow { rule_idx };
                }
            }

            (RuleKind::Allow, EntryKind::File) => {
                if matches_allowed_anchored_file(r, rel_path) {
                    return Verdict::Allow { rule_idx };
                }
                if matches_allowed_unanchored_file(r, rel_path) {
                    return Verdict::Allow { rule_idx };
                }
            }

            (RuleKind::Ignore, EntryKind::Dir) => {
                if matches_ignored_anchored_dir(r, rel_path) {
                    return Verdict::Ignore { rule_idx };
                }
                if matches_ignored_unanchored_dir(r, rel_path) {
                    return Verdict::Ignore { rule_idx };
                }
            }
            (RuleKind::Ignore, EntryKind::File) => {
                if matches_ignored_anchored_file(r, rel_path) {
                    return Verdict::Ignore { rule_idx };
                }
                if matches_ignored_unanchored_file(r, rel_path) {
                    return Verdict::Ignore { rule_idx };
                }
            }
        }
    }

    // 0) inheritance gate: only apply if no explicit rule matched
    // This allows later rules to override inherited ignore state
    if let InheritedState::SubtreeIgnored { rule_idx } = &ctx.inherited {
        return Verdict::IgnoredByInheritance {
            rule_idx: *rule_idx,
        };
    }

    Verdict::Unaccounted
}

#[cfg(debug_assertions)]
fn debug_enter(ctx: &WalkCtx, abs: &Path) {
    let rel_disp = if ctx.rel.as_os_str().is_empty() {
        ".".to_string()
    } else {
        ctx.rel.display().to_string()
    };

    eprintln!(
        "{}> enter {}  (abs={})  live_rules={}  inherited={:?}",
        indent(ctx.depth),
        rel_disp,
        abs.display(),
        ctx.live_rule_idxs.len(),
        ctx.inherited
    );
}

#[cfg(not(debug_assertions))]
fn debug_enter(_ctx: &WalkCtx, _abs: &Path) {}

#[cfg(debug_assertions)]
fn debug_exit(ctx: &WalkCtx, _abs: &Path) {
    let rel_disp = if ctx.rel.as_os_str().is_empty() {
        ".".to_string()
    } else {
        ctx.rel.display().to_string()
    };

    eprintln!("{}< exit  {}", indent(ctx.depth), rel_disp);
}

#[cfg(not(debug_assertions))]
fn debug_exit(_ctx: &WalkCtx, _abs: &Path) {}

#[cfg(debug_assertions)]
fn indent(depth: usize) -> String {
    // 2 spaces per depth, cheap and readable
    "  ".repeat(depth)
}
