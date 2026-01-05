use fspec_placeholder::ComponentAst;
use regex::Regex;

/// A compiled component with a pre-compiled regex for efficient matching.
///
/// This stores the original ComponentAst along with a compiled regex pattern
/// and placeholder index mappings, avoiding repeated regex compilation during
/// tree traversal.
#[derive(Debug, Clone)]
pub struct CompiledComponent {
    /// The original AST (kept for debugging/comparison)
    pub ast: ComponentAst,
    /// Pre-compiled regex pattern (anchored with ^ and $)
    pub regex: Regex,
    /// Mapping of placeholder names to capture group indices
    pub placeholder_indices: Vec<(String, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    Info,
    // TODO: not currently used so unsure if this is correct
    #[default]
    Warning,
    Error,
}

/// core settings (to be expanded)
#[derive(Debug, Clone, Copy)]
pub struct MatchSettings {
    /// If true, a non-slash-terminated leaf may match a file OR a directory
    pub allow_file_or_dir_leaf: bool,
    pub default_severity: Severity,
}

impl Default for MatchSettings {
    fn default() -> Self {
        Self {
            allow_file_or_dir_leaf: true,
            default_severity: Severity::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Allow,
    Ignore,
}

// Note: Rule, FSPattern, FSEntry, DirType, FileType, and FileOrDirType
// no longer derive Eq/PartialEq because CompiledComponent contains Regex,
// which doesn't implement Eq/PartialEq.
#[derive(Debug, Clone)]
pub struct Rule {
    pub line: usize,
    pub kind: RuleKind,
    pub pattern: FSPattern,
}

#[derive(Debug, Clone)]
pub enum FSPattern {
    Anchored(Vec<FSEntry>),
    Unanchored(Vec<FSEntry>),
}

#[derive(Debug, Clone)]
pub enum FSEntry {
    // Unambiguously a directory
    Dir(DirType),
    // Unambiguously a file
    File(FileType),
    // Either a file or a directory.
    // This may seem odd but is the general UNIX behavior for tooling, where
    // "./bin" represents "either a file or a directory"
    Either(FileOrDirType),
}

#[derive(Debug, Clone)]
pub enum DirType {
    Component(CompiledComponent),
    Star,
    DoubleStar,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Component(CompiledComponent),
    Star,
}

#[derive(Debug, Clone)]
pub enum FileOrDirType {
    Component(CompiledComponent),
    Star,
}
