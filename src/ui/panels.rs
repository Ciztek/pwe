// UI Panels - top, bottom, and central panel rendering
use super::theme::Theme;
use eframe::egui;
use std::time::Duration;

pub fn render_top_panel(ctx: &egui::Context, theme: Theme) -> bool {
    theme.apply(ctx);

    let mut theme_switched = false;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(10.0);

            // System indicator
            ui.colored_label(theme.primary(), "●");
            ui.add_space(5.0);

            // Main title with technical styling
            ui.heading(
                egui::RichText::new("PWE KARAOKE")
                    .color(theme.text_primary())
                    .strong(),
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Subtitle
            ui.label(
                egui::RichText::new("AUDIO SYSTEM v0.1.0")
                    .color(theme.text_muted())
                    .small(),
            );

            // Spacer to push theme button to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);

                // Theme switcher button
                let theme_btn = egui::Button::new(
                    egui::RichText::new(format!("◐ {}", theme.name()))
                        .color(theme.primary())
                        .small(),
                );
                if ui.add(theme_btn).clicked() {
                    theme_switched = true;
                }
            });
        });
    });

    theme_switched
}

pub fn render_bottom_panel(
    ctx: &egui::Context,
    is_playing: bool,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.set_style(ui.style().clone());

        // Status indicator
        ui.horizontal(|ui| {
            if is_playing {
                ui.colored_label(theme.active(), "▶ PLAYING");
            } else {
                ui.colored_label(theme.text_muted(), "⏸ PAUSED");
            }
        });

        ui.add_space(5.0);

        // Progress bar and time display
        ui.horizontal(|ui| {
            // Current time
            let current_str = format_duration(current_position);
            ui.colored_label(theme.text_primary(), current_str);

            ui.add_space(10.0);

            // Progress bar
            let progress = if let Some(duration) = song_duration {
                if duration.as_secs() > 0 {
                    current_position.as_secs_f32() / duration.as_secs_f32()
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let progress_bar = egui::ProgressBar::new(progress)
                .desired_width(ui.available_width() - 100.0)
                .fill(theme.active())
                .animate(is_playing);

            ui.add(progress_bar);

            ui.add_space(10.0);

            // Total duration
            if let Some(duration) = song_duration {
                let duration_str = format_duration(duration);
                ui.colored_label(theme.text_primary(), duration_str);
            } else {
                ui.colored_label(theme.text_muted(), "--:--");
            }
        });
    });
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
