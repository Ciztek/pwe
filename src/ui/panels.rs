use super::theme::Theme;
use crate::app::AppView;
use eframe::egui;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PlaybackAction {
    None,
    PlayPause,
    Stop,
    SkipForward,
    SkipBackward,
}

pub fn render_top_panel(
    ctx: &egui::Context,
    theme: Theme,
    current_view: AppView,
) -> (bool, Option<AppView>) {
    theme.apply(ctx);

    let mut theme_switched = false;
    let mut view_change = None;

    egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame::none().fill(theme.card_surface()))
        .min_height(50.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(32.0);

                ui.label(
                    egui::RichText::new("IRON-VOX")
                        .size(20.0)
                        .color(theme.primary())
                        .strong(),
                );

                ui.add_space(32.0);

                let library_color = if current_view == AppView::Library {
                    theme.primary()
                } else {
                    theme.text_muted()
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("[ Library ]").color(library_color),
                    ))
                    .clicked()
                {
                    view_change = Some(AppView::Library);
                }

                ui.add_space(8.0);

                let karaoke_color = if current_view == AppView::Karaoke {
                    theme.primary()
                } else {
                    theme.text_muted()
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("[ Karaoke ]").color(karaoke_color),
                    ))
                    .clicked()
                {
                    view_change = Some(AppView::Karaoke);
                }

                ui.add_space(8.0);

                let settings_color = if current_view == AppView::Settings {
                    theme.primary()
                } else {
                    theme.text_muted()
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("[ Settings ]").color(settings_color),
                    ))
                    .clicked()
                {
                    view_change = Some(AppView::Settings);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);

                    let theme_text = format!("[ {} ]", theme.name());
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(theme_text)
                                .color(theme.accent())
                                .small(),
                        ))
                        .clicked()
                    {
                        theme_switched = true;
                    }
                });
            });
            ui.add_space(8.0);
        });

    (theme_switched, view_change)
}

pub fn render_bottom_panel(
    ctx: &egui::Context,
    is_playing: bool,
    current_position: Duration,
    song_duration: Option<Duration>,
    theme: Theme,
    current_song_name: Option<&str>,
) -> PlaybackAction {
    let mut action = PlaybackAction::None;

    egui::TopBottomPanel::bottom("bottom_panel")
        .frame(egui::Frame::none().fill(theme.card_surface()))
        .min_height(100.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);

                ui.painter().rect_filled(
                    egui::Rect::from_min_size(ui.cursor().min, egui::vec2(40.0, 40.0)),
                    2.0,
                    theme.secondary(),
                );
                ui.allocate_space(egui::vec2(40.0, 40.0));

                ui.add_space(12.0);

                if let Some(name) = current_song_name {
                    ui.label(
                        egui::RichText::new(name)
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);

                    ui.label(
                        egui::RichText::new("Vol:")
                            .color(theme.text_muted())
                            .size(12.0),
                    );

                    ui.add_space(8.0);

                    if ui
                        .button(egui::RichText::new(">>").color(theme.text_muted()))
                        .clicked()
                    {
                        tracing::info!("Skip forward - to be implemented");
                    }

                    ui.add_space(4.0);

                    let play_text = if is_playing { "⏸" } else { "▶" };
                    if ui
                        .button(
                            egui::RichText::new(play_text)
                                .color(theme.primary())
                                .size(16.0),
                        )
                        .clicked()
                    {
                        action = PlaybackAction::PlayPause;
                    }

                    ui.add_space(4.0);

                    if ui
                        .button(egui::RichText::new("<<").color(theme.text_muted()))
                        .clicked()
                    {
                        tracing::info!("Skip backward - to be implemented");
                    }
                });
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);

                let current_str = format_duration(current_position);
                ui.label(
                    egui::RichText::new(current_str)
                        .color(theme.text_muted())
                        .size(11.0)
                        .monospace(),
                );

                ui.add_space(8.0);

                let progress = if let Some(duration) = song_duration {
                    if duration.as_secs() > 0 {
                        let ratio = current_position.as_secs_f32() / duration.as_secs_f32();
                        ratio.clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let bar_width = ui.available_width() - 80.0;
                let bar_height = 8.0;
                let (rect, _response) =
                    ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());

                ui.painter().rect_filled(rect, 2.0, theme.secondary());

                if progress > 0.0 {
                    let fill_width = (bar_width * progress).clamp(0.0, bar_width);
                    let fill_rect =
                        egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, bar_height));
                    ui.painter().rect_filled(fill_rect, 2.0, theme.accent());

                    let handle_x = rect.min.x + fill_width;
                    let handle_rect = egui::Rect::from_min_size(
                        egui::pos2(handle_x - 1.0, rect.min.y - 2.0),
                        egui::vec2(2.0, bar_height + 4.0),
                    );
                    ui.painter().rect_filled(handle_rect, 0.0, theme.accent());
                }

                ui.add_space(8.0);

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

                ui.add_space(16.0);
            });

            ui.add_space(8.0);
        });

    action
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
