// src/lib.rs
mod ast;
mod error;
mod parser;

use crate::error::ParseError;

pub use ast::{FSPattern, Limiter, LimiterKind, Node, Quant, SegPart, Segment};

pub fn parse_pattern(input: &str) -> Result<FSPattern, ParseError> {
    parser::parse_pattern(input)
}
