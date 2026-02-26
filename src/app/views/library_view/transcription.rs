// Transcription status UI

use eframe::egui;
use tracing::info;

use crate::app::KaraokeApp;
use crate::ui::widgets;

pub fn render_transcription_status(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    if !app.transcription_state.is_transcribing {
        return;
    }

    ui.add_space(8.0);

    let should_cancel = widgets::render_armor_card(ui, app.ui.theme, |ui| {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                egui::RichText::new("Transcribing lyrics...")
                    .color(app.ui.theme.accent())
                    .strong()
                    .size(13.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.button(
                    egui::RichText::new("[×]")
                        .size(14.0)
                        .color(app.ui.theme.error()),
                )
                .on_hover_text("Cancel transcription")
                .clicked()
            })
            .inner
        })
        .inner
    });

    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("🎵 {}", app.transcription_state.song_name))
                .color(app.ui.theme.text_primary())
                .size(12.0),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("This may take a few minutes depending on song length...")
                .color(app.ui.theme.text_muted())
                .size(10.0),
        );
    });

    ui.add_space(8.0);

    if should_cancel {
        info!("Transcription cancelled by user");
        app.transcription_state.is_transcribing = false;
        app.transcription_state.song_path = None;
    }

    check_transcription_completion(app);
}

fn check_transcription_completion(app: &mut KaraokeApp) {
    if let Some(ref path) = app.transcription_state.song_path {
        let lrc_path = path.with_extension("lrc");
        if lrc_path.exists() {
            info!("Transcription completed, refreshing library");
            app.transcription_state.is_transcribing = false;
            app.transcription_state.song_path = None;
            app.library.refresh_library();
        }
    }
}
