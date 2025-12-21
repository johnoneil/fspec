#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
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
        Error::Io(e)
    }
}
