// src/lib.rs
mod ast;
mod parser;
use chumsky::Parser;

pub use ast::{Node, Pattern};

pub fn parse_pattern(input: &str) -> Result<Pattern, Vec<chumsky::error::Simple<char>>> {
    parser::pattern_parser().parse(input)
}
