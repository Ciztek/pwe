// Frame update loop handling

use eframe::egui;
use enum_cycling::EnumCycle;
use std::time::Duration;
use tracing::{error, info};

use crate::app::{playback, AppView, DownloadMessage, KaraokeApp};
use crate::ui::panels;

pub fn update_frame(app: &mut KaraokeApp, ctx: &egui::Context) {
    update_per_frame_state(app);
    poll_download_progress(app, ctx);
    update_audio_state(app, ctx);
    update_karaoke_sync(app);

    handle_ui_panels(app, ctx);
    render_main_view(app, ctx);
}

fn update_per_frame_state(app: &mut KaraokeApp) {
    app.app_state.textures_loaded_this_frame = 0;

    let now = std::time::Instant::now();
    if let Some(last_time) = app.app_state.last_frame_time {
        let frame_time = now.duration_since(last_time).as_secs_f32();
        if frame_time > 0.0 {
            let fps = 1.0 / frame_time;
            app.app_state.fps_smooth = if app.app_state.fps_smooth > 0.0 {
                app.app_state.fps_smooth.mul_add(0.9, fps * 0.1)
            } else {
                fps
            };
        }
    }
    app.app_state.last_frame_time = Some(now);
}

fn poll_download_progress(app: &mut KaraokeApp, ctx: &egui::Context) {
    if let Some(rx) = &app.network_state.download_rx {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                DownloadMessage::Started { total } => {
                    app.download_state.total_count = total;
                    app.download_state.is_downloading = true;
                    info!("📊 Download started: {} videos", total);
                },
                DownloadMessage::Progress {
                    index,
                    song,
                    status,
                } => {
                    app.download_state.current_index = index;
                    app.download_state.current_song.clone_from(&song);
                    app.download_state.status_message = status;
                    info!(
                        "📥 Downloading {}/{}: {}",
                        index, app.download_state.total_count, song
                    );
                },
                DownloadMessage::Completed => {
                    app.download_state.is_downloading = false;
                    app.download_state.status_message = "All downloads completed!".to_string();
                    info!("✅ All downloads completed");
                },
                DownloadMessage::Error(e) => {
                    app.download_state.is_downloading = false;
                    app.download_state.status_message = format!("Error: {e}");
                    error!("❌ Download error: {}", e);
                },
            }
            ctx.request_repaint();
        }
    }
}

fn update_audio_state(app: &mut KaraokeApp, ctx: &egui::Context) {
    if app.audio.is_playing {
        if app.audio.audio_player.is_empty() {
            app.audio.is_playing = false;
        } else {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn update_karaoke_sync(app: &mut KaraokeApp) {
    // Static for logging throttle
    static mut LAST_LOG_TIME: u64 = 0;

    let current_position = app.audio.audio_player.get_position();
    #[allow(clippy::cast_possible_truncation)]
    let current_position_ms = current_position.as_millis() as u64;

    // Log position every 2 seconds for debugging
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

    app.karaoke.update(current_position_ms);
}

fn handle_ui_panels(app: &mut KaraokeApp, ctx: &egui::Context) {
    let (theme_switched, view_change) = panels::render_top_panel(
        ctx,
        app.ui.theme,
        app.ui.current_view,
        app.app_state.fps_smooth,
    );

    if theme_switched {
        app.ui.theme = app.ui.theme.up();
        info!("Theme switched to {:?}", app.ui.theme);
    }

    if let Some(new_view) = view_change {
        app.ui.current_view = new_view;
        info!("View changed to {:?}", new_view);
    }

    let current_position = app.audio.audio_player.get_position();
    let current_song = app
        .audio
        .current_file
        .as_ref()
        .and_then(|path| app.library.library.iter().find(|s| &s.path == path));

    let playback_action = panels::render_bottom_panel(
        ctx,
        app.audio.is_playing,
        current_position,
        app.audio.song_duration,
        app.ui.theme,
        current_song,
    );

    playback::handle_playback_action(app, playback_action);
}

fn render_main_view(app: &mut KaraokeApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(app.ui.theme.background())
                .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
        )
        .show(ctx, |ui| match app.ui.current_view {
            AppView::Library => crate::app::views::render_library_view(app, ui),
            AppView::Karaoke => crate::app::views::render_karaoke_view(app, ui),
            AppView::Settings => crate::app::views::render_settings_view(app, ui),
        });
}
