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
