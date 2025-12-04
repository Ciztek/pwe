use eframe::egui;
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};
use crate::library::{scanner, Song};
use crate::ui::{panels, theme::Theme, widgets};

pub struct KaraokeApp {
    // UI State
    counter: i32,
    user_text: String,
    theme: Theme,

    // Audio
    audio_player: AudioPlayer,
    is_playing: bool,
    current_file: Option<PathBuf>,
    error_message: Option<String>,
    song_duration: Option<std::time::Duration>,

    // Library
    library: Vec<Song>,
    library_path: Option<PathBuf>,
    library_filter: String,
}

impl KaraokeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let audio_player: AudioPlayer = AudioPlayer::new();

        if audio_player.is_available() {
            info!("PWE Karaoke initialized successfully with audio");
        } else {
            warn!("PWE Karaoke initialized without audio support");
        }

        Self {
            counter: 0,
            user_text: String::from("Hello, PWE Karaoke!"),
            theme: Theme::IronFlower,
            audio_player,
            is_playing: false,
            current_file: None,
            error_message: None,
            song_duration: None,
            library: Vec::new(),
            library_path: None,
            library_filter: String::new(),
        }
    }

    fn scan_library(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Library Folder")
            .pick_folder()
        {
            info!("Scanning library: {}", path.display());
            self.library = scanner::scan_directory(&path);
            self.library_path = Some(path);
            info!("Library loaded with {} songs", self.library.len());
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            info!("Selected file: {}", path.display());
            self.load_and_play_file(path);
        }
    }

    /// Loads an audio file and starts playback.
    ///
    /// # Parameters
    /// - `path`: Path to audio file (must exist and be a valid format)
    ///
    /// # Behavior
    /// - Attempts to read duration metadata
    /// - Loads file with audio decoder
    /// - Starts playback immediately
    /// - Sets error_message if loading fails
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

    fn stop_audio(&mut self) {
        self.audio_player.stop();
        self.is_playing = false;
        self.current_file = None;
        info!("Audio stopped");
    }

    fn play_beep(&mut self) {
        if let Some(sink) = self.audio_player.sink() {
            info!("Playing test sound");

            if let Some(source) = generator::create_beep(440.0, 200) {
                sink.append(source);
                self.is_playing = true;
            }
        }
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_playing && self.audio_player.is_empty() {
            self.is_playing = false;
        }

        let current_position = self.audio_player.get_position();

        let theme_switched = panels::render_top_panel(ctx, self.theme);
        if theme_switched {
            self.theme = self.theme.toggle();
            info!("Theme switched to {:?}", self.theme);
        }
        panels::render_bottom_panel(
            ctx,
            self.is_playing,
            current_position,
            self.song_duration,
            self.theme,
        );

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let audio_action = widgets::render_file_playback_section(
                ui,
                self.is_playing,
                self.current_file.as_deref(),
                self.error_message.as_deref(),
                self.theme,
            );

            match audio_action {
                widgets::AudioAction::OpenFile => self.open_file(),
                widgets::AudioAction::PlayPause => self.toggle_playback(),
                widgets::AudioAction::Stop => self.stop_audio(),
                widgets::AudioAction::Play => {}, // Not used in file playback section
                widgets::AudioAction::None => {},
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            let library_action = widgets::render_library_section(
                ui,
                &self.library,
                self.library_path.as_deref(),
                &mut self.library_filter,
                self.theme,
            );

            match library_action {
                widgets::LibraryAction::ScanFolder => self.scan_library(),
                widgets::LibraryAction::PlaySong(path) => self.load_and_play_file(path),
                widgets::LibraryAction::None => {},
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            widgets::render_text_section(ui, &mut self.user_text);
            widgets::render_counter_section(ui, &mut self.counter);

            let beep_action = widgets::render_audio_section(ui, self.is_playing);
            match beep_action {
                widgets::AudioAction::Play => self.play_beep(),
                widgets::AudioAction::Stop => self.stop_audio(),
                _ => {},
            }

            widgets::render_info_section(ui);
        });
    }
}
