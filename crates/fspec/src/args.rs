use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// fspec: validate a directory tree against an .fspec file.
#[derive(Debug, Parser)]
#[command(name = "fspec")]
#[command(about = "Validate a directory tree against .fspec rules", long_about = None)]
pub struct Cli {
    /// Target PATH to scan (defaults to current directory)
    pub path: Option<PathBuf>,

    /// Explicit scan root (overrides PATH if provided)
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Explicit spec file path (NOT IMPLEMENTED YET in core)
    #[arg(long)]
    pub spec: Option<PathBuf>,

    /// Leaf matching mode
    #[arg(long, value_enum, default_value_t = LeafMode::Loose)]
    pub leaf: LeafMode,

    /// Default severity to use for findings (and current fail threshold)
    #[arg(long, value_enum, default_value_t = SeverityArg::Warning)]
    pub severity: SeverityArg,

    /// Quiet output (print only paths)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Verbosity (-v, -vv)
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LeafMode {
    Strict,
    Loose,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SeverityArg {
    Info,
    Warning,
    Error,
}
