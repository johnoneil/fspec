// src/ast.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Literal(String),
    Slash,
    GlobStar,
    Placeholder { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub nodes: Vec<Node>,
}
