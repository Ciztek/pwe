// Network settings section
use super::super::actions::SettingsAction;
use super::super::components::{button, card, layout};
use super::super::theme::Theme;
use crate::app::DownloadState;
use crate::config::NetworkConfig;
use eframe::egui;

/// Render network settings section
pub fn render(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut NetworkConfig,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    layout::space(ui, layout::SPACE_LG);

    #[allow(clippy::branches_sharing_code)]
    let mut action = if download_state.is_downloading {
        render_download_progress(ui, theme, download_state);
        layout::space(ui, layout::SPACE_LG);
        None
    } else {
        None
    };

    if action.is_none() {
        action = render_playlist_download(ui, theme, config, download_state);
    }

    layout::space(ui, layout::SPACE_LG);
    render_download_settings(ui, theme, config);

    if action.is_none() {
        action = super::action_buttons(ui, theme);
    }

    action
}

fn render_download_progress(ui: &mut egui::Ui, theme: Theme, state: &DownloadState) {
    card::settings_card(ui, theme, "DOWNLOAD IN PROGRESS", |ui, theme| {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                egui::RichText::new(format!(
                    "Downloading: {} / {}",
                    state.current_index, state.total_count
                ))
                .color(theme.accent())
                .strong()
                .size(14.0),
            );
        });

        if !state.current_song.is_empty() {
            layout::space(ui, layout::SPACE_SM);
            layout::info_text(ui, &format!("🎵 {}", state.current_song), theme);
        }

        if !state.status_message.is_empty() {
            layout::space(ui, layout::SPACE_XS);
            layout::hint_text(ui, &state.status_message, theme);
        }

        layout::space(ui, layout::SPACE_SM);

        let progress = if state.total_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let ratio = state.current_index as f32 / state.total_count as f32;
            ratio
        } else {
            0.0
        };

        ui.add(egui::ProgressBar::new(progress).show_percentage());
    });
}

fn render_playlist_download(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut NetworkConfig,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    let mut action = None;

    card::settings_card(ui, theme, "PLAYLIST DOWNLOAD", |ui, theme| {
        layout::hint_text(ui, "Paste public playlist URLs to download songs", theme);
        layout::space(ui, layout::SPACE_MD);

        if let Some(yt_action) = render_youtube_playlist(ui, theme, config, download_state) {
            action = Some(yt_action);
        }

        layout::space(ui, layout::SPACE_MD);

        if let Some(spotify_action) = render_spotify_playlist(ui, theme, config, download_state) {
            action = Some(spotify_action);
        }

        layout::space(ui, layout::SPACE_MD);
        layout::hint_text(
            ui,
            "💡 Tip: Public playlists only. No authentication needed!",
            theme,
        );
    });

    action
}

fn render_youtube_playlist(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut NetworkConfig,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    layout::subsection_label(ui, "YouTube Playlist:", theme);

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut config.youtube_playlist_url)
                .desired_width(400.0)
                .hint_text("https://www.youtube.com/playlist?list=...")
                .interactive(!download_state.is_downloading),
        );

        let enabled = !config.youtube_playlist_url.is_empty() && !download_state.is_downloading;

        if button::StyledButton::new("[ Download ]")
            .style(button::ButtonStyle::Accent)
            .enabled(enabled)
            .show(ui, theme)
            .clicked()
        {
            return Some(SettingsAction::DownloadYouTubePlaylist);
        }

        None
    })
    .inner
}

fn render_spotify_playlist(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut NetworkConfig,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    layout::subsection_label(ui, "Spotify Playlist:", theme);

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut config.spotify_playlist_url)
                .desired_width(400.0)
                .hint_text("https://open.spotify.com/playlist/...")
                .interactive(!download_state.is_downloading),
        );

        let enabled = !config.spotify_playlist_url.is_empty() && !download_state.is_downloading;

        if button::StyledButton::new("[ Download ]")
            .style(button::ButtonStyle::Accent)
            .enabled(enabled)
            .show(ui, theme)
            .clicked()
        {
            return Some(SettingsAction::DownloadSpotifyPlaylist);
        }

        None
    })
    .inner
}

fn render_download_settings(ui: &mut egui::Ui, theme: Theme, config: &mut NetworkConfig) {
    card::settings_card(ui, theme, "DOWNLOAD SETTINGS", |ui, theme| {
        layout::subsection_label(ui, "Download Path:", theme);

        let path_display = config.download_path.as_ref().map_or_else(
            || "📁 Not set (will use first library path)".to_string(),
            |p| format!("📁 {}", p.display()),
        );

        ui.horizontal(|ui| {
            layout::info_text(ui, &path_display, theme);

            if button::StyledButton::new("[ Browse ]")
                .style(button::ButtonStyle::Primary)
                .show(ui, theme)
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    config.download_path = Some(path);
                }
            }
        });

        layout::space(ui, layout::SPACE_SM);
        ui.label(
            egui::RichText::new("⚠ Requires yt-dlp to be installed")
                .color(theme.alert())
                .size(11.0),
        );
        layout::hint_text(ui, "Install: pip install yt-dlp", theme);
    });
}
