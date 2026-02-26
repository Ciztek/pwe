use super::super::theme::Theme;
use eframe::egui;

/// Standard spacing values
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

/// Add vertical space
pub fn space(ui: &mut egui::Ui, amount: f32) {
    ui.add_space(amount);
}

/// Subsection label
pub fn subsection_label(ui: &mut egui::Ui, text: &str, theme: Theme) {
    ui.label(egui::RichText::new(text).color(theme.primary()).strong());
    space(ui, SPACE_XS);
}

/// Info text (muted, smaller)
pub fn info_text(ui: &mut egui::Ui, text: &str, theme: Theme) {
    ui.label(
        egui::RichText::new(text)
            .color(theme.text_muted())
            .size(11.0),
    );
}

/// Italic info text
pub fn hint_text(ui: &mut egui::Ui, text: &str, theme: Theme) {
    ui.label(
        egui::RichText::new(text)
            .color(theme.text_muted())
            .size(10.0)
            .italics(),
    );
}

/// Horizontal separator with space
pub fn separator(ui: &mut egui::Ui) {
    space(ui, SPACE_SM);
    ui.separator();
    space(ui, SPACE_SM);
}

/// Two-column layout helper
pub fn two_column<L, R>(ui: &mut egui::Ui, left_width: f32, left: L, right: R)
where
    L: FnOnce(&mut egui::Ui),
    R: FnOnce(&mut egui::Ui),
{
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(left_width);
            left(ui);
        });

        space(ui, SPACE_SM);
        ui.separator();
        space(ui, SPACE_SM);

        ui.vertical(|ui| {
            ui.set_min_width(ui.available_width());
            right(ui);
        });
    });
}

/// Action button row (typically at bottom of forms)
pub fn action_row(ui: &mut egui::Ui, buttons: impl FnOnce(&mut egui::Ui)) {
    space(ui, SPACE_XL);
    ui.horizontal(|ui| {
        buttons(ui);
    });
}
