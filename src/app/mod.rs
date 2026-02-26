// Main application module - coordinates all submodules

pub mod audio;
pub mod frame;
pub mod helpers;
pub mod karaoke;
pub mod library;
pub mod network;
pub mod playback;
pub mod state;
pub mod views;

use eframe::egui;
use tracing::info;

use crate::ui::settings::SettingsState;

pub use audio::Audio;
pub use karaoke::Karaoke;
pub use library::Library;
pub use network::{DownloadMessage, DownloadState, NetworkState, TranscriptionState};
pub use state::{AppState, AppView, LibraryViewFilter, UI};

pub struct KaraokeApp {
    pub settings_state: SettingsState,
    pub app_state: AppState,
    pub ui: UI,
    pub audio: Audio,
    pub library: Library,
    pub karaoke: Karaoke,
    pub download_state: DownloadState,
    pub transcription_state: TranscriptionState,
    pub network_state: NetworkState,
}

impl KaraokeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        let audio_player = crate::audio::player::AudioPlayer::new();

        if audio_player.is_available() {
            info!("PWE Karaoke initialized successfully with audio");
        } else {
            info!("PWE Karaoke initialized without audio support");
        }

        let settings_state = SettingsState::default();
        let download_path = settings_state
            .config
            .network
            .download_path
            .clone()
            .or_else(|| crate::library::storage::get_library_directory().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("downloads"));

        Self {
            app_state: AppState::default(),
            settings_state,
            audio: Audio::new(),
            ui: UI::new(),
            library: Library::new(),
            karaoke: Karaoke::new(),
            download_state: DownloadState::default(),
            transcription_state: TranscriptionState::default(),
            network_state: NetworkState::new(download_path),
        }
    }

    fn setup_fonts(ctx: &egui::Context) {
        #[allow(unused_mut)]
        let mut fonts = egui::FontDefinitions::default();

        #[cfg(feature = "custom-font")]
        {
            fonts.font_data.insert(
                "CaskaydiaMono".to_owned(),
                egui::FontData::from_static(include_bytes!("../../assets/CaskaydiaMono.ttf")),
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

    pub fn seek_to_position(&mut self, position: std::time::Duration) {
        self.audio.seek_to_position(position);
    }

    pub fn transcribe_lyrics(&mut self, song_path: std::path::PathBuf) {
        helpers::transcribe_lyrics(self, song_path);
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        frame::update_frame(self, ctx);
    }
}
