use super::super::theme::Theme;
use eframe::egui;

/// Styled card with border accent (the "armor card")
pub fn armor_card<R>(
    ui: &mut egui::Ui,
    theme: Theme,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let frame = egui::Frame::none()
        .fill(theme.card_surface())
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, theme.secondary()));

    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left accent bar
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(3.0, ui.available_height()),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.primary());

                ui.add_space(8.0);
                ui.vertical(|ui| content(ui)).inner
            })
            .inner
        })
        .inner
}

/// Settings-style card with title
pub fn settings_card<R>(
    ui: &mut egui::Ui,
    theme: Theme,
    title: &str,
    content: impl FnOnce(&mut egui::Ui, Theme) -> R,
) -> R {
    ui.group(|ui| {
        ui.set_min_width(400.0);

        ui.label(
            egui::RichText::new(title)
                .color(theme.primary())
                .size(13.0)
                .strong(),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        let result = content(ui, theme);

        ui.add_space(8.0);
        result
    })
    .inner
}
