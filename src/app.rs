// Main application logic
use eframe::egui;
use tracing::{info, warn};

use crate::audio::{generator, player::AudioPlayer};
use crate::ui::{panels, widgets};

pub struct KaraokeApp {
    // UI State
    counter: i32,
    user_text: String,

    // Audio
    audio_player: AudioPlayer,
    is_playing: bool,
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
        }
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

    fn stop_audio(&mut self) {
        self.audio_player.stop();
        self.is_playing = false;
        info!("Audio stopped");
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
            widgets::render_text_section(ui, &mut self.user_text);
            widgets::render_counter_section(ui, &mut self.counter);

            let audio_action = widgets::render_audio_section(ui, self.is_playing);
            match audio_action {
                widgets::AudioAction::Play => self.play_beep(),
                widgets::AudioAction::Stop => self.stop_audio(),
                widgets::AudioAction::None => {},
            }

            widgets::render_info_section(ui);
        });
    }
}
