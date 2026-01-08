mod args;
mod render;

use crate::args::{Cli, LeafMode, OutputFormat, SeverityArg};
use clap::Parser;
use fspec_core::{MatchSettings, Severity, check_tree, check_tree_with_spec};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let root: PathBuf = resolve_root(&cli);

    let mut settings = MatchSettings::default();

    // These fields are described in your CLI design doc.
    // If your MatchSettings uses setters instead of public fields, adapt accordingly.
    settings.allow_file_or_dir_leaf = matches!(cli.leaf, LeafMode::Loose);
    settings.default_severity = match cli.severity {
        SeverityArg::Info => Severity::Info,
        SeverityArg::Warning => Severity::Warning,
        SeverityArg::Error => Severity::Error,
    };

    let report = (if let Some(spec) = cli.spec.as_deref() {
        check_tree_with_spec(&root, Some(spec), &settings)
    } else {
        check_tree(&root, &settings)
    })
    .unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    let out = render::render(&report, &settings, cli.format, cli.verbosity, cli.quiet);

    println!("{}", out);

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
