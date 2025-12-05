use super::theme::Theme;
use eframe::egui;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Audio,
    Display,
    Library,
    Network,
}

pub struct SettingsState {
    current_section: SettingsSection,
    noise_gate_enabled: bool,
    input_gain: f32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            current_section: SettingsSection::Audio,
            noise_gate_enabled: true,
            input_gain: 0.75,
        }
    }
}

pub fn render_settings_panel(ui: &mut egui::Ui, theme: Theme, state: &mut SettingsState) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(200.0);

            ui.add_space(8.0);

            render_category_button(
                ui,
                theme,
                "AUDIO SYSTEM",
                SettingsSection::Audio,
                state.current_section,
                &mut state.current_section,
            );

            render_category_button(
                ui,
                theme,
                "DISPLAY / HUD",
                SettingsSection::Display,
                state.current_section,
                &mut state.current_section,
            );

            render_category_button(
                ui,
                theme,
                "LIBRARY PATHS",
                SettingsSection::Library,
                state.current_section,
                &mut state.current_section,
            );

            render_category_button(
                ui,
                theme,
                "ALAYA-LINK (Net)",
                SettingsSection::Network,
                state.current_section,
                &mut state.current_section,
            );
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_min_width(ui.available_width());

            match state.current_section {
                SettingsSection::Audio => render_audio_settings(ui, theme, state),
                SettingsSection::Display => render_display_settings(ui, theme),
                SettingsSection::Library => render_library_settings(ui, theme),
                SettingsSection::Network => render_network_settings(ui, theme),
            }
        });
    });
}

fn render_category_button(
    ui: &mut egui::Ui,
    theme: Theme,
    label: &str,
    section: SettingsSection,
    current: SettingsSection,
    current_mut: &mut SettingsSection,
) {
    let is_active = section == current;
    let text_color = if is_active {
        theme.primary()
    } else {
        theme.text_muted()
    };

    let prefix = if is_active { "[ > ] " } else { "[   ] " };

    if ui
        .button(egui::RichText::new(format!("{}{}", prefix, label)).color(text_color))
        .clicked()
    {
        *current_mut = section;
    }
}

fn render_audio_settings(ui: &mut egui::Ui, theme: Theme, state: &mut SettingsState) {
    ui.add_space(16.0);

    render_settings_card(ui, theme, "AUDIO OUTPUT", |ui, theme| {
        ui.label(
            egui::RichText::new("Device:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("audio_device")
            .selected_text("Default Output")
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut "default", "default", "Default Output");
                info!("Audio device selection - to be implemented");
            });

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Latency:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);
        ui.label(egui::RichText::new("25ms").color(theme.text_primary()));
    });

    ui.add_space(16.0);

    render_settings_card(ui, theme, "MICROPHONE CALIBRATION", |ui, theme| {
        ui.label(
            egui::RichText::new("Input Gain:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        let slider_response = ui.add(
            egui::Slider::new(&mut state.input_gain, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );

        ui.label(
            egui::RichText::new(format!("{}%", (state.input_gain * 100.0) as i32))
                .color(theme.accent())
                .monospace(),
        );

        if slider_response.changed() {
            info!("Input gain changed to: {}", state.input_gain);
        }

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Noise Gate:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let switch_text = if state.noise_gate_enabled {
                "[ ON ]"
            } else {
                "[ OFF ]"
            };
            let switch_color = if state.noise_gate_enabled {
                theme.accent()
            } else {
                theme.text_muted()
            };

            if ui
                .button(egui::RichText::new(switch_text).color(switch_color))
                .clicked()
            {
                state.noise_gate_enabled = !state.noise_gate_enabled;
                info!("Noise gate toggled: {}", state.noise_gate_enabled);
            }
        });
    });

    ui.add_space(16.0);

    render_settings_card(ui, theme, "THEME OVERRIDE", |ui, theme| {
        ui.radio_value(
            &mut "auto",
            "auto",
            egui::RichText::new("AUTO (System)").color(theme.text_muted()),
        );
        ui.radio_value(
            &mut "tekkadan",
            "tekkadan",
            egui::RichText::new("TEKKADAN (Dark)").color(theme.primary()),
        );
        ui.radio_value(
            &mut "barbatos",
            "barbatos",
            egui::RichText::new("BARBATOS (Light)").color(theme.accent()),
        );

        info!("Theme selection - handled by main theme switcher");
    });

    ui.add_space(24.0);

    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("[ RESET TO FACTORY ]").color(theme.alert()))
            .clicked()
        {
            info!("Reset to factory defaults - to be implemented");
        }

        ui.add_space(8.0);

        if ui
            .button(egui::RichText::new("[ SAVE CONFIG ]").color(theme.primary()))
            .clicked()
        {
            info!("Save configuration - to be implemented");
        }
    });
}

fn render_display_settings(ui: &mut egui::Ui, theme: Theme) {
    ui.add_space(16.0);

    render_settings_card(ui, theme, "HUD OPTIONS", |ui, theme| {
        ui.label(
            egui::RichText::new("Display settings will be implemented here")
                .color(theme.text_muted())
                .italics(),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("• Font size adjustments")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Lyric display options")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• HUD visibility toggles")
                .color(theme.text_muted())
                .size(12.0),
        );
    });
}

fn render_library_settings(ui: &mut egui::Ui, theme: Theme) {
    ui.add_space(16.0);

    render_settings_card(ui, theme, "LIBRARY PATHS", |ui, theme| {
        ui.label(
            egui::RichText::new("Library path configuration")
                .color(theme.text_muted())
                .italics(),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("• Add/remove library folders")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Auto-scan settings")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• File type filters")
                .color(theme.text_muted())
                .size(12.0),
        );
    });
}

fn render_network_settings(ui: &mut egui::Ui, theme: Theme) {
    ui.add_space(16.0);

    render_settings_card(ui, theme, "ALAYA-LINK CONNECTION", |ui, theme| {
        ui.label(
            egui::RichText::new("Network features coming soon")
                .color(theme.text_muted())
                .italics(),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("• Online song database")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Remote library sync")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Multiplayer karaoke")
                .color(theme.text_muted())
                .size(12.0),
        );
    });
}

fn render_settings_card<F>(ui: &mut egui::Ui, theme: Theme, title: &str, content: F)
where
    F: FnOnce(&mut egui::Ui, Theme),
{
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

        content(ui, theme);

        ui.add_space(8.0);
    });
}
