use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Represents metadata for a song in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub original_path: PathBuf,
    pub stored_filename: String,
    pub title: String,
    pub added_date: String,
}

/// Manages the persistent library storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub entries: Vec<LibraryEntry>,
}

impl LibraryMetadata {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: LibraryEntry) {
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, stored_filename: &str) -> Option<LibraryEntry> {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.stored_filename == stored_filename)
        {
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).context("Failed to read library metadata file")?;
        let metadata: LibraryMetadata =
            serde_json::from_str(&contents).context("Failed to parse library metadata")?;
        Ok(metadata)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize library metadata")?;
        std::fs::write(path, json).context("Failed to write library metadata file")?;
        Ok(())
    }
}

/// Gets the application data directory where the library is stored
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

/// Gets the path to the library metadata file
pub fn get_metadata_file() -> Result<PathBuf> {
    let library_dir = get_library_directory()?;
    Ok(library_dir.join("library.json"))
}

/// Copies a file to the library storage with a unique name
pub fn copy_to_library(source: &Path) -> Result<String> {
    let library_dir = get_library_directory()?;

    // Generate a unique filename based on the original filename and timestamp
    let _original_name = source
        .file_name()
        .and_then(|s| s.to_str())
        .context("Invalid source filename")?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("Failed to get timestamp")?
        .as_secs();

    let extension = source.extension().and_then(|s| s.to_str()).unwrap_or("");

    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let stored_filename = format!("{}_{}.{}", stem, timestamp, extension);
    let dest_path = library_dir.join(&stored_filename);

    info!(
        "Copying {} to library as {}",
        source.display(),
        stored_filename
    );

    std::fs::copy(source, &dest_path).context("Failed to copy file to library")?;

    Ok(stored_filename)
}

/// Removes a file from the library storage
pub fn remove_from_library(stored_filename: &str) -> Result<()> {
    let library_dir = get_library_directory()?;
    let file_path = library_dir.join(stored_filename);

    if file_path.exists() {
        info!("Removing {} from library", stored_filename);
        std::fs::remove_file(&file_path).context("Failed to remove file from library")?;
    }

    Ok(())
}

/// Loads the library metadata, creating a new one if it doesn't exist
pub fn load_library_metadata() -> LibraryMetadata {
    match get_metadata_file() {
        Ok(metadata_path) => {
            if metadata_path.exists() {
                match LibraryMetadata::load_from_file(&metadata_path) {
                    Ok(metadata) => {
                        info!(
                            "Loaded library metadata with {} entries",
                            metadata.entries.len()
                        );
                        metadata
                    },
                    Err(e) => {
                        error!("Failed to load library metadata: {}", e);
                        LibraryMetadata::new()
                    },
                }
            } else {
                info!("No existing library metadata, creating new");
                LibraryMetadata::new()
            }
        },
        Err(e) => {
            error!("Failed to get metadata file path: {}", e);
            LibraryMetadata::new()
        },
    }
}

/// Saves the library metadata
pub fn save_library_metadata(metadata: &LibraryMetadata) -> Result<()> {
    let metadata_path = get_metadata_file()?;
    metadata.save_to_file(&metadata_path)?;
    info!(
        "Saved library metadata with {} entries",
        metadata.entries.len()
    );
    Ok(())
}
