#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub index: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at byte {}: {}", self.index, self.message)
    }
}

impl std::error::Error for ParseError {}
