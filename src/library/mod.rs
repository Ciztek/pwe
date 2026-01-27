// Library module - song library management
pub mod scanner;
pub mod storage;

use crate::audio::metadata::AudioMetadata;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Song {
    pub path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub extension: String,
    pub has_lyrics: bool,
    pub metadata: Option<AudioMetadata>,
    pub is_favorite: bool,
}

impl Song {
    /// Creates a Song from a file path by extracting name, extension, and metadata.
    ///
    /// # Parameters
    /// - `path`: Full path to the audio file
    ///
    /// # Returns
    /// - `Some(Song)`: Successfully parsed filename, extension, and metadata
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

        // Extract audio metadata including cover art
        let metadata = crate::audio::metadata::extract_metadata(&path).ok();

        Some(Song {
            path,
            name,
            extension,
            has_lyrics,
            metadata,
            is_favorite: false,
        })
    }

    /// Gets the display title (metadata title or filename)
    pub fn display_title(&self) -> &str {
        self.metadata
            .as_ref()
            .and_then(|m| m.title.as_deref())
            .unwrap_or(&self.name)
    }

    /// Gets the artist name if available
    pub fn artist(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.artist.as_deref())
    }

    /// Gets the album name if available
    pub fn album(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.album.as_deref())
    }

    /// Gets duration in seconds if available
    pub fn duration(&self) -> Option<u64> {
        self.metadata.as_ref().and_then(|m| m.duration_secs)
    }
}
