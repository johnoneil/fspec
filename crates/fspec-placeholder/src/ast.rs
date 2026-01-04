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
    /// `{a|b|"c"}` (unnamed) or `{name:a|b|"c"}` (named; must contain at least one pipe)
    OneOf(OneOfNode),

    /// `{name}` or `{name:lim(...)}`
    Capture(CaptureNode),
    // Later: Ref(String) if you want to distinguish `{year}` as reference.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneOfNode {
    /// Name for named one-of (e.g., `ext` in `{ext:mp4|mkv}`), or `None` for unnamed
    pub name: Option<NamedOneOf>,
    pub choices: Vec<Choice>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedOneOf {
    pub name: String,
    pub name_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Ident { value: String, span: Span },
    Str { value: String, span: Span }, // quoted string
}

/// A limiter applied to a capture, e.g. `int(4)` or `re("...")`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimiterSpec {
    /// Limiter name, e.g. "int", "re"
    pub name: String,

    /// Span of the limiter name
    pub name_span: Span,

    /// Arguments inside parentheses, if any
    pub args: Vec<LimiterArg>,

    /// Span of the entire limiter expression, e.g. `int(4)`
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureNode {
    pub name: String,
    pub name_span: Span,
    pub limiter: Option<LimiterSpec>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimiterArg {
    Number { value: String, span: Span },
    Ident { value: String, span: Span },
    Str { value: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Limiter {
    // --- Level-1 known limiters ---

    // ASCII case / style
    SnakeCase {
        span: Span,
    },
    KebabCase {
        span: Span,
    },
    PascalCase {
        span: Span,
    },
    UpperCase {
        span: Span,
    },
    LowerCase {
        span: Span,
    },

    // exactly n ASCII digits [0-9], where n >= 1
    Int {
        digits: u32,
        digits_span: Span,
        span: Span,
    },

    // regex escape hatch (dialect is implementation-defined; recommend Rust `regex`)
    Re {
        pattern: String,
        pattern_span: Span,
        span: Span,
    },

    // Unicode properties
    Letters {
        span: Span,
    }, // \p{L}
    Numbers {
        span: Span,
    }, // \p{Nd}
    Alnum {
        span: Span,
    }, // \p{L}\p{Nd}

    // --- Forward compatibility ---
    //
    // Parsers at a given conformance level may reject `Unknown`, but the AST
    // can still represent it to enable higher-level tooling and future levels.
    Unknown {
        name: String,
        name_span: Span,
        args: Vec<LimiterArg>,
        span: Span,
    },
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
