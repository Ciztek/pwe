use anyhow::{Context, Result};
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use tracing::info;

/// Audio file metadata extracted from tags
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u32>,
    pub total_tracks: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub duration_secs: Option<u64>,
    pub lyrics: Option<String>,
    pub cover_art: Option<Vec<u8>>, // Album art as raw bytes
}

impl AudioMetadata {
    /// Gets a display name prioritizing: title > filename
    #[allow(dead_code)]
    pub fn display_name(&self, fallback_filename: &str) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| fallback_filename.to_string())
    }

    /// Gets a display artist or "Unknown Artist"
    #[allow(dead_code)]
    pub fn display_artist(&self) -> String {
        self.artist
            .clone()
            .or_else(|| self.album_artist.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string())
    }

    /// Gets a display album or "Unknown Album"
    #[allow(dead_code)]
    pub fn display_album(&self) -> String {
        self.album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string())
    }

    /// Formats track number as "01" or "01/12" if total is known
    #[allow(dead_code)]
    pub fn display_track_number(&self) -> Option<String> {
        self.track_number.map(|num| {
            if let Some(total) = self.total_tracks {
                format!("{:02}/{:02}", num, total)
            } else {
                format!("{:02}", num)
            }
        })
    }
}

/// Extracts metadata from an audio file using Symphonia
#[allow(dead_code)]
pub fn extract_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
    let path = path.as_ref();

    info!("Extracting metadata from: {}", path.display());

    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open audio file: {}", path.display()))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    let metadata_opts = MetadataOptions::default();

    let mut probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Failed to probe audio file")?;

    let mut metadata = AudioMetadata::default();

    // Get duration from the format
    if let Some(track) = probed.format.default_track() {
        if let Some(time_base) = track.codec_params.time_base {
            if let Some(n_frames) = track.codec_params.n_frames {
                let duration = time_base.calc_time(n_frames);
                metadata.duration_secs = Some(duration.seconds);
            }
        }
    }

    // Extract metadata tags from probe metadata (ID3/container level)
    if let Some(probe_meta) = probed.metadata.get().as_ref() {
        if let Some(metadata_rev) = probe_meta.current() {
            for tag in metadata_rev.tags() {
                match tag.std_key {
                    Some(StandardTagKey::TrackTitle) => {
                        metadata.title = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::Artist) => {
                        metadata.artist = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::Album) => {
                        metadata.album = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::AlbumArtist) => {
                        metadata.album_artist = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::TrackNumber) => {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.track_number = Some(num);
                        }
                    },
                    Some(StandardTagKey::TrackTotal) => {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.total_tracks = Some(num);
                        }
                    },
                    Some(StandardTagKey::DiscNumber) => {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.disc_number = Some(num);
                        }
                    },
                    Some(StandardTagKey::Genre) => {
                        metadata.genre = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::Date) | Some(StandardTagKey::ReleaseDate) => {
                        metadata.date = Some(tag.value.to_string());
                    },
                    Some(StandardTagKey::Lyrics) => {
                        metadata.lyrics = Some(tag.value.to_string());
                    },
                    _ => {},
                }
            }

            // Extract cover art (visual) from probe metadata
            for visual in metadata_rev.visuals() {
                if visual.usage == Some(symphonia::core::meta::StandardVisualKey::FrontCover)
                    || visual.usage.is_none()
                {
                    metadata.cover_art = Some(visual.data.to_vec());
                    info!(
                        "Found cover art: {} bytes, media type: {:?}",
                        visual.data.len(),
                        visual.media_type
                    );
                    break;
                }
            }
        }
    }

    // Also check format-level metadata (stream metadata)
    if let Some(metadata_rev) = probed.format.metadata().current() {
        for tag in metadata_rev.tags() {
            match tag.std_key {
                Some(StandardTagKey::TrackTitle) => {
                    if metadata.title.is_none() {
                        metadata.title = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::Artist) => {
                    if metadata.artist.is_none() {
                        metadata.artist = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::Album) => {
                    if metadata.album.is_none() {
                        metadata.album = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::AlbumArtist) => {
                    if metadata.album_artist.is_none() {
                        metadata.album_artist = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::TrackNumber) => {
                    if metadata.track_number.is_none() {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.track_number = Some(num);
                        }
                    }
                },
                Some(StandardTagKey::TrackTotal) => {
                    if metadata.total_tracks.is_none() {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.total_tracks = Some(num);
                        }
                    }
                },
                Some(StandardTagKey::DiscNumber) => {
                    if metadata.disc_number.is_none() {
                        if let Ok(num) = tag.value.to_string().parse::<u32>() {
                            metadata.disc_number = Some(num);
                        }
                    }
                },
                Some(StandardTagKey::Genre) => {
                    if metadata.genre.is_none() {
                        metadata.genre = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::Date) | Some(StandardTagKey::ReleaseDate) => {
                    if metadata.date.is_none() {
                        metadata.date = Some(tag.value.to_string());
                    }
                },
                Some(StandardTagKey::Lyrics) => {
                    if metadata.lyrics.is_none() {
                        metadata.lyrics = Some(tag.value.to_string());
                    }
                },
                _ => {},
            }
        }

        // Extract cover art (visual) from format metadata if not found yet
        if metadata.cover_art.is_none() {
            for visual in metadata_rev.visuals() {
                if visual.usage == Some(symphonia::core::meta::StandardVisualKey::FrontCover)
                    || visual.usage.is_none()
                {
                    metadata.cover_art = Some(visual.data.to_vec());
                    info!(
                        "Found cover art: {} bytes, media type: {:?}",
                        visual.data.len(),
                        visual.media_type
                    );
                    break;
                }
            }
        }
    }

    info!(
        "Extracted metadata - Title: {:?}, Artist: {:?}, Album: {:?}",
        metadata.title, metadata.artist, metadata.album
    );

    Ok(metadata)
}
