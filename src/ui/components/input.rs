use super::super::theme::Theme;
use eframe::egui;

/// Labeled slider with value display
pub fn labeled_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    theme: Theme,
    format: impl Fn(f32) -> String,
) {
    ui.label(
        egui::RichText::new(label)
            .color(theme.text_muted())
            .size(12.0),
    );
    ui.add_space(4.0);

    ui.add(
        egui::Slider::new(value, range)
            .show_value(false)
            .trailing_fill(true),
    );

    ui.label(
        egui::RichText::new(format(*value))
            .color(theme.accent())
            .monospace(),
    );
}

/// Checkbox with styled label
pub fn checkbox(
    ui: &mut egui::Ui,
    label: &str,
    checked: &mut bool,
    theme: Theme,
) -> egui::Response {
    ui.checkbox(
        checked,
        egui::RichText::new(label).color(theme.text_primary()),
    )
}
