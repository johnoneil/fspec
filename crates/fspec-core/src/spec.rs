#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleKind {
    Allow,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub line: usize, // 1-based line number for diagnostics
    pub kind: RuleKind,
    pub pattern: String, // raw pattern string (uncompiled)
}
