//! Core specification data structures for fspec.
//!
//! These types represent the *meaning* of a parsed `.fspec` rule,
//! independent of how it was parsed or how it will be evaluated.
//!
//! Parsing code produces these structures.
//! Matching / evaluation code consumes them.

/// Whether a rule allows or ignores matching paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleKind {
    Allow,
    Ignore,
}

/// A single rule from a `.fspec` file.
///
/// Each rule comes from one line in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// 1-based line number in the `.fspec` file.
    /// Used for diagnostics and error reporting.
    pub line: usize,

    /// Whether this rule allows or ignores matching paths.
    pub kind: RuleKind,

    /// The parsed pattern describing which paths this rule applies to.
    pub pattern: Pattern,
}

/// A parsed path pattern.
///
/// Patterns may be anchored (starting with `/`) or unanchored.
///
/// Example:
///   `/assets/*/*.png`  -> Anchored
///   `assets/*/*.png`   -> Unanchored
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Pattern must match from the filesystem root.
    /// (Leading `/` in the `.fspec` pattern.)
    Anchored(Vec<Component>),

    /// Pattern matches relative to the root.
    /// (No leading `/` in the `.fspec` pattern.)
    Unanchored(Vec<Component>),
}

/// One component of a path pattern, split on `/`.
///
/// Each component describes what kind of filesystem entry
/// is expected at that position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    /// This component must match a directory entry.
    ///
    /// Used for all non-final path components, and also for
    /// final components when the pattern ends with a `/`.
    Dir(Segment),

    /// This component matches the final path entry.
    ///
    /// The entry may be a file or a directory, unless restricted
    /// by higher-level rules.
    Entry(Segment),
}

/// A single path segment matcher.
///
/// This describes *how the name of one path segment is matched*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// A literal path segment (no globbing).
    ///
    /// Examples:
    ///   `assets`
    ///   `images`
    ///   `foo.bar`
    Lit(String),

    /// A single-star glob (`*`), matching any single path segment.
    Star,

    /// A double-star glob (`**`), matching zero or more path segments.
    ///
    /// This is only meaningful in directory positions.
    DoubleStar,
}
