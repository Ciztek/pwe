use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<PathBuf>,
    #[serde(default)]
    pub created_at: String,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn add_song(&mut self, path: PathBuf) {
        if !self.songs.contains(&path) {
            self.songs.push(path);
        }
    }

    pub fn remove_song(&mut self, path: &PathBuf) {
        self.songs.retain(|p| p != path);
    }

    pub fn contains(&self, path: &PathBuf) -> bool {
        self.songs.contains(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistCollection {
    pub playlists: Vec<Playlist>,
}

impl PlaylistCollection {
    pub fn create_playlist(&mut self, name: String) -> bool {
        if self.playlists.iter().any(|p| p.name == name) {
            return false; // Playlist with this name already exists
        }
        self.playlists.push(Playlist::new(name));
        true
    }

    pub fn delete_playlist(&mut self, name: &str) -> bool {
        if let Some(pos) = self.playlists.iter().position(|p| p.name == name) {
            self.playlists.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_playlist_mut(&mut self, name: &str) -> Option<&mut Playlist> {
        self.playlists.iter_mut().find(|p| p.name == name)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
        let pwe_dir = config_dir.join("pwe-karaoke");

        std::fs::create_dir_all(&pwe_dir)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;

        let playlists_path = pwe_dir.join("playlists.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize playlists: {e}"))?;

        std::fs::write(&playlists_path, json)
            .map_err(|e| format!("Failed to write playlists file: {e}"))?;

        tracing::info!(
            "Saved {} playlists to {:?}",
            self.playlists.len(),
            playlists_path
        );
        Ok(())
    }

    pub fn load() -> Self {
        let Some(config_dir) = dirs::config_dir() else {
            tracing::warn!("Could not find config directory for playlists");
            return Self::default();
        };

        let playlists_path = config_dir.join("pwe-karaoke").join("playlists.json");

        if !playlists_path.exists() {
            tracing::info!("No playlists file found, starting with empty collection");
            return Self::default();
        }

        match std::fs::read_to_string(&playlists_path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(collection) => {
                    tracing::info!("Loaded playlists from {:?}", playlists_path);
                    collection
                },
                Err(e) => {
                    tracing::error!("Failed to parse playlists file: {}", e);
                    Self::default()
                },
            },
            Err(e) => {
                tracing::error!("Failed to read playlists file: {}", e);
                Self::default()
            },
        }
    }
}
