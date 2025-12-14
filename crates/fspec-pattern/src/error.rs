#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub index: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(literal: String) -> ParseError {
        // TODO: what should index be?
        ParseError {
            index: 1,
            message: literal,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at byte {}: {}", self.index, self.message)
    }
}

impl std::error::Error for ParseError {}
