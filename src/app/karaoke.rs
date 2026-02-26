use tracing::{error, info, warn};

use crate::app::state::LyricLine;
use crate::lrc::{self, LrcEvent};

pub struct Karaoke {
    pub lyrics: Vec<LyricLine>,
    pub current_line_index: Option<usize>,
    pub lrc_error: Option<String>,
}

impl Karaoke {
    pub const fn new() -> Self {
        Self {
            lyrics: Vec::new(),
            current_line_index: None,
            lrc_error: None,
        }
    }

    /// Loads LRC file for the given audio file
    pub fn load_lyrics(&mut self, audio_path: &std::path::Path) {
        self.lyrics.clear();
        self.current_line_index = None;
        self.lrc_error = None;

        let lrc_path = audio_path.with_extension("lrc");
        info!("Looking for LRC file at: {}", lrc_path.display());

        if !lrc_path.exists() {
            info!("No LRC file found at: {}", lrc_path.display());
            self.lrc_error = Some(format!("No lyrics file found at:\n{}", lrc_path.display()));
            return;
        }

        info!("Found LRC file, parsing...");
        match lrc::parse_lrc_file(&lrc_path) {
            Ok(events) => {
                info!("Successfully parsed LRC file with {} events", events.len());
                self.parse_lrc_events(events);

                if self.lyrics.is_empty() {
                    warn!("LRC file contained no lyric lines");
                    self.lrc_error = Some("LRC file contains no lyrics".to_string());
                } else {
                    info!("Loaded {} lyric lines", self.lyrics.len());
                }
            },
            Err(e) => {
                error!("Failed to parse LRC file: {}", e);
                self.lrc_error = Some(format!("Failed to parse lyrics:\n{e}"));
            },
        }
    }

    /// Converts LRC events into lyric lines
    fn parse_lrc_events(&mut self, events: Vec<LrcEvent>) {
        for event in events {
            match event {
                LrcEvent::Lyric {
                    timestamps,
                    segments,
                } => {
                    if let Some(first_ts) = timestamps.first() {
                        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
                        self.lyrics.push(LyricLine {
                            timestamp_ms: first_ts.to_millis(),
                            text,
                        });
                    }
                },
                LrcEvent::Metadata { .. } => {
                    // Ignore metadata for now
                },
            }
        }

        self.lyrics.sort_by_key(|line| line.timestamp_ms);
        info!("Loaded {} lyric lines", self.lyrics.len());

        if !self.lyrics.is_empty() {
            let preview_count = self.lyrics.len().min(5);
            info!("First {} lyrics timestamps:", preview_count);
            for (i, line) in self.lyrics.iter().take(preview_count).enumerate() {
                info!(
                    "  Line {}: {}ms - \"{}\"",
                    i + 1,
                    line.timestamp_ms,
                    line.text.chars().take(30).collect::<String>()
                );
            }
        }
    }

    /// Updates the current line based on playback position
    pub fn update(&mut self, current_position_ms: u64) {
        if self.lyrics.is_empty() {
            return;
        }

        let mut active_index = None;
        for (i, line) in self.lyrics.iter().enumerate() {
            if line.timestamp_ms <= current_position_ms {
                active_index = Some(i);
            } else {
                break;
            }
        }

        if active_index != self.current_line_index {
            if let Some(idx) = active_index {
                if idx < self.lyrics.len() {
                    info!(
                        "Lyrics updated: line {} at {}ms - \"{}\"",
                        idx + 1,
                        current_position_ms,
                        self.lyrics[idx].text.chars().take(50).collect::<String>()
                    );
                }
            }
        }

        self.current_line_index = active_index;
    }

    pub fn clear(&mut self) {
        self.lyrics.clear();
        self.current_line_index = None;
        self.lrc_error = None;
    }
}
