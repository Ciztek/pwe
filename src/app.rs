use eframe::egui;
use enum_cycling::EnumCycle;
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};
use crate::library::{scanner, storage, Song};
use crate::ui::{panels, settings::SettingsState, theme::Theme, widgets};

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

pub struct KaraokeApp {
    settings_state: SettingsState,

    ui: UI,
    audio: Audio,
    library: Library,
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
        let metadata = storage::load_library_metadata();
        let library_dir = storage::get_library_directory().ok();

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
            }
            Err(e) => {
                error!("Failed to add file to library: {}", e);
            }
        }
    }

    /// Removes a song from the persistent library storage
    fn remove_from_library(&mut self, song: &Song) {
        // Find the metadata entry for this song
        let stored_filename = song.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

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

impl KaraokeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        let audio_player: AudioPlayer = AudioPlayer::new();

        if audio_player.is_available() {
            info!("PWE Karaoke initialized successfully with audio");
        } else {
            warn!("PWE Karaoke initialized without audio support");
        }

        Self {
            settings_state: SettingsState::default(),
            audio: Audio::new(),
            ui: UI::new(),
            library: Library::new(),
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
        if self.audio.is_playing {
            ctx.request_repaint();
            if self.audio.audio_player.is_empty() {
                self.audio.is_playing = false;
            }
        }

        let current_position = self.audio.audio_player.get_position();

        let current_song_name = self
            .audio
            .current_file
            .as_ref()
            .and_then(|path| path.file_stem().and_then(|s| s.to_str()));

        let (theme_switched, view_change) =
            panels::render_top_panel(ctx, self.ui.theme, self.ui.current_view);
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
            current_song_name,
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
                info!("Playback stopped");
            },
            panels::PlaybackAction::SkipForward => {
                info!("Skip forward - to be implemented");
            },
            panels::PlaybackAction::SkipBackward => {
                info!("Skip backward - to be implemented");
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
                    self.ui.theme,
                );

                match library_action {
                    widgets::LibraryAction::PlaySong(path) => self.audio.load_and_play_file(path),
                    widgets::LibraryAction::AddSong => self.library.add_song_dialog(),
                    widgets::LibraryAction::AddSongFromPath => self.library.add_song_from_path(),
                    widgets::LibraryAction::RemoveSong(path) => {
                        // Find the song by path and remove it
                        if let Some(song) = self.library.library.iter().find(|s| s.path == path).cloned() {
                            self.library.remove_from_library(&song);
                        }
                    }
                    widgets::LibraryAction::None => {},
                }
            });
        });
        ui.add_space(8.0);
    }

    fn render_karaoke_view(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.label(
                    egui::RichText::new("KARAOKE MODE")
                        .size(32.0)
                        .color(self.ui.theme.primary())
                        .strong(),
                );

                ui.add_space(24.0);

                ui.label(
                    egui::RichText::new("Lyrics display and karaoke HUD")
                        .size(16.0)
                        .color(self.ui.theme.text_muted())
                        .italics(),
                );

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("TO BE IMPLEMENTED")
                        .size(14.0)
                        .color(self.ui.theme.accent())
                        .monospace(),
                );

                ui.add_space(32.0);

                if ui
                    .button(
                        egui::RichText::new("[ Start Karaoke Session ]")
                            .size(16.0)
                            .color(self.ui.theme.primary()),
                    )
                    .clicked()
                {
                    info!("Karaoke session start - to be implemented");
                }
            });
        });
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        crate::ui::settings::render_settings_panel(ui, self.ui.theme, &mut self.settings_state);
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
