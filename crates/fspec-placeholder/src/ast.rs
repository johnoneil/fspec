// fspec-placeholder/src/ast.rs

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAst {
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Literal(LiteralPart),
    Star(Span),
    Placeholder(PlaceholderPart),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralPart {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderPart {
    pub node: PlaceholderNode,
    /// Span of the entire `{ ... }` including braces.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceholderNode {
    /// `{a|b|"c"}` (must contain at least one pipe)
    OneOf(OneOfNode),

    /// `{name}` or `{name:lim(...)}`
    Capture(CaptureNode),
    // Later: Ref(String) if you want to distinguish `{year}` as reference.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneOfNode {
    pub choices: Vec<Choice>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Ident { value: String, span: Span },
    Str { value: String, span: Span }, // quoted string
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureNode {
    pub name: String,
    pub name_span: Span,
    pub limiter: Option<LimiterSpec>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimiterSpec {
    pub name: String,
    pub name_span: Span,
    pub args: Vec<LimiterArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimiterArg {
    Number { value: String, span: Span },
    Ident { value: String, span: Span },
    Str { value: String, span: Span },
}

/// Byte span in the original component string.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize, // inclusive
    pub end: usize,   // exclusive
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn join(a: Span, b: Span) -> Span {
        Span {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }
}
