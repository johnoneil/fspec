use crate::matcher::matches_allowed_anchored_dir;
use crate::matcher::matches_allowed_anchored_file;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Rule};

#[derive(Debug, Clone, Default)]
pub struct WalkOutput {
    pub allowed_files: Vec<PathBuf>,
    pub allowed_dirs: Vec<PathBuf>,
    pub ignored_files: Vec<PathBuf>,
    pub ignored_dirs: Vec<PathBuf>,
    pub unaccounted_files: Vec<PathBuf>,
    pub unaccounted_dirs: Vec<PathBuf>,
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
    // Later examples:
    // SubtreeIgnored { rule_idx: usize },
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

    eprintln!("walk: start at {}", root.display());
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

            // DEBUG: Just add everything to allowed.
            //ctx.walk_output.allowed_dirs.push(ctx.rel.clone());

            let rel_path = ctx.rel.clone();

            if rules
                .iter()
                .any(|r| matches_allowed_anchored_dir(r, &rel_path))
            {
                ctx.walk_output.allowed_dirs.push(rel_path);
            } else {
                ctx.walk_output.unaccounted_dirs.push(rel_path);
            }

            // Debug: directory child
            eprintln!("{}+ dir  {}", indent(ctx.depth), ctx.rel.display());

            // Recurse
            walk_dir(ctx, rules)?;

            // Restore context (so we can continue siblings)
            ctx.rel = saved_rel;
            ctx.depth = saved_depth;
            ctx.live_rule_idxs = saved_live;
            ctx.inherited = saved_inh;
        } else if ty.is_file() {
            // DEBUG: just add everything to allowed.
            // ctx.walk_output
            //     .allowed_files
            //     .push(ctx.rel.join(name.as_ref()));

            let rel_path = ctx.rel.join(name.as_ref());

            if rules
                .iter()
                .any(|r| matches_allowed_anchored_file(r, &rel_path))
            {
                ctx.walk_output.allowed_files.push(rel_path);
            } else {
                ctx.walk_output.unaccounted_files.push(rel_path);
            }

            eprintln!(
                "{}- file {}",
                indent(ctx.depth + 1),
                ctx.rel.join(name.as_ref()).display()
            );
        } else {
            // symlink / fifo / socket / etc.
            // For now, just note it.
            eprintln!(
                "{}? other {}",
                indent(ctx.depth + 1),
                ctx.rel.join(name.as_ref()).display()
            );
        }
    }

    debug_exit(ctx, &abs);
    Ok(())
}

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

fn debug_exit(ctx: &WalkCtx, _abs: &Path) {
    let rel_disp = if ctx.rel.as_os_str().is_empty() {
        ".".to_string()
    } else {
        ctx.rel.display().to_string()
    };

    eprintln!("{}< exit  {}", indent(ctx.depth), rel_disp);
}

fn indent(depth: usize) -> String {
    // 2 spaces per depth, cheap and readable
    "  ".repeat(depth)
}
