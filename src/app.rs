use eframe::egui;
use enum_cycling::EnumCycle;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};
use crate::config::AppConfig;
use crate::library::{scanner, storage, Song};
use crate::lrc::{self, LrcEvent};
use crate::network::downloader::Downloader;
use crate::ui::{panels, settings::SettingsState, theme::Theme, widgets};
use std::collections::HashMap;

#[derive(Default)]
pub struct AppState {
    pub song_pagination: usize,
    pub thumbnail_texture_cache: HashMap<PathBuf, egui::TextureHandle>,
    pub fps_smooth: f32,
    pub last_frame_time: Option<std::time::Instant>,
    pub textures_loaded_this_frame: usize,
}

pub struct Audio {
    audio_player: AudioPlayer,
    is_playing: bool,
    current_file: Option<PathBuf>,
    error_message: Option<String>,
    song_duration: Option<std::time::Duration>,
}

pub struct UI {
    theme: Theme,
    current_view: AppView,
}

pub struct Library {
    library: Vec<Song>,
    library_path: Option<PathBuf>,
    library_filter: String,
    metadata: storage::LibraryMetadata,
    library_dir: Option<PathBuf>,
    add_song_path_input: String,
}

pub struct Karaoke {
    lyrics: Vec<LyricLine>,
    current_line_index: Option<usize>,
    lrc_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadState {
    pub is_downloading: bool,
    pub current_index: usize,
    pub total_count: usize,
    pub current_song: String,
    pub status_message: String,
}

pub struct NetworkState {
    pub downloader: Downloader,
    pub download_tx: Option<Sender<DownloadMessage>>,
    pub download_rx: Option<Receiver<DownloadMessage>>,
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    Started {
        total: usize,
    },
    Progress {
        index: usize,
        song: String,
        status: String,
    },
    Completed,
    Error(String),
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            is_downloading: false,
            current_index: 0,
            total_count: 0,
            current_song: String::new(),
            status_message: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}

pub struct KaraokeApp {
    settings_state: SettingsState,
    app_state: AppState,
    ui: UI,
    audio: Audio,
    library: Library,
    karaoke: Karaoke,
    download_state: DownloadState,
    network_state: NetworkState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Library,
    Karaoke,
    Settings,
}

impl Audio {
    pub fn new() -> Self {
        let audio_player: AudioPlayer = AudioPlayer::new();

        if audio_player.is_available() {
            info!("Audio player initialized successfully");
        } else {
            warn!("Audio player initialized without audio support");
        }
        Self {
            audio_player,
            is_playing: false,
            current_file: None,
            error_message: None,
            song_duration: None,
        }
    }

    #[allow(dead_code)]
    fn toggle_playback(&mut self) {
        if self.is_playing {
            self.audio_player.pause();
            self.is_playing = false;
            info!("Playback paused");
        } else if self.audio_player.is_paused() {
            self.audio_player.resume();
            self.is_playing = true;
            info!("Playback resumed");
        }
    }

    #[allow(dead_code)]
    fn stop_audio(&mut self) {
        self.audio_player.stop();
        self.is_playing = false;
        self.current_file = None;
        info!("Audio stopped");
    }

    #[allow(dead_code)]
    fn play_beep(&mut self) {
        if let Some(sink) = self.audio_player.sink() {
            info!("Playing test sound");

            if let Some(source) = generator::create_beep(440.0, 200) {
                sink.append(source);
                self.is_playing = true;
            }
        }
    }
    fn load_and_play_file(&mut self, path: PathBuf) {
        self.error_message = None;

        // Clear old audio state
        self.audio_player.clear();

        self.song_duration = loader::get_audio_duration(&path);

        match loader::load_audio_file(&path) {
            Ok(decoder) => {
                if self.audio_player.is_available() {
                    self.audio_player.clear();

                    if let Some(sink) = self.audio_player.sink() {
                        sink.append(decoder);
                        sink.play();
                    }

                    self.audio_player.start_tracking();

                    self.current_file = Some(path);
                    self.is_playing = true;
                    info!("Started playback");
                }
            },
            Err(e) => {
                error!("Failed to load file: {}", e);
                self.error_message = Some(loader::format_load_error(&e));
            },
        }
    }
    #[allow(dead_code)]
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            info!("Selected file: {}", path.display());
            self.load_and_play_file(path);
        }
    }
}

impl UI {
    pub fn new() -> Self {
        Self {
            theme: Theme::Tekkadan,
            current_view: AppView::Library,
        }
    }
}

impl Library {
    pub fn new() -> Self {
        // Load library metadata and scan the library directory
        let mut metadata = storage::load_library_metadata();
        let library_dir = storage::get_library_directory().ok();

        // Sync library with actual files on disk
        if let Err(e) = storage::sync_library(&mut metadata) {
            error!("Failed to sync library: {}", e);
        } else {
            // Save the synced metadata
            if let Err(e) = storage::save_library_metadata(&metadata) {
                error!("Failed to save synced library metadata: {}", e);
            }
        }

        let mut lib = Self {
            library: Vec::new(),
            library_path: None,
            library_filter: String::new(),
            metadata,
            library_dir: library_dir.clone(),
            add_song_path_input: String::new(),
        };

        // Scan the library directory on startup
        if let Some(dir) = library_dir {
            lib.load_library_from_storage(&dir);
        }

        lib
    }

    /// Loads songs from the persistent library storage
    fn load_library_from_storage(&mut self, library_dir: &PathBuf) {
        info!("Loading library from storage: {}", library_dir.display());
        self.library = scanner::scan_directory(library_dir);
        self.library_path = Some(library_dir.clone());
        info!("Library loaded with {} songs", self.library.len());

        // Log lyrics status
        let with_lyrics = self.library.iter().filter(|s| s.has_lyrics).count();
        if with_lyrics > 0 {
            info!("{} songs have lyrics files", with_lyrics);
        }
    }

    /// Refreshes the library by syncing metadata and rescanning
    fn refresh_library(&mut self) {
        info!("Refreshing library...");

        // Sync metadata with file system
        if let Err(e) = storage::sync_library(&mut self.metadata) {
            error!("Failed to sync library: {}", e);
        } else {
            // Save the synced metadata
            if let Err(e) = storage::save_library_metadata(&self.metadata) {
                error!("Failed to save synced library metadata: {}", e);
            }
        }

        // Rescan the library directory
        if let Some(dir) = self.library_dir.clone() {
            self.load_library_from_storage(&dir);
        }

        info!("Library refresh complete");
    }

    /// Adds a file to the persistent library storage
    fn add_to_library(&mut self, source_path: PathBuf) {
        match storage::copy_to_library(&source_path) {
            Ok(stored_filename) => {
                let title = source_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let entry = storage::LibraryEntry {
                    original_path: source_path.clone(),
                    stored_filename: stored_filename.clone(),
                    title,
                    added_date: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };

                self.metadata.add_entry(entry);

                if let Err(e) = storage::save_library_metadata(&self.metadata) {
                    error!("Failed to save library metadata: {}", e);
                }

                // Rescan the library
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
    fn remove_from_library(&mut self, song: &Song) {
        // Find the metadata entry for this song
        let stored_filename = song.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if let Some(entry) = self.metadata.remove_entry(stored_filename) {
            if let Err(e) = storage::remove_from_library(&entry.stored_filename) {
                error!("Failed to remove file from library: {}", e);
            }

            if let Err(e) = storage::save_library_metadata(&self.metadata) {
                error!("Failed to save library metadata: {}", e);
            }

            // Rescan the library
            if let Some(dir) = self.library_dir.clone() {
                self.load_library_from_storage(&dir);
            }

            info!("Removed {} from library", entry.title);
        }
    }

    /// Opens a file dialog to add a song to the library
    fn add_song_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Add Song to Library")
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            self.add_to_library(path);
        }
    }

    /// Adds a song from a path string input
    fn add_song_from_path(&mut self) {
        if !self.add_song_path_input.is_empty() {
            let path = PathBuf::from(&self.add_song_path_input);
            if path.exists() && path.is_file() {
                self.add_to_library(path);
                self.add_song_path_input.clear();
            } else {
                error!("Invalid file path: {}", self.add_song_path_input);
            }
        }
    }
}

impl Karaoke {
    pub fn new() -> Self {
        Self {
            lyrics: Vec::new(),
            current_line_index: None,
            lrc_error: None,
        }
    }

    /// Loads LRC file for the given audio file
    pub fn load_lyrics(&mut self, audio_path: &PathBuf) {
        self.lyrics.clear();
        self.current_line_index = None;
        self.lrc_error = None;

        // Try to find LRC file with same name as audio file
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
                self.lrc_error = Some(format!("Failed to parse lyrics:\n{}", e));
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
                    // For simple LRC files, use the first timestamp
                    if let Some(first_ts) = timestamps.first() {
                        // Combine all segment texts
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

        // Sort lyrics by timestamp
        self.lyrics.sort_by_key(|line| line.timestamp_ms);

        info!("Loaded {} lyric lines", self.lyrics.len());

        // Log first few timestamps for debugging
        if self.lyrics.len() > 0 {
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

        // Find the active line (last line with timestamp <= current position)
        let mut active_index = None;
        for (i, line) in self.lyrics.iter().enumerate() {
            if line.timestamp_ms <= current_position_ms {
                active_index = Some(i);
            } else {
                break;
            }
        }

        // Only log when index changes
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

impl KaraokeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        let audio_player: AudioPlayer = AudioPlayer::new();

        if audio_player.is_available() {
            info!("PWE Karaoke initialized successfully with audio");
        } else {
            warn!("PWE Karaoke initialized without audio support");
        }

        let settings_state = SettingsState::default();
        let download_path = settings_state
            .config
            .network
            .download_path
            .clone()
            .or_else(|| storage::get_library_directory().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("downloads"));

        Self {
            app_state: AppState::default(),
            settings_state,
            audio: Audio::new(),
            ui: UI::new(),
            library: Library::new(),
            karaoke: Karaoke::new(),
            download_state: DownloadState::default(),
            network_state: NetworkState {
                downloader: Downloader::new(download_path),
                download_tx: None,
                download_rx: None,
            },
        }
    }

    fn setup_fonts(ctx: &egui::Context) {
        #[allow(unused_mut)]
        let mut fonts = egui::FontDefinitions::default();

        #[cfg(feature = "custom-font")]
        {
            fonts.font_data.insert(
                "CaskaydiaMono".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/CaskaydiaMono.ttf")),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "CaskaydiaMono".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "CaskaydiaMono".to_owned());
        }

        #[cfg(not(feature = "custom-font"))]
        {
            info!("Using default egui fonts (custom font not enabled)");
        }

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reset per-frame counters
        self.app_state.textures_loaded_this_frame = 0;

        // Calculate FPS
        let now = std::time::Instant::now();
        if let Some(last_time) = self.app_state.last_frame_time {
            let frame_time = now.duration_since(last_time).as_secs_f32();
            if frame_time > 0.0 {
                let fps = 1.0 / frame_time;
                // Smooth FPS with exponential moving average
                self.app_state.fps_smooth = if self.app_state.fps_smooth > 0.0 {
                    self.app_state.fps_smooth * 0.9 + fps * 0.1
                } else {
                    fps
                };
            }
        }
        self.app_state.last_frame_time = Some(now);

        // Poll download progress channel
        if let Some(rx) = &self.network_state.download_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    DownloadMessage::Started { total } => {
                        self.download_state.total_count = total;
                        self.download_state.is_downloading = true;
                        info!("📊 Download started: {} videos", total);
                    },
                    DownloadMessage::Progress {
                        index,
                        song,
                        status,
                    } => {
                        self.download_state.current_index = index;
                        self.download_state.current_song = song.clone();
                        self.download_state.status_message = status;
                        info!(
                            "📥 Downloading {}/{}: {}",
                            index, self.download_state.total_count, song
                        );
                    },
                    DownloadMessage::Completed => {
                        self.download_state.is_downloading = false;
                        self.download_state.status_message = "All downloads completed!".to_string();
                        info!("✅ All downloads completed");
                    },
                    DownloadMessage::Error(e) => {
                        self.download_state.is_downloading = false;
                        self.download_state.status_message = format!("Error: {}", e);
                        error!("❌ Download error: {}", e);
                    },
                }
                ctx.request_repaint();
            }
        }

        if self.audio.is_playing {
            if self.audio.audio_player.is_empty() {
                self.audio.is_playing = false;
            } else {
                // Request repaint at reduced rate (20 FPS is enough for smooth lyrics/progress)
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }

        let current_position = self.audio.audio_player.get_position();

        // Update karaoke lyrics sync
        let current_position_ms = current_position.as_millis() as u64;

        // Log position every 2 seconds for debugging
        static mut LAST_LOG_TIME: u64 = 0;
        unsafe {
            if current_position_ms > 0 && current_position_ms / 2000 > LAST_LOG_TIME / 2000 {
                info!(
                    "Playback position: {}ms ({}s)",
                    current_position_ms,
                    current_position_ms / 1000
                );
                LAST_LOG_TIME = current_position_ms;
            }
        }

        self.karaoke.update(current_position_ms);

        // Find the current song in the library
        let current_song = self
            .audio
            .current_file
            .as_ref()
            .and_then(|path| self.library.library.iter().find(|s| &s.path == path));

        let (theme_switched, view_change) = panels::render_top_panel(
            ctx,
            self.ui.theme,
            self.ui.current_view,
            self.app_state.fps_smooth,
        );
        if theme_switched {
            self.ui.theme = self.ui.theme.up();
            info!("Theme switched to {:?}", self.ui.theme);
        }
        if let Some(new_view) = view_change {
            self.ui.current_view = new_view;
            info!("View changed to {:?}", new_view);
        }

        let playback_action = panels::render_bottom_panel(
            ctx,
            self.audio.is_playing,
            current_position,
            self.audio.song_duration,
            self.ui.theme,
            current_song,
        );

        match playback_action {
            panels::PlaybackAction::PlayPause => {
                if self.audio.current_file.is_some() {
                    if self.audio.is_playing {
                        self.audio.audio_player.pause();
                        self.audio.is_playing = false;
                        info!("Playback paused");
                    } else {
                        self.audio.audio_player.resume();
                        self.audio.is_playing = true;
                        info!("Playback resumed");
                    }
                }
            },
            panels::PlaybackAction::Stop => {
                self.audio.audio_player.clear();
                self.audio.is_playing = false;
                self.audio.current_file = None;
                self.karaoke.clear();
                info!("Playback stopped");
            },
            panels::PlaybackAction::SkipForward => {
                // Play next song in library
                if let Some(current_path) = &self.audio.current_file {
                    if let Some(current_index) = self
                        .library
                        .library
                        .iter()
                        .position(|s| &s.path == current_path)
                    {
                        let next_index = if current_index + 1 < self.library.library.len() {
                            current_index + 1
                        } else {
                            0 // Wrap to first song
                        };

                        if self.library.library.len() > 1 || current_index != next_index {
                            let next_song_path = self.library.library[next_index].path.clone();
                            info!("Skipping to next song: {}", next_song_path.display());
                            self.karaoke.clear();
                            self.audio.load_and_play_file(next_song_path.clone());
                            self.karaoke.load_lyrics(&next_song_path);
                        }
                    }
                }
            },
            panels::PlaybackAction::SkipBackward => {
                // Play previous song in library
                if let Some(current_path) = &self.audio.current_file {
                    if let Some(current_index) = self
                        .library
                        .library
                        .iter()
                        .position(|s| &s.path == current_path)
                    {
                        let prev_index = if current_index > 0 {
                            current_index - 1
                        } else {
                            self.library.library.len().saturating_sub(1) // Wrap to last song
                        };

                        if self.library.library.len() > 1 || current_index != prev_index {
                            let prev_song_path = self.library.library[prev_index].path.clone();
                            info!("Skipping to previous song: {}", prev_song_path.display());
                            self.karaoke.clear();
                            self.audio.load_and_play_file(prev_song_path.clone());
                            self.karaoke.load_lyrics(&prev_song_path);
                        }
                    }
                }
            },
            panels::PlaybackAction::Seek(ratio) => {
                if let Some(duration) = self.audio.song_duration {
                    let new_position = Duration::from_secs_f32(duration.as_secs_f32() * ratio);
                    self.seek_to_position(new_position);
                    info!("Seeked to {:?} ({}%)", new_position, (ratio * 100.0) as i32);
                }
            },
            panels::PlaybackAction::None => {},
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.ui.theme.background())
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui: &mut egui::Ui| match self.ui.current_view {
                AppView::Library => self.render_library_view(ui),
                AppView::Karaoke => self.render_karaoke_view(ui),
                AppView::Settings => self.render_settings_view(ui),
            });
    }
}

impl KaraokeApp {
    fn render_library_view(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.library.library_filter)
                            .desired_width(170.0)
                            .hint_text("Search..."),
                    );
                    ui.label(
                        egui::RichText::new("Q")
                            .color(self.ui.theme.text_muted())
                            .size(11.0),
                    );
                });

                ui.add_space(16.0);

                render_sidebar_section(
                    ui,
                    self.ui.theme,
                    "MY LIBRARY",
                    &["All Songs", "Favorites", "History"],
                );

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("PLAYLISTS")
                        .color(self.ui.theme.text_muted())
                        .size(11.0)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("(to be implemented)")
                        .color(self.ui.theme.text_muted())
                        .size(10.0)
                        .italics(),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                ui.set_max_height(ui.available_height());

                let library_action = widgets::render_library_section(
                    ui,
                    &self.library.library,
                    self.library.library_path.as_deref(),
                    &mut self.library.library_filter,
                    &mut self.library.add_song_path_input,
                    &mut self.app_state,
                    self.ui.theme,
                );

                match library_action {
                    widgets::LibraryAction::PlaySong(path) => {
                        // Clear old lyrics first
                        self.karaoke.clear();
                        // Load and play the audio file
                        self.audio.load_and_play_file(path.clone());
                        // Try to load lyrics for the new song
                        self.karaoke.load_lyrics(&path);
                    },
                    widgets::LibraryAction::AddSong => self.library.add_song_dialog(),
                    widgets::LibraryAction::AddSongFromPath => self.library.add_song_from_path(),
                    widgets::LibraryAction::RemoveSong(path) => {
                        // Find the song by path and remove it
                        if let Some(song) = self
                            .library
                            .library
                            .iter()
                            .find(|s| s.path == path)
                            .cloned()
                        {
                            self.library.remove_from_library(&song);
                        }
                    },
                    widgets::LibraryAction::RefreshLibrary => {
                        self.library.refresh_library();
                    },
                    widgets::LibraryAction::None => {},
                }
            });
        });
        ui.add_space(8.0);
    }

    fn render_karaoke_view(&mut self, ui: &mut egui::Ui) {
        // Show current song info at the top
        if let Some(song_name) = self
            .audio
            .current_file
            .as_ref()
            .and_then(|path| path.file_stem().and_then(|s| s.to_str()))
        {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Now Playing:")
                        .color(self.ui.theme.text_muted())
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(song_name)
                        .color(self.ui.theme.text_primary())
                        .size(14.0)
                        .strong(),
                );
            });
            ui.add_space(8.0);
        }

        if self.karaoke.lyrics.is_empty() {
            // No lyrics loaded - show placeholder
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);

                    if let Some(error) = &self.karaoke.lrc_error {
                        ui.label(
                            egui::RichText::new("⚠️")
                                .size(48.0)
                                .color(self.ui.theme.text_muted()),
                        );
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(error)
                                .size(16.0)
                                .color(self.ui.theme.text_muted()),
                        );
                    } else if self.audio.current_file.is_none() {
                        ui.label(
                            egui::RichText::new("🎤")
                                .size(64.0)
                                .color(self.ui.theme.text_muted()),
                        );
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("No song playing")
                                .size(24.0)
                                .color(self.ui.theme.text_muted()),
                        );

                        ui.add_space(16.0);

                        ui.label(
                            egui::RichText::new("Go to Library and play a song with a .lrc file")
                                .size(14.0)
                                .color(self.ui.theme.text_muted())
                                .italics(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("No lyrics available")
                                .size(24.0)
                                .color(self.ui.theme.text_muted()),
                        );

                        ui.add_space(16.0);

                        if let Some(audio_path) = &self.audio.current_file {
                            let lrc_path = audio_path.with_extension("lrc");
                            ui.label(
                                egui::RichText::new(format!(
                                    "Create a .lrc file at:\n{}",
                                    lrc_path.display()
                                ))
                                .size(12.0)
                                .color(self.ui.theme.text_muted())
                                .italics(),
                            );
                        }
                    }
                });
            });
            return;
        }

        // Display lyrics in karaoke style with auto-scroll
        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden);

        // Auto-scroll to current line
        if let Some(current_idx) = self.karaoke.current_line_index {
            // Calculate approximate position of current line
            // Each line takes approximately 60px (12px spacing + 28-48px text height)
            let line_height = 60.0;
            let target_offset = (current_idx as f32 * line_height).max(0.0);

            scroll_area = scroll_area.vertical_scroll_offset(target_offset);
        }

        scroll_area.show(ui, |ui| {
            ui.add_space(100.0); // Top padding for visual balance

            let current_index = self.karaoke.current_line_index;

            for (i, line) in self.karaoke.lyrics.iter().enumerate() {
                let is_current = current_index == Some(i);
                let is_upcoming = current_index.map(|idx| i == idx + 1).unwrap_or(false);
                let is_past = current_index.map(|idx| i < idx).unwrap_or(false);

                // Style based on line state with reduced opacity
                let (color, size, strong) = if is_current {
                    (self.ui.theme.accent(), 28.0, true)
                } else if is_upcoming {
                    (self.ui.theme.primary().gamma_multiply(0.7), 22.0, false)
                } else if is_past {
                    (self.ui.theme.text_muted().gamma_multiply(0.4), 18.0, false)
                } else {
                    (self.ui.theme.text_muted().gamma_multiply(0.25), 16.0, false)
                };

                let mut text = egui::RichText::new(&line.text).color(color).size(size);

                if strong {
                    text = text.strong();
                }

                ui.centered_and_justified(|ui| {
                    ui.label(text);
                });

                ui.add_space(16.0); // Consistent spacing between lines
            }

            ui.add_space(200.0);
        });
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        if let Some(action) = crate::ui::settings::render_settings_panel(
            ui,
            self.ui.theme,
            &mut self.settings_state,
            &self.download_state,
        ) {
            self.handle_settings_action(action);
        }
    }

    fn handle_settings_action(&mut self, action: crate::ui::settings::SettingsAction) {
        use crate::ui::settings::SettingsAction;

        match action {
            SettingsAction::SaveConfig => match self.settings_state.config.save() {
                Ok(()) => {
                    info!("Configuration saved successfully");
                },
                Err(e) => {
                    error!("Failed to save configuration: {}", e);
                },
            },
            SettingsAction::ResetConfig => {
                self.settings_state.config = AppConfig::default();
                info!("Configuration reset to factory defaults");
            },
            SettingsAction::RescanLibrary => {
                // Use the same refresh logic as the main page
                self.library.refresh_library();
            },
            SettingsAction::DownloadYouTubePlaylist => {
                let url = self
                    .settings_state
                    .config
                    .network
                    .youtube_playlist_url
                    .clone();
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                info!("🎬 YouTube Playlist Download Started");
                info!("URL: {}", url);
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                if !self.network_state.downloader.is_available() {
                    error!("❌ yt-dlp is not installed!");
                    warn!("💡 Install with: pip install yt-dlp");
                    self.download_state.status_message = "Error: yt-dlp not found".to_string();
                    return;
                }

                // Parse playlist URL to extract video IDs
                if let Some(playlist_id) = Self::extract_youtube_playlist_id(&url) {
                    info!("📋 Playlist ID: {}", playlist_id);
                    self.download_state.is_downloading = true;
                    self.download_state.current_index = 0;
                    self.download_state.total_count = 0; // Will be updated
                    self.download_state.current_song = "Extracting playlist info...".to_string();
                    self.download_state.status_message = "Initializing download...".to_string();

                    // Spawn async download task
                    self.start_youtube_playlist_download(playlist_id);
                } else {
                    error!("❌ Invalid YouTube playlist URL");
                    self.download_state.status_message = "Error: Invalid playlist URL".to_string();
                }
            },
            SettingsAction::DownloadSpotifyPlaylist => {
                let url = self
                    .settings_state
                    .config
                    .network
                    .spotify_playlist_url
                    .clone();
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                info!("🎵 Spotify Playlist Download Started");
                info!("URL: {}", url);
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                if !self.network_state.downloader.is_available() {
                    error!("❌ yt-dlp is not installed!");
                    warn!("💡 Install with: pip install yt-dlp");
                    self.download_state.status_message = "Error: yt-dlp not found".to_string();
                    return;
                }

                self.download_state.is_downloading = true;
                self.download_state.current_index = 0;
                self.download_state.total_count = 0;
                self.download_state.current_song = "Fetching Spotify playlist...".to_string();
                self.download_state.status_message = "Initializing...".to_string();

                // Spawn async download task
                self.start_spotify_playlist_download(url);
            },
        }
    }

    /// Extract YouTube playlist ID from URL
    fn extract_youtube_playlist_id(url: &str) -> Option<String> {
        // Handle URLs like:
        // - https://www.youtube.com/playlist?list=PLxxxxxx
        // - https://youtube.com/playlist?list=PLxxxxxx
        if let Some(start) = url.find("list=") {
            let id_start = start + 5;
            let id_end = url[id_start..]
                .find('&')
                .map(|pos| id_start + pos)
                .unwrap_or(url.len());
            Some(url[id_start..id_end].to_string())
        } else {
            None
        }
    }

    /// Start YouTube playlist download (simplified synchronous version)
    fn start_youtube_playlist_download(&mut self, playlist_id: String) {
        info!("🚀 Starting download for playlist: {}", playlist_id);

        // Create channel for progress updates
        let (tx, rx) = channel();
        self.network_state.download_tx = Some(tx.clone());
        self.network_state.download_rx = Some(rx);

        let url = format!("https://www.youtube.com/playlist?list={}", playlist_id);
        let downloader = self.network_state.downloader.clone();

        // Spawn background thread for downloads
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                // Get playlist videos
                match downloader.get_playlist_videos(&url).await {
                    Ok(videos) => {
                        let total = videos.len();
                        let _ = tx.send(DownloadMessage::Started { total });

                        for (idx, (video_id, title)) in videos.iter().enumerate() {
                            let _ = tx.send(DownloadMessage::Progress {
                                index: idx + 1,
                                song: title.clone(),
                                status: "Downloading...".to_string(),
                            });

                            match downloader.download_youtube_video(video_id).await {
                                Ok(path) => {
                                    info!("✅ Downloaded: {}", path.display());
                                },
                                Err(e) => {
                                    error!("❌ Failed to download {}: {}", title, e);
                                },
                            }
                        }

                        let _ = tx.send(DownloadMessage::Completed);
                    },
                    Err(e) => {
                        error!("❌ Failed to fetch playlist: {}", e);
                        let _ = tx.send(DownloadMessage::Error(e));
                    },
                }
            });
        });
    }

    /// Start Spotify playlist download
    fn start_spotify_playlist_download(&mut self, playlist_url: String) {
        info!("🚀 Starting Spotify download for: {}", playlist_url);

        // Create channel for progress updates
        let (tx, rx) = channel();
        self.network_state.download_tx = Some(tx.clone());
        self.network_state.download_rx = Some(rx);

        let downloader = self.network_state.downloader.clone();

        // Spawn background thread for downloads
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                // Get Spotify playlist tracks
                match downloader.get_spotify_playlist_tracks(&playlist_url).await {
                    Ok(tracks) => {
                        let total = tracks.len();
                        let _ = tx.send(DownloadMessage::Started { total });

                        for (idx, (title, artist)) in tracks.iter().enumerate() {
                            let track_info = format!("{} - {}", title, artist);
                            let _ = tx.send(DownloadMessage::Progress {
                                index: idx + 1,
                                song: track_info.clone(),
                                status: "Searching on YouTube...".to_string(),
                            });

                            // Download using YouTube search
                            match downloader.download_spotify_track(title, artist).await {
                                Ok(path) => {
                                    info!("✅ Downloaded: {}", path.display());
                                },
                                Err(e) => {
                                    error!("❌ Failed to download {}: {}", track_info, e);
                                },
                            }
                        }

                        let _ = tx.send(DownloadMessage::Completed);
                    },
                    Err(e) => {
                        error!("❌ Failed to fetch Spotify playlist: {}", e);
                        let _ = tx.send(DownloadMessage::Error(e));
                    },
                }
            });
        });
    }
}

fn render_sidebar_section(ui: &mut egui::Ui, theme: Theme, title: &str, items: &[&str]) {
    ui.label(
        egui::RichText::new(title)
            .color(theme.text_muted())
            .size(11.0)
            .strong(),
    );

    ui.add_space(8.0);

    for item in items {
        if ui
            .button(
                egui::RichText::new(format!("> {}", item))
                    .color(theme.text_muted())
                    .size(12.0),
            )
            .clicked()
        {
            info!("Sidebar item '{}' clicked - to be implemented", item);
        }
    }
}

impl KaraokeApp {
    fn seek_to_position(&mut self, position: std::time::Duration) {
        if let Some(path) = self.audio.current_file.clone() {
            let was_playing = self.audio.is_playing;

            // Clear current playback
            self.audio.audio_player.clear();

            match loader::load_audio_file(&path) {
                Ok(decoder) => {
                    // Note: Rodio doesn't support seeking in most formats
                    // We just reload from the beginning for now
                    // The position tracker is updated to maintain karaoke sync

                    if let Some(sink) = self.audio.audio_player.sink() {
                        sink.append(decoder);

                        if was_playing {
                            sink.play();
                        } else {
                            sink.pause();
                        }

                        // Update the position tracker
                        self.audio.audio_player.set_position(position);

                        info!("Seeked to position: {:?}", position);
                    }
                },
                Err(e) => {
                    error!("Failed to seek: {}", e);
                },
            }
        }
    }
}
