use crate::library::Song;
use crate::ui::theme::Theme;
use eframe::egui;

const ALBUM_ART_SIZE: f32 = 40.0;

pub fn render(ui: &mut egui::Ui, song: Option<&Song>, theme: Theme) {
    if let Some(song) = song {
        if let Some(texture_id) = try_load_album_art(ui, song) {
            render_texture(ui, texture_id);
            return;
        }
    }
    render_placeholder(ui, theme);
}

fn try_load_album_art(ui: &egui::Ui, song: &Song) -> Option<egui::TextureId> {
    let metadata = song.metadata.as_ref()?;
    let cover_data = metadata.cover_art.as_ref()?;
    let dynamic_image = image::load_from_memory(cover_data).ok()?;

    let rgba_image = dynamic_image.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [rgba_image.width() as _, rgba_image.height() as _],
        rgba_image.as_raw(),
    );

    let texture = ui.ctx().load_texture(
        format!("playback_art_{}", song.path.display()),
        color_image,
        egui::TextureOptions::LINEAR,
    );

    Some(texture.id())
}

fn render_texture(ui: &mut egui::Ui, texture_id: egui::TextureId) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ALBUM_ART_SIZE, ALBUM_ART_SIZE),
        egui::Sense::hover(),
    );
    ui.painter().image(
        texture_id,
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

fn render_placeholder(ui: &mut egui::Ui, theme: Theme) {
    ui.painter().rect_filled(
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ALBUM_ART_SIZE, ALBUM_ART_SIZE)),
        2.0,
        theme.secondary(),
    );
    ui.allocate_space(egui::vec2(ALBUM_ART_SIZE, ALBUM_ART_SIZE));
}
