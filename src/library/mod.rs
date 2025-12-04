// Library module - song library management
pub mod scanner;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Song {
    pub path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub extension: String,
}

impl Song {
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
