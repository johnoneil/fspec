// src/lib.rs
mod ast;
mod error;
mod parser;

use crate::error::ParseError;

pub use ast::{Limiter, LimiterKind, Node, Pattern, Quant, SegPart, Segment};

pub fn parse_pattern(input: &str) -> Result<Pattern, ParseError> {
    parser::parse_pattern(input)
}
