use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;
use tracing::{error, info};

use crate::song::Song;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac"];

fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        AUDIO_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Library {
    songs: Vec<Song>,
    path: PathBuf,
}

impl Library {
    pub fn get_library_directory() -> Result<PathBuf> {
        // For installed applications, use the standard data directory
        // For development, use a subdirectory in the project

        #[cfg(not(debug_assertions))]
        {
            // Production: Use platform-specific app data directory
            let app_data = if cfg!(target_os = "windows") {
                // Windows: %APPDATA%\PWE Karaoke
                std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .context("Failed to get APPDATA directory")?
                    .join("PWE Karaoke")
            } else if cfg!(target_os = "macos") {
                // macOS: ~/Library/Application Support/PWE Karaoke
                dirs::data_dir()
                    .context("Failed to get data directory")?
                    .join("PWE Karaoke")
            } else {
                // Linux: ~/.local/share/pwe-karaoke
                dirs::data_dir()
                    .context("Failed to get data directory")?
                    .join("pwe-karaoke")
            };

            let library_dir = app_data.join("Library");
            std::fs::create_dir_all(&library_dir).context("Failed to create library directory")?;

            info!("Library directory: {}", library_dir.display());
            Ok(library_dir)
        }

        #[cfg(debug_assertions)]
        {
            // Development: Use local directory
            let library_dir = PathBuf::from("dev_library");
            std::fs::create_dir_all(&library_dir)
                .context("Failed to create development library directory")?;

            info!("Development library directory: {}", library_dir.display());
            Ok(library_dir)
        }
    }

    pub fn try_new() -> Result<Self> {
        let library_dir = Self::get_library_directory()?;
        Ok(Self {
            songs: Vec::new(),
            path: library_dir,
        })
    }

    pub fn try_scan(&mut self) -> Result<Self> {
        info!("Scanning library folder: {}", self.path.display());

        let mut songs = Vec::new();
        for entry in walkdir::WalkDir::new(&self.path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            if !entry_path.is_file() {
                continue;
            }

            if !is_audio_file(entry_path) {
                continue;
            }

            match Song::from_path(entry_path.to_path_buf()) {
                Some(song) => {
                    songs.push(song);
                },
                None => {
                    error!("Failed to parse song from: {}", entry_path.display());
                },
            }
        }
        self.songs = songs;
        Ok(self.clone())
    }

    pub fn songs(&self) -> &Vec<Song> {
        &self.songs
    }
}
