pub mod metadata;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use tracing::warn;

use metadata::{extract_metadata, AudioMetadata};

#[derive(Debug, Clone)]
pub struct Song {
    path: PathBuf,
    name: String,
    extension: String,
    has_lyrics: bool,
    metadata: Option<AudioMetadata>,
}

impl Serialize for Song {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            self.path
                .to_str()
                .ok_or_else(|| serde::ser::Error::custom("Invalid UTF-8 path"))?,
        )
    }
}

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path_str = String::deserialize(deserializer)?;
        let path = PathBuf::from(path_str);

        Song::from_path(path).ok_or_else(|| de::Error::custom("Failed to rebuild Song"))
    }
}

impl Song {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_stem()?.to_str()?.to_string();
        let extension = path.extension()?.to_str()?.to_string();

        let has_lyrics = path.with_extension("lrc").exists();

        let metadata = match extract_metadata(&path) {
            Ok(meta) => Some(meta),
            Err(e) => {
                warn!("Metadata error: {}", e);
                None
            },
        };

        Some(Self {
            path,
            name,
            extension,
            has_lyrics,
            metadata,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn extension(&self) -> &str {
        &self.extension
    }
    pub fn has_lyrics(&self) -> bool {
        self.has_lyrics
    }
    pub fn metadata(&self) -> Option<&AudioMetadata> {
        self.metadata.as_ref()
    }
}
