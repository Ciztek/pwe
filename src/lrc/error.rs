use std::fmt;

#[derive(Debug)]
pub enum LrcError {
    InvalidTimestamp(String),
    #[allow(dead_code)]
    UnexpectedToken(String),
    Io(String),
}

impl std::error::Error for LrcError {}

impl fmt::Display for LrcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimestamp(s) => write!(f, "Invalid timestamp: {s}"),
            Self::UnexpectedToken(s) => write!(f, "Unexpected token: {s}"),
            Self::Io(s) => write!(f, "IO error: {s}"),
        }
    }
}
