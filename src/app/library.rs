// Music library management

use std::path::{Path, PathBuf};
use tracing::{error, info};

use crate::library::{scanner, storage, Song};

pub struct Library {
    #[allow(clippy::struct_field_names)]
    pub library: Vec<Song>,
    #[allow(clippy::struct_field_names)]
    pub library_path: Option<PathBuf>,
    #[allow(clippy::struct_field_names)]
    pub library_filter: String,
    #[allow(clippy::struct_field_names)]
    pub library_view_filter: crate::app::state::LibraryViewFilter,
    pub metadata: storage::LibraryMetadata,
    #[allow(clippy::struct_field_names)]
    pub library_dir: Option<PathBuf>,
    pub add_song_path_input: String,
    pub play_history: Vec<PathBuf>,
    pub playlists: crate::playlist::PlaylistCollection,
    pub new_playlist_name: String,
    pub show_playlist_dialog: bool,
}

impl Library {
    pub fn new() -> Self {
        let mut metadata = storage::load_library_metadata();
        let library_dir = storage::get_library_directory().ok();

        if let Err(e) = storage::sync_library(&mut metadata) {
            error!("Failed to sync library: {}", e);
        } else if let Err(e) = storage::save_library_metadata(&metadata) {
            error!("Failed to save synced library metadata: {}", e);
        }

        let config = crate::config::AppConfig::load();
        let play_history: Vec<PathBuf> = config
            .library
            .play_history
            .iter()
            .map(PathBuf::from)
            .collect();

        let mut lib = Self {
            library: Vec::new(),
            library_path: None,
            library_filter: String::new(),
            library_view_filter: crate::app::state::LibraryViewFilter::AllSongs,
            metadata,
            library_dir: library_dir.clone(),
            add_song_path_input: String::new(),
            play_history,
            playlists: crate::playlist::PlaylistCollection::load(),
            new_playlist_name: String::new(),
            show_playlist_dialog: false,
        };

        if let Some(dir) = library_dir {
            lib.load_library_from_storage(&dir);
        }

        lib
    }

    /// Loads songs from the persistent library storage
    pub fn load_library_from_storage(&mut self, library_dir: &PathBuf) {
        info!("Loading library from storage: {}", library_dir.display());
        self.library = scanner::scan_directory(library_dir);

        for song in &mut self.library {
            if let Some(filename) = song.path.file_name().and_then(|n| n.to_str()) {
                if let Some(entry) = self
                    .metadata
                    .entries
                    .iter()
                    .find(|e| e.stored_filename == filename)
                {
                    song.is_favorite = entry.is_favorite;
                }
            }
        }

        self.library_path = Some(library_dir.clone());
        info!("Library loaded with {} songs", self.library.len());

        let with_lyrics = self.library.iter().filter(|s| s.has_lyrics).count();
        if with_lyrics > 0 {
            info!("{} songs have lyrics files", with_lyrics);
        }
    }

    /// Refreshes the library by syncing metadata and rescanning
    pub fn refresh_library(&mut self) {
        info!("Refreshing library...");

        if let Err(e) = storage::sync_library(&mut self.metadata) {
            error!("Failed to sync library: {}", e);
        } else if let Err(e) = storage::save_library_metadata(&self.metadata) {
            error!("Failed to save synced library metadata: {}", e);
        }

        if let Some(dir) = self.library_dir.clone() {
            self.load_library_from_storage(&dir);
        }

        info!("Library refresh complete");
    }

    /// Adds a file to the persistent library storage
    pub fn add_to_library(&mut self, source_path: &Path) {
        match storage::copy_to_library(source_path) {
            Ok(stored_filename) => {
                let title = source_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let entry = storage::LibraryEntry {
                    original_path: source_path.to_path_buf(),
                    stored_filename,
                    title,
                    added_date: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    is_favorite: false,
                };

                self.metadata.add_entry(entry);

                if let Err(e) = storage::save_library_metadata(&self.metadata) {
                    error!("Failed to save library metadata: {}", e);
                }

                if let Some(dir) = self.library_dir.clone() {
                    self.load_library_from_storage(&dir);
                }

                info!("Added {} to library", source_path.display());
            },
            Err(e) => {
                error!("Failed to add file to library: {}", e);
            },
        }
    }

    /// Removes a song from the persistent library storage
    pub fn remove_from_library(&mut self, song: &Song) {
        let stored_filename = song.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if let Some(entry) = self.metadata.remove_entry(stored_filename) {
            if let Err(e) = storage::remove_from_library(&entry.stored_filename) {
                error!("Failed to remove file from library: {}", e);
            }

            if let Err(e) = storage::save_library_metadata(&self.metadata) {
                error!("Failed to save library metadata: {}", e);
            }

            if let Some(dir) = self.library_dir.clone() {
                self.load_library_from_storage(&dir);
            }

            info!("Removed {} from library", entry.title);
        }
    }

    /// Toggles favorite status for a song
    pub fn toggle_favorite(&mut self, path: &PathBuf) {
        if let Some(song) = self.library.iter_mut().find(|s| &s.path == path) {
            song.is_favorite = !song.is_favorite;

            let stored_filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Some(entry) = self
                .metadata
                .entries
                .iter_mut()
                .find(|e| e.stored_filename == stored_filename)
            {
                entry.is_favorite = song.is_favorite;

                if let Err(e) = storage::save_library_metadata(&self.metadata) {
                    error!("Failed to save library metadata: {}", e);
                } else {
                    let status = if song.is_favorite {
                        "favorited"
                    } else {
                        "unfavorited"
                    };
                    info!("Song {} {}", song.display_title(), status);
                }
            }
        }
    }

    /// Adds a song to play history (most recent first, max 10)
    pub fn add_to_history(&mut self, path: &PathBuf) {
        self.play_history.retain(|p| p != path);
        self.play_history.insert(0, path.clone());

        if self.play_history.len() > 10 {
            self.play_history.truncate(10);
        }

        info!(
            "Added to history: {} (history size: {})",
            path.display(),
            self.play_history.len()
        );
    }

    /// Opens a file dialog to add a song to the library
    pub fn add_song_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Add Song to Library")
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            self.add_to_library(&path);
        }
    }
}
