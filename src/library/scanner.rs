// Scanner - scan folders for audio files
use super::Song;
use std::path::Path;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Supported audio file extensions
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac"];

/// Check if a file has a supported audio extension
fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        AUDIO_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}

/// Scan a directory recursively for audio files
pub fn scan_directory<P: AsRef<Path>>(path: P) -> Vec<Song> {
    let path = path.as_ref();

    info!("Scanning directory: {}", path.display());

    let mut songs = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        // Skip directories
        if !entry_path.is_file() {
            continue;
        }

        // Check if it's an audio file
        if !is_audio_file(entry_path) {
            continue;
        }

        // Try to create a Song from the path
        match Song::from_path(entry_path.to_path_buf()) {
            Some(song) => {
                songs.push(song);
            },
            None => {
                warn!("Failed to parse song from: {}", entry_path.display());
            },
        }
    }

    // Sort by name for consistent ordering
    songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    info!("Found {} audio files", songs.len());

    songs
}
