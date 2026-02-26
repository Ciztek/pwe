use crate::ui::theme::Theme;
use eframe::egui;
use std::time::Duration;

use super::PlaybackAction;

const BAR_HEIGHT: f32 = 8.0;

pub fn render(
    ui: &mut egui::Ui,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
) -> Option<PlaybackAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.add_space(16.0);
        render_time_label(ui, current_position, theme);
        ui.add_space(8.0);

        if let Some(seek_action) = render_seekbar(ui, current_position, song_duration, theme) {
            action = Some(seek_action);
        }

        ui.add_space(8.0);
        render_duration_label(ui, song_duration, theme);
        ui.add_space(16.0);
    });

    action
}

fn render_time_label(ui: &mut egui::Ui, current_position: Duration, theme: Theme) {
    let current_str = format_duration(current_position);
    ui.label(
        egui::RichText::new(current_str)
            .color(theme.text_muted())
            .size(11.0)
            .monospace(),
    );
}

fn render_duration_label(ui: &mut egui::Ui, song_duration: Option<Duration>, theme: Theme) {
    if let Some(duration) = song_duration {
        let duration_str = format_duration(duration);
        ui.label(
            egui::RichText::new(duration_str)
                .color(theme.text_muted())
                .size(11.0)
                .monospace(),
        );
    } else {
        ui.label(
            egui::RichText::new("--:--")
                .color(theme.text_muted())
                .size(11.0)
                .monospace(),
        );
    }
}

fn render_seekbar(
    ui: &mut egui::Ui,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
) -> Option<PlaybackAction> {
    let progress = calculate_progress(current_position, song_duration);
    let bar_width = ui.available_width() - 80.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(bar_width, BAR_HEIGHT),
        egui::Sense::click_and_drag(),
    );

    let action = handle_seek_interaction(&response, rect, bar_width);
    draw_progress_bar(ui, rect, bar_width, progress, theme);

    action
}

fn calculate_progress(current_position: Duration, song_duration: Option<Duration>) -> f32 {
    if let Some(duration) = song_duration {
        if duration.as_secs() > 0 {
            let ratio = current_position.as_secs_f32() / duration.as_secs_f32();
            return ratio.clamp(0.0, 1.0);
        }
    }
    0.0
}

fn handle_seek_interaction(
    response: &egui::Response,
    rect: egui::Rect,
    bar_width: f32,
) -> Option<PlaybackAction> {
    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let ratio = ((pos.x - rect.min.x) / bar_width).clamp(0.0, 1.0);
            return Some(PlaybackAction::Seek(ratio));
        }
    }
    None
}

fn draw_progress_bar(ui: &egui::Ui, rect: egui::Rect, bar_width: f32, progress: f32, theme: Theme) {
    ui.painter().rect_filled(rect, 2.0, theme.secondary());

    if progress > 0.0 {
        let fill_width = (bar_width * progress).clamp(0.0, bar_width);
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, BAR_HEIGHT));
        ui.painter().rect_filled(fill_rect, 2.0, theme.accent());

        draw_progress_handle(ui, rect, fill_width, theme);
    }
}

fn draw_progress_handle(ui: &egui::Ui, rect: egui::Rect, fill_width: f32, theme: Theme) {
    let handle_x = rect.min.x + fill_width;
    let handle_rect = egui::Rect::from_min_size(
        egui::pos2(handle_x - 1.0, rect.min.y - 2.0),
        egui::vec2(2.0, BAR_HEIGHT + 4.0),
    );
    ui.painter().rect_filled(handle_rect, 0.0, theme.accent());
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{minutes:02}:{seconds:02}")
}
