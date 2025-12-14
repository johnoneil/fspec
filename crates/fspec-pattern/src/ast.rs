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
pub struct Pattern {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Slash,
    Segment(Segment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    // whole-segment tokens
    GlobStar,
    Star,
    Dot,
    DotDot,

    // normal segment contents
    Parts(Vec<SegPart>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegPart {
    Literal(String),
    NamedPlaceholder {
        name: String,
        limiter: Option<Limiter>,
    },
    AnonymousPlaceholder {
        limiter: Limiter,
    },
}
