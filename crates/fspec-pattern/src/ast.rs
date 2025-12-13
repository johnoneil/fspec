// src/ast.rs
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
pub enum Limiter {
    Int { min_digits: usize, allow_more: bool }, // int(3) or int(3+)
    Semver,                                      // semver
    CamelCase,                                   // camel_case
    PascalCase,                                  // pascal_case
    SnakeCase,                                   // snake_case
    KebabCase,                                   // kebab_case
    FlatCase,                                    // flat_case
    UpperCase,                                   // upper_case
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub nodes: Vec<Node>,
}
