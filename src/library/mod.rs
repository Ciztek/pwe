// Library module - song library management
pub mod scanner;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Song {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
}

impl Song {
    /// Creates a Song from a file path by extracting name and extension.
    ///
    /// # Parameters
    /// - `path`: Full path to the audio file
    ///
    /// # Returns
    /// - `Some(Song)`: Successfully parsed filename and extension
    /// - `None`: Path has no filename, invalid UTF-8, or no extension
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
