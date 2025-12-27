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
    Dir(DirType),
    File(FileType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirType {
    Lit(String),
    Star,
    DoubleStar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Lit(String),
    Star,
}
