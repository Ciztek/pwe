use super::super::theme::Theme;
use eframe::egui;

/// Button style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Accent,
    Alert,
}

/// Styled button builder
pub struct StyledButton {
    text: String,
    style: ButtonStyle,
    enabled: bool,
    tooltip: Option<String>,
}

impl StyledButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: ButtonStyle::Primary,
            enabled: true,
            tooltip: None,
        }
    }

    pub const fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn show(self, ui: &mut egui::Ui, theme: Theme) -> egui::Response {
        let color = match self.style {
            ButtonStyle::Primary => theme.primary(),
            ButtonStyle::Accent => theme.accent(),
            ButtonStyle::Alert => theme.alert(),
        };

        let text = egui::RichText::new(&self.text).color(color);
        let response = ui.add_enabled(self.enabled, egui::Button::new(text));

        if let Some(tooltip) = self.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response
        }
    }
}

/// Toggle button (ON/OFF style)
pub fn toggle_button(ui: &mut egui::Ui, enabled: &mut bool, theme: Theme) -> egui::Response {
    let text = if *enabled { "[ ON ]" } else { "[ OFF ]" };
    let color = if *enabled {
        theme.accent()
    } else {
        theme.text_muted()
    };

    let response = ui.button(egui::RichText::new(text).color(color));
    if response.clicked() {
        *enabled = !*enabled;
    }
    response
}

/// Sidebar selection button
pub fn sidebar_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    theme: Theme,
) -> egui::Response {
    let prefix = if is_active { "[ > ] " } else { "[   ] " };
    let color = if is_active {
        theme.primary()
    } else {
        theme.text_muted()
    };

    ui.button(egui::RichText::new(format!("{prefix}{label}")).color(color))
}
