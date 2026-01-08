// Library module - song library management
pub mod scanner;
pub mod storage;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Song {
    pub path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub extension: String,
    pub has_lyrics: bool,
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

        // Check if corresponding .lrc file exists
        let lrc_path = path.with_extension("lrc");
        let has_lyrics = lrc_path.exists();

        Some(Song {
            path,
            name,
            extension,
            has_lyrics,
        })
    }
}
