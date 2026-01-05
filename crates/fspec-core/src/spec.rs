use fspec_placeholder::ComponentAst;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub line: usize,
    pub kind: RuleKind,
    pub pattern: FSPattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FSPattern {
    Anchored(Vec<FSEntry>),
    Unanchored(Vec<FSEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirType {
    Component(ComponentAst),
    Star,
    DoubleStar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Component(ComponentAst),
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOrDirType {
    Component(ComponentAst),
    Star,
}
