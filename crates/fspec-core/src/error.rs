use fspec_placeholder::parser::ParseError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        line: usize,
        col: usize,
        msg: String,
    },
    Semantic {
        msg: String,
    },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io {
            path: "Unknown".into(),
            source: e,
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        // In component strings we don't expect newlines, so treat offset as "column".
        let line = 1;
        let col = e.at.saturating_add(1);

        let span_str = e
            .span
            .as_ref()
            .map(|s| format!("{}..{}", s.start, s.end))
            .unwrap_or_else(|| "none".to_string());

        let tok_str = e
            .source_tokenize
            .as_ref()
            .map(|s| format!(", tokenize={}", s))
            .unwrap_or_default();

        let msg = format!(
            "{} (kind={:?}, at={}, span={}{} )",
            e.message, e.kind, e.at, span_str, tok_str
        );

        Error::Parse { line, col, msg }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { path, source } => {
                write!(f, "IO error at {}: {}", path.display(), source)
            }
            Error::Parse { line, col, msg } => {
                write!(f, "Parse error at line {}, column {}: {}", line, col, msg)
            }
            Error::Semantic { msg } => {
                write!(f, "Semantic error: {}", msg)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
