// Display settings section
use super::super::actions::SettingsAction;
use super::super::components::{card, input, layout};
use super::super::theme::Theme;
use crate::config::DisplayConfig;
use eframe::egui;

/// Render display settings section
pub fn render(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut DisplayConfig,
) -> Option<SettingsAction> {
    layout::space(ui, layout::SPACE_LG);
    render_hud_options(ui, theme, config);
    layout::space(ui, layout::SPACE_LG);
    render_lyrics_info(ui, theme);
    super::action_buttons(ui, theme)
}

fn render_hud_options(ui: &mut egui::Ui, theme: Theme, config: &mut DisplayConfig) {
    card::settings_card(ui, theme, "HUD OPTIONS", |ui, theme| {
        render_font_size(ui, theme, config);
        layout::space(ui, layout::SPACE_MD);
        render_display_toggles(ui, theme, config);
    });
}

fn render_font_size(ui: &mut egui::Ui, theme: Theme, config: &mut DisplayConfig) {
    layout::subsection_label(ui, "Font Size:", theme);
    ui.horizontal(|ui| {
        ui.add(
            egui::Slider::new(&mut config.font_size, 10.0..=32.0)
                .text("pt")
                .trailing_fill(true),
        );
        if ui.button("[ Reset ]").clicked() {
            config.font_size = 16.0;
        }
    });
}

fn render_display_toggles(ui: &mut egui::Ui, theme: Theme, config: &mut DisplayConfig) {
    input::checkbox(ui, "Show Audio Waveform", &mut config.show_waveform, theme);
    layout::space(ui, layout::SPACE_SM);
    input::checkbox(ui, "Show Pitch Guide", &mut config.show_pitch_guide, theme);
    layout::space(ui, layout::SPACE_SM);
    input::checkbox(ui, "Start in Fullscreen", &mut config.fullscreen, theme);
}

fn render_lyrics_info(ui: &mut egui::Ui, theme: Theme) {
    card::settings_card(ui, theme, "LYRICS DISPLAY", |ui, theme| {
        for text in [
            "• Lyrics automatically loaded from .lrc files",
            "• Current/upcoming lines highlighted",
            "• Color customization coming soon",
        ] {
            layout::info_text(ui, text, theme);
        }
    });
}
