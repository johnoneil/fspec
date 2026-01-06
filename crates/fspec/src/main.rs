mod args;
mod render;

use crate::args::{Cli, LeafMode, SeverityArg};
use clap::Parser;
use fspec_core::{MatchSettings, Severity, check_tree};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let root: PathBuf = resolve_root(&cli);

    // NOTE: fspec-core currently hardcodes <root>/.fspec inside check_tree().
    // Until core supports a custom spec path, we error out if --spec is used.
    if cli.spec.is_some() {
        eprintln!(
            "error: --spec is not implemented yet (fspec-core currently only reads <root>/.fspec)"
        );
        return ExitCode::from(2);
    }

    let mut settings = MatchSettings::default();

    // These fields are described in your CLI design doc.
    // If your MatchSettings uses setters instead of public fields, adapt accordingly.
    settings.allow_file_or_dir_leaf = matches!(cli.leaf, LeafMode::Loose);
    settings.default_severity = match cli.severity {
        SeverityArg::Info => Severity::Info,
        SeverityArg::Warning => Severity::Warning,
        SeverityArg::Error => Severity::Error,
    };

    // TODO: Update check_tree to support running an a different directory than
    // the .fspec resides in. like:
    // let report = if let Some(spec) = &cli.spec {
    //     check_tree_with_spec(&root, spec, &settings)
    // } else {
    //     check_tree(&root, &settings)
    // };

    let report = match check_tree(&root, &settings) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let out = render::render_human(&report, &settings, cli.verbosity, cli.quiet);

    // For now always stdout (per your “ok if incomplete” note).
    // --output/--format can land tomorrow.
    print!("{out}");

    // Current “finding” heuristic: any unaccounted path => fail.
    // (In the future: incorporate per-item severity + threshold logic.)
    if report.unaccounted_paths().is_empty() {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

fn resolve_root(cli: &Cli) -> PathBuf {
    // Priority:
    //   1) --root
    //   2) positional PATH
    //   3) "."
    if let Some(r) = &cli.root {
        return r.clone();
    }
    if let Some(p) = &cli.path {
        return p.clone();
    }
    Path::new(".").to_path_buf()
}
