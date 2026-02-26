// Audio settings section
use super::super::actions::SettingsAction;
use super::super::components::{button, card, input, layout};
use super::super::theme::Theme;
use crate::config::AudioConfig;
use eframe::egui;

/// Render audio settings section
pub fn render(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut AudioConfig,
    devices: &[crate::audio::devices::AudioDevice],
) -> Option<SettingsAction> {
    layout::space(ui, layout::SPACE_LG);
    render_output_card(ui, theme, config, devices);
    layout::space(ui, layout::SPACE_LG);
    render_microphone_card(ui, theme, config);
    layout::space(ui, layout::SPACE_LG);
    render_theme_card(ui, theme);
    super::action_buttons(ui, theme)
}

fn render_output_card(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut AudioConfig,
    devices: &[crate::audio::devices::AudioDevice],
) {
    card::settings_card(ui, theme, "AUDIO OUTPUT", |ui, theme| {
        layout::subsection_label(ui, "Device:", theme);
        render_device_selector(ui, config, devices);
        layout::space(ui, layout::SPACE_MD);
        layout::subsection_label(ui, "Latency:", theme);
        layout::info_text(ui, "~25ms (auto)", theme);
    });
}

fn render_device_selector(
    ui: &mut egui::Ui,
    config: &mut AudioConfig,
    devices: &[crate::audio::devices::AudioDevice],
) {
    let selected = config
        .output_device
        .clone()
        .unwrap_or_else(|| "Default Output".to_string());

    egui::ComboBox::from_id_salt("audio_device")
        .selected_text(&selected)
        .show_ui(ui, |ui| {
            for device in devices {
                let label = if device.is_default {
                    format!("{} (Default)", device.name)
                } else {
                    device.name.clone()
                };
                ui.selectable_value(&mut config.output_device, Some(device.name.clone()), label);
            }
        });
}

fn render_microphone_card(ui: &mut egui::Ui, theme: Theme, config: &mut AudioConfig) {
    card::settings_card(ui, theme, "MICROPHONE CALIBRATION", |ui, theme| {
        render_input_gain(ui, theme, config);
        layout::space(ui, layout::SPACE_MD);
        render_noise_gate(ui, theme, config);
    });
}

fn render_input_gain(ui: &mut egui::Ui, theme: Theme, config: &mut AudioConfig) {
    input::labeled_slider(
        ui,
        "Input Gain:",
        &mut config.input_gain,
        0.0..=1.0,
        theme,
        #[allow(clippy::cast_possible_truncation)]
        |val| format!("{}%", (val * 100.0) as i32),
    );
}

fn render_noise_gate(ui: &mut egui::Ui, theme: Theme, config: &mut AudioConfig) {
    layout::subsection_label(ui, "Noise Gate:", theme);
    button::toggle_button(ui, &mut config.noise_gate_enabled, theme);

    if config.noise_gate_enabled {
        layout::space(ui, layout::SPACE_SM);
        layout::subsection_label(ui, "Threshold:", theme);
        ui.add(
            egui::Slider::new(&mut config.noise_gate_threshold, 0.0..=0.1)
                .text("Level")
                .trailing_fill(true),
        );
    }
}

fn render_theme_card(ui: &mut egui::Ui, theme: Theme) {
    card::settings_card(ui, theme, "THEME OVERRIDE", |ui, theme| {
        layout::hint_text(ui, "Theme settings controlled in main view", theme);
    });
}
