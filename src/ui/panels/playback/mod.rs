mod album_art;
mod controls;
mod progress;

use crate::library::Song;
use crate::ui::theme::Theme;
use eframe::egui;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PlaybackAction {
    None,
    PlayPause,
    Stop,
    SkipForward,
    SkipBackward,
    Seek(f32),
}

pub fn render_bottom_panel(
    ctx: &egui::Context,
    is_playing: bool,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
    current_song: Option<&Song>,
) -> PlaybackAction {
    let mut action = PlaybackAction::None;

    egui::TopBottomPanel::bottom("bottom_panel")
        .frame(egui::Frame::none().fill(theme.card_surface()))
        .min_height(100.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            render_top_row(ui, is_playing, current_song, theme, &mut action);
            ui.add_space(8.0);
            render_progress_row(ui, current_position, song_duration, theme, &mut action);
            ui.add_space(8.0);
        });

    action
}

fn render_top_row(
    ui: &mut egui::Ui,
    is_playing: bool,
    current_song: Option<&Song>,
    theme: Theme,
    action: &mut PlaybackAction,
) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        album_art::render(ui, current_song, theme);
        ui.add_space(12.0);
        render_song_title(ui, current_song, theme);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            render_volume_indicator(ui, theme);
            ui.add_space(8.0);

            if let Some(control_action) = controls::render(ui, is_playing, theme) {
                *action = control_action;
            }
        });
    });
}

fn render_song_title(ui: &mut egui::Ui, current_song: Option<&Song>, theme: Theme) {
    if let Some(song) = current_song {
        ui.label(
            egui::RichText::new(song.display_title())
                .color(theme.text_primary())
                .size(14.0),
        );
    } else {
        ui.label(
            egui::RichText::new("No track loaded")
                .color(theme.text_muted())
                .size(14.0),
        );
    }
}

fn render_volume_indicator(ui: &mut egui::Ui, theme: Theme) {
    ui.label(
        egui::RichText::new("Vol:")
            .color(theme.text_muted())
            .size(12.0),
    );
}

fn render_progress_row(
    ui: &mut egui::Ui,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
    action: &mut PlaybackAction,
) {
    if let Some(seek_action) = progress::render(ui, current_position, song_duration, theme) {
        *action = seek_action;
    }
}
