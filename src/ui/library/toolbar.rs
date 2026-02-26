// Library toolbar - action buttons for library management
use super::super::actions::LibraryAction;
use super::super::components::{button, layout};
use super::super::theme::Theme;
use eframe::egui;

pub fn render(
    ui: &mut egui::Ui,
    _add_song_path: &mut String,
    theme: Theme,
) -> Option<LibraryAction> {
    let mut action = None;

    layout::space(ui, layout::SPACE_SM);

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("PERSISTENT LIBRARY")
                .color(theme.primary())
                .size(12.0)
                .strong(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);

            if button::StyledButton::new("[ + Add Song ]")
                .style(button::ButtonStyle::Accent)
                .show(ui, theme)
                .clicked()
            {
                action = Some(LibraryAction::AddSong);
            }

            layout::space(ui, layout::SPACE_SM);

            if button::StyledButton::new("[ ↻ Refresh ]")
                .style(button::ButtonStyle::Primary)
                .show(ui, theme)
                .clicked()
            {
                action = Some(LibraryAction::RefreshLibrary);
            }
        });
    });

    action
}
