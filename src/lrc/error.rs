use std::fmt;

#[derive(Debug)]
pub enum LrcError {
    InvalidTimestamp(String),
    UnexpectedToken(String),
    Io(String),
}

impl std::error::Error for LrcError {}

impl fmt::Display for LrcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LrcError::InvalidTimestamp(s) => write!(f, "Invalid timestamp: {}", s),
            LrcError::UnexpectedToken(s) => write!(f, "Unexpected token: {}", s),
            LrcError::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}
