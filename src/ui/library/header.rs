// Library header - shows featured song with album art and track count
use super::super::theme::Theme;
use crate::library::Song;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, library: &[&Song], theme: Theme) {
    ui.horizontal(|ui| {
        render_placeholder_art(ui, theme);
        ui.add_space(16.0);
        render_info(ui, library, theme);
    });
}

fn render_placeholder_art(ui: &mut egui::Ui, theme: Theme) {
    let size = egui::vec2(80.0, 80.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 2.0, theme.card_surface().gamma_multiply(0.7));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "[ Art ]",
        egui::FontId::proportional(10.0),
        theme.text_muted(),
    );
}

fn render_info(ui: &mut egui::Ui, library: &[&Song], theme: Theme) {
    ui.vertical(|ui| {
        if library.is_empty() {
            render_empty_state(ui, theme);
        } else {
            render_first_song(ui, library[0], library.len(), theme);
        }
    });
}

fn render_empty_state(ui: &mut egui::Ui, theme: Theme) {
    ui.label(
        egui::RichText::new("No Library Loaded")
            .size(18.0)
            .color(theme.text_muted())
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Scan a directory to get started")
            .size(12.0)
            .color(theme.text_muted())
            .italics(),
    );
}

fn render_first_song(ui: &mut egui::Ui, song: &Song, total: usize, theme: Theme) {
    ui.horizontal(|ui| {
        render_album_art_if_available(ui, song);
        render_song_metadata(ui, song, total, theme);
    });
}

fn render_album_art_if_available(ui: &mut egui::Ui, song: &Song) {
    if let Some(metadata) = &song.metadata {
        if let Some(cover_data) = &metadata.cover_art {
            if let Ok(dynamic_image) = image::load_from_memory(cover_data) {
                let rgba = dynamic_image.to_rgba8();
                let size = 48.0;
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as _, rgba.height() as _],
                    rgba.as_raw(),
                );
                let texture = ui.ctx().load_texture(
                    format!("library_header_art_{}", song.path.display()),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                ui.add_space(8.0);
            }
        }
    }
}

fn render_song_metadata(ui: &mut egui::Ui, song: &Song, total: usize, theme: Theme) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(song.display_title())
                .size(18.0)
                .color(theme.text_primary())
                .strong(),
        );
        ui.add_space(4.0);

        if let Some(artist) = song.artist() {
            ui.label(
                egui::RichText::new(artist)
                    .size(13.0)
                    .color(theme.text_muted()),
            );
        }

        if let Some(album) = song.album() {
            ui.label(
                egui::RichText::new(format!("♫ {album}"))
                    .size(11.0)
                    .color(theme.text_muted()),
            );
        }

        ui.label(
            egui::RichText::new(format!("{total} tracks loaded"))
                .size(12.0)
                .color(theme.text_muted()),
        );
    });
}
