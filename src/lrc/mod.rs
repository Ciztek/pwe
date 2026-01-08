pub mod error;
pub mod parser;
pub mod timestamp;
pub mod tokenizer;
pub mod tokens;

pub use error::LrcError;
pub use tokens::LrcEvent;

use std::path::Path;

// convenience parser entry
pub fn parse_lrc(text: &str) -> Result<Vec<LrcEvent>, LrcError> {
    parser::parse_lrc(text)
}

pub fn parse_lrc_file(path: &Path) -> Result<Vec<LrcEvent>, LrcError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| LrcError::Io(format!("Failed to read LRC file {}: {}", path.display(), e)))?;
    parse_lrc(&text)
}
