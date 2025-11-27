// Main application logic
use eframe::egui;
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};
use crate::ui::{panels, widgets};

pub struct KaraokeApp {
    // UI State
    counter: i32,
    user_text: String,

    // Audio
    audio_player: AudioPlayer,
    is_playing: bool,
    current_file: Option<PathBuf>,
    error_message: Option<String>,
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
            audio_player,
            is_playing: false,
            current_file: None,
            error_message: None,
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

    fn load_and_play_file(&mut self, path: PathBuf) {
        self.error_message = None;

        match loader::load_audio_file(&path) {
            Ok(decoder) => {
                if let Some(sink) = self.audio_player.sink() {
                    // Clear any existing audio
                    self.audio_player.clear();

                    // Add new file to sink
                    sink.append(decoder);

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
        // Update playing state
        if self.is_playing && self.audio_player.is_empty() {
            self.is_playing = false;
        }

        // Render UI panels
        panels::render_top_panel(ctx);
        panels::render_bottom_panel(ctx, self.is_playing, self.counter);

        // Central panel with main content
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let audio_action = widgets::render_file_playback_section(
                ui,
                self.is_playing,
                self.current_file.as_deref(),
                self.error_message.as_deref(),
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
