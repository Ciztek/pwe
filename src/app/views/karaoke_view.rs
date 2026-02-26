// Karaoke view rendering

use eframe::egui;

use crate::app::KaraokeApp;

pub fn render_karaoke_view(app: &KaraokeApp, ui: &mut egui::Ui) {
    // Show current song info at the top
    let header_response = app
        .audio
        .current_file
        .as_ref()
        .and_then(|path| path.file_stem().and_then(|s| s.to_str()))
        .map(|song_name| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Now Playing:")
                        .color(app.ui.theme.text_muted())
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(song_name)
                        .color(app.ui.theme.text_primary())
                        .size(14.0)
                        .strong(),
                );
            })
        });

    if header_response.is_some() {
        ui.add_space(8.0);
    }

    // Calculate remaining height for content
    let available_height = ui.available_height();

    if app.karaoke.lyrics.is_empty() {
        // No lyrics loaded - show placeholder centered in remaining space
        egui::Frame::none().show(ui, |ui| {
            ui.set_min_height(available_height);
            ui.vertical_centered(|ui| {
                ui.add_space(available_height * 0.3);

                if let Some(error) = &app.karaoke.lrc_error {
                    ui.label(
                        egui::RichText::new("⚠️")
                            .size(48.0)
                            .color(app.ui.theme.text_muted()),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(error)
                            .size(16.0)
                            .color(app.ui.theme.text_muted()),
                    );
                } else if app.audio.current_file.is_none() {
                    ui.label(
                        egui::RichText::new("🎤")
                            .size(64.0)
                            .color(app.ui.theme.text_muted()),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("No song playing")
                            .size(24.0)
                            .color(app.ui.theme.text_muted()),
                    );

                    ui.add_space(16.0);

                    ui.label(
                        egui::RichText::new("Go to Library and play a song with a .lrc file")
                            .size(14.0)
                            .color(app.ui.theme.text_muted())
                            .italics(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No lyrics available")
                            .size(24.0)
                            .color(app.ui.theme.text_muted()),
                    );

                    ui.add_space(16.0);

                    if let Some(audio_path) = &app.audio.current_file {
                        let lrc_path = audio_path.with_extension("lrc");
                        ui.label(
                            egui::RichText::new(format!(
                                "Create a .lrc file at:\n{}",
                                lrc_path.display()
                            ))
                            .size(12.0)
                            .color(app.ui.theme.text_muted())
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
        .min_scrolled_height(available_height)
        .max_height(available_height)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden);

    // Auto-scroll to current line
    if let Some(current_idx) = app.karaoke.current_line_index {
        let line_height = 60.0;
        #[allow(clippy::cast_precision_loss)]
        let target_offset = (current_idx as f32 * line_height).max(0.0);
        scroll_area = scroll_area.vertical_scroll_offset(target_offset);
    }

    scroll_area.show(ui, |ui| {
        ui.add_space(100.0);

        let current_index = app.karaoke.current_line_index;

        for (i, line) in app.karaoke.lyrics.iter().enumerate() {
            let is_current = current_index == Some(i);
            let is_upcoming = current_index.is_some_and(|idx| i == idx + 1);
            let is_past = current_index.is_some_and(|idx| i < idx);

            let (color, size, strong) = if is_current {
                (app.ui.theme.accent(), 28.0, true)
            } else if is_upcoming {
                (app.ui.theme.primary().gamma_multiply(0.7), 22.0, false)
            } else if is_past {
                (app.ui.theme.text_muted().gamma_multiply(0.4), 18.0, false)
            } else {
                (app.ui.theme.text_muted().gamma_multiply(0.25), 16.0, false)
            };

            let mut text = egui::RichText::new(&line.text).color(color).size(size);

            if strong {
                text = text.strong();
            }

            ui.centered_and_justified(|ui| {
                ui.label(text);
            });

            ui.add_space(16.0);
        }

        ui.add_space(200.0);
    });
}
