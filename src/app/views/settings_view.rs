// Settings view rendering

use eframe::egui;
use tracing::{error, info};

use crate::app::KaraokeApp;
use crate::config::AppConfig;
use crate::ui::actions::SettingsAction;

pub fn render_settings_view(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    if let Some(action) = crate::ui::settings::render_settings_panel(
        ui,
        app.ui.theme,
        &mut app.settings_state,
        &app.download_state,
    ) {
        handle_settings_action(app, action);
    }
}

fn handle_settings_action(app: &mut KaraokeApp, action: SettingsAction) {
    match action {
        SettingsAction::SaveConfig => match app.settings_state.config.save() {
            Ok(()) => {
                info!("Configuration saved successfully");
            },
            Err(e) => {
                error!("Failed to save configuration: {}", e);
            },
        },
        SettingsAction::ResetConfig => {
            app.settings_state.config = AppConfig::default();
            info!("Configuration reset to factory defaults");
        },
        SettingsAction::RescanLibrary => {
            app.library.refresh_library();
        },
        SettingsAction::DownloadYouTubePlaylist => {
            let url = app
                .settings_state
                .config
                .network
                .youtube_playlist_url
                .clone();
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            info!("🎬 YouTube Playlist Download Started");
            info!("URL: {}", url);
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            if !app.network_state.downloader.is_available() {
                error!("❌ yt-dlp is not installed!");
                error!("💡 Install with: pip install yt-dlp");
                app.download_state.status_message = "Error: yt-dlp not found".to_string();
                return;
            }

            if let Some(playlist_id) =
                crate::app::network::NetworkState::extract_youtube_playlist_id(&url)
            {
                info!("📋 Playlist ID: {}", playlist_id);
                app.download_state.is_downloading = true;
                app.download_state.current_index = 0;
                app.download_state.total_count = 0;
                app.download_state.current_song = "Extracting playlist info...".to_string();
                app.download_state.status_message = "Initializing download...".to_string();

                app.network_state
                    .start_youtube_playlist_download(&playlist_id);
            } else {
                error!("❌ Invalid YouTube playlist URL");
                app.download_state.status_message = "Error: Invalid playlist URL".to_string();
            }
        },
        SettingsAction::DownloadSpotifyPlaylist => {
            let url = app
                .settings_state
                .config
                .network
                .spotify_playlist_url
                .clone();
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            info!("🎵 Spotify Playlist Download Started");
            info!("URL: {}", url);
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            if !app.network_state.downloader.is_available() {
                error!("❌ yt-dlp is not installed!");
                error!("💡 Install with: pip install yt-dlp");
                app.download_state.status_message = "Error: yt-dlp not found".to_string();
                return;
            }

            app.download_state.is_downloading = true;
            app.download_state.current_index = 0;
            app.download_state.total_count = 0;
            app.download_state.current_song = "Fetching Spotify playlist...".to_string();
            app.download_state.status_message = "Initializing...".to_string();

            app.network_state.start_spotify_playlist_download(url);
        },
    }
}
