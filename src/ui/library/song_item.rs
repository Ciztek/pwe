// Individual song item renderer
use super::super::actions::LibraryAction;
use super::super::components::layout;
use super::super::theme::Theme;
use crate::app::AppState;
use crate::library::Song;
use eframe::egui;

pub fn render(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    app_state: &mut AppState,
    playlists: &crate::playlist::PlaylistCollection,
) -> Option<LibraryAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        render_album_art(ui, song, app_state, theme);
        render_song_info(ui, song, theme, &mut action);
        render_actions(ui, song, theme, playlists, &mut action);
    });

    action
}

fn render_album_art(ui: &mut egui::Ui, song: &Song, app_state: &mut AppState, theme: Theme) {
    let size = 48.0;

    if let Some(texture) = load_album_art_texture(ui, song, app_state) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        render_placeholder_art(ui, size, theme);
    }

    layout::space(ui, layout::SPACE_SM);
}

fn load_album_art_texture(
    ui: &egui::Ui,
    song: &Song,
    app_state: &mut AppState,
) -> Option<egui::TextureHandle> {
    if let Some(cached) = app_state.thumbnail_texture_cache.get(&song.path) {
        return Some(cached.clone());
    }

    if let Some(metadata) = &song.metadata {
        if let Some(cover_data) = &metadata.cover_art {
            if let Ok(dynamic_image) = image::load_from_memory(cover_data) {
                let rgba = dynamic_image.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as _, rgba.height() as _],
                    rgba.as_raw(),
                );
                let texture = ui.ctx().load_texture(
                    format!("album_art_{}", song.path.display()),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                app_state
                    .thumbnail_texture_cache
                    .insert(song.path.clone(), texture.clone());
                return Some(texture);
            }
        }
    }

    None
}

fn render_placeholder_art(ui: &mut egui::Ui, size: f32, theme: Theme) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 2.0, theme.card_surface().gamma_multiply(0.6));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "♫",
        egui::FontId::proportional(20.0),
        theme.text_muted(),
    );
}

fn render_song_info(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    action: &mut Option<LibraryAction>,
) {
    ui.vertical(|ui| {
        let label = ui.add(
            egui::Label::new(
                egui::RichText::new(song.display_title())
                    .color(theme.text_primary())
                    .size(13.0),
            )
            .sense(egui::Sense::click()),
        );

        if label.clicked() {
            *action = Some(LibraryAction::PlaySong(song.path.clone()));
        }

        if label.hovered() {
            ui.painter().rect_stroke(
                label.rect.expand(2.0),
                2.0,
                egui::Stroke::new(1.0, theme.accent()),
            );
        }

        if let Some(artist) = song.artist() {
            ui.label(
                egui::RichText::new(artist)
                    .color(theme.text_muted())
                    .size(10.0),
            );
        }
    });
}

fn render_actions(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    playlists: &crate::playlist::PlaylistCollection,
    action: &mut Option<LibraryAction>,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        render_remove_button(ui, song, theme, action);
        layout::space(ui, layout::SPACE_SM);
        render_favorite_button(ui, song, theme, action);
        layout::space(ui, layout::SPACE_XS);
        render_playlist_menu(ui, song, theme, playlists, action);
        layout::space(ui, layout::SPACE_SM);
        render_lyrics_indicator(ui, song, theme, action);
        layout::space(ui, layout::SPACE_SM);
        render_duration(ui, song, theme);
    });
}

fn render_remove_button(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    action: &mut Option<LibraryAction>,
) {
    if ui
        .button(egui::RichText::new("[×]").size(14.0).color(theme.error()))
        .clicked()
    {
        *action = Some(LibraryAction::RemoveSong(song.path.clone()));
    }
}

fn render_favorite_button(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    action: &mut Option<LibraryAction>,
) {
    let icon = if song.is_favorite { "★" } else { "☆" };
    let color = if song.is_favorite {
        theme.accent()
    } else {
        theme.text_muted()
    };

    if ui
        .button(egui::RichText::new(icon).size(16.0).color(color))
        .clicked()
    {
        *action = Some(LibraryAction::ToggleFavorite(song.path.clone()));
    }
}

fn render_playlist_menu(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    playlists: &crate::playlist::PlaylistCollection,
    action: &mut Option<LibraryAction>,
) {
    if playlists.playlists.is_empty() {
        return;
    }

    ui.menu_button(
        egui::RichText::new("▼")
            .size(12.0)
            .color(theme.text_muted()),
        |ui| {
            ui.label(
                egui::RichText::new("Add to Playlist")
                    .color(theme.primary())
                    .size(11.0),
            );
            ui.separator();

            for playlist in &playlists.playlists {
                let is_in = playlist.contains(&song.path);
                let label = if is_in {
                    format!("✓ {}", playlist.name)
                } else {
                    playlist.name.clone()
                };

                if ui.button(label).clicked() {
                    *action = if is_in {
                        Some(LibraryAction::RemoveFromPlaylist(
                            playlist.name.clone(),
                            song.path.clone(),
                        ))
                    } else {
                        Some(LibraryAction::AddToPlaylist(
                            playlist.name.clone(),
                            song.path.clone(),
                        ))
                    };
                    ui.close_menu();
                }
            }
        },
    );
}

fn render_lyrics_indicator(
    ui: &mut egui::Ui,
    song: &Song,
    theme: Theme,
    action: &mut Option<LibraryAction>,
) {
    if song.has_lyrics {
        ui.label(egui::RichText::new("🎤").size(13.0).color(theme.accent()))
            .on_hover_text("Has lyrics file");
    } else if ui
        .button(
            egui::RichText::new("[ 🎙 Transcribe ]")
                .size(10.0)
                .color(theme.primary()),
        )
        .on_hover_text("Generate lyrics using Whisper AI")
        .clicked()
    {
        *action = Some(LibraryAction::TranscribeLyrics(song.path.clone()));
    }
}

fn render_duration(ui: &mut egui::Ui, song: &Song, theme: Theme) {
    if let Some(duration) = song.duration() {
        let minutes = duration / 60;
        let seconds = duration % 60;
        ui.label(
            egui::RichText::new(format!("{minutes}:{seconds:02}"))
                .size(11.0)
                .color(theme.text_muted())
                .monospace(),
        );
    }
}
