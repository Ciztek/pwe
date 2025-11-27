// Library module - song library management
pub mod scanner;

use std::path::PathBuf;

/// Represents a song in the library
#[derive(Debug, Clone)]
pub struct Song {
    /// Full path to the audio file
    pub path: PathBuf,
    /// Display name (filename without extension)
    pub name: String,
    /// File extension
    pub extension: String,
}

impl Song {
    /// Create a new Song from a path
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())?;

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())?;

        Some(Song {
            path,
            name,
            extension,
        })
    }
}
