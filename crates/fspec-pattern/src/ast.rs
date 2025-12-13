#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quant {
    Any,
    Exactly(usize),
    AtLeast(usize),
    Range { min: usize, max: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimiterKind {
    Int,
    Semver,
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    FlatCase,
    UpperCase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limiter {
    pub kind: LimiterKind,
    pub quant: Quant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Literal(String),
    Slash,
    GlobStar,
    Placeholder {
        name: String,
        limiter: Option<Limiter>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub nodes: Vec<Node>,
}
