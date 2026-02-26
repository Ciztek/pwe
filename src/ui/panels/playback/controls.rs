use crate::ui::theme::Theme;
use eframe::egui;

use super::PlaybackAction;

pub fn render(ui: &mut egui::Ui, is_playing: bool, theme: Theme) -> Option<PlaybackAction> {
    let mut action = if ui
        .button(egui::RichText::new("<<").color(theme.text_muted()))
        .clicked()
    {
        Some(PlaybackAction::SkipBackward)
    } else {
        None
    };

    ui.add_space(4.0);

    if render_play_pause(ui, is_playing, theme) {
        action = Some(PlaybackAction::PlayPause);
    }

    ui.add_space(4.0);

    if ui
        .button(egui::RichText::new(">>").color(theme.text_muted()))
        .clicked()
    {
        action = Some(PlaybackAction::SkipForward);
    }

    action
}

fn render_play_pause(ui: &mut egui::Ui, is_playing: bool, theme: Theme) -> bool {
    let play_text = if is_playing { "⏸" } else { "▶" };
    ui.button(
        egui::RichText::new(play_text)
            .color(theme.primary())
            .size(16.0),
    )
    .clicked()
}
