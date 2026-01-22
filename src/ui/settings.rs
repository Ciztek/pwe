use super::theme::Theme;
use crate::audio::devices;
use crate::config::AppConfig;
use eframe::egui;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone)]
pub enum SettingsAction {
    SaveConfig,
    ResetConfig,
    RescanLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Audio,
    Display,
    Library,
    Network,
}

pub struct SettingsState {
    current_section: SettingsSection,
    pub config: AppConfig,
    available_output_devices: Vec<devices::AudioDevice>,
    new_library_path: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            current_section: SettingsSection::Audio,
            config: AppConfig::load(),
            available_output_devices: devices::list_output_devices(),
            new_library_path: String::new(),
        }
    }
}

pub fn render_settings_panel(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    let mut action = None;

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

            action = match state.current_section {
                SettingsSection::Audio => render_audio_settings(ui, theme, state),
                SettingsSection::Display => render_display_settings(ui, theme, state),
                SettingsSection::Library => render_library_settings(ui, theme, state),
                SettingsSection::Network => render_network_settings(ui, theme),
            };
        });
    });

    action
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

fn render_audio_settings(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    let mut action = None;
    ui.add_space(16.0);

    render_settings_card(ui, theme, "AUDIO OUTPUT", |ui, theme| {
        ui.label(
            egui::RichText::new("Device:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        let selected_device = state
            .config
            .audio
            .output_device
            .clone()
            .unwrap_or_else(|| "Default Output".to_string());

        egui::ComboBox::from_id_salt("audio_device")
            .selected_text(&selected_device)
            .show_ui(ui, |ui| {
                for device in &state.available_output_devices {
                    let label = if device.is_default {
                        format!("{} (Default)", device.name)
                    } else {
                        device.name.clone()
                    };

                    if ui
                        .selectable_value(
                            &mut state.config.audio.output_device,
                            Some(device.name.clone()),
                            label,
                        )
                        .clicked()
                    {
                        info!("Selected audio device: {}", device.name);
                    }
                }
            });

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Latency:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);
        ui.label(egui::RichText::new("~25ms (auto)").color(theme.text_primary()));
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
            egui::Slider::new(&mut state.config.audio.input_gain, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );

        ui.label(
            egui::RichText::new(format!(
                "{}%",
                (state.config.audio.input_gain * 100.0) as i32
            ))
            .color(theme.accent())
            .monospace(),
        );

        if slider_response.changed() {
            info!("Input gain changed to: {}", state.config.audio.input_gain);
        }

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Noise Gate:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let switch_text = if state.config.audio.noise_gate_enabled {
                "[ ON ]"
            } else {
                "[ OFF ]"
            };
            let switch_color = if state.config.audio.noise_gate_enabled {
                theme.accent()
            } else {
                theme.text_muted()
            };

            if ui
                .button(egui::RichText::new(switch_text).color(switch_color))
                .clicked()
            {
                state.config.audio.noise_gate_enabled = !state.config.audio.noise_gate_enabled;
                info!(
                    "Noise gate toggled: {}",
                    state.config.audio.noise_gate_enabled
                );
            }
        });

        if state.config.audio.noise_gate_enabled {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Threshold:")
                    .color(theme.text_muted())
                    .size(12.0),
            );
            ui.add(
                egui::Slider::new(&mut state.config.audio.noise_gate_threshold, 0.0..=0.1)
                    .text("Level")
                    .trailing_fill(true),
            );
        }
    });

    ui.add_space(16.0);

    render_settings_card(ui, theme, "THEME OVERRIDE", |ui, theme| {
        ui.label(
            egui::RichText::new("Theme settings controlled in main view")
                .color(theme.text_muted())
                .italics()
                .size(12.0),
        );
    });

    ui.add_space(24.0);

    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("[ RESET TO FACTORY ]").color(theme.alert()))
            .clicked()
        {
            action = Some(SettingsAction::ResetConfig);
        }

        ui.add_space(8.0);

        if ui
            .button(egui::RichText::new("[ SAVE CONFIG ]").color(theme.primary()))
            .clicked()
        {
            action = Some(SettingsAction::SaveConfig);
        }
    });

    action
}

fn render_display_settings(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    let mut action = None;
    ui.add_space(16.0);

    render_settings_card(ui, theme, "HUD OPTIONS", |ui, theme| {
        ui.label(
            egui::RichText::new("Font Size:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut state.config.display.font_size, 10.0..=32.0)
                    .text("pt")
                    .trailing_fill(true),
            );

            if ui.button("[ Reset ]").clicked() {
                state.config.display.font_size = 16.0;
            }
        });

        ui.add_space(12.0);

        ui.checkbox(
            &mut state.config.display.show_waveform,
            egui::RichText::new("Show Audio Waveform").color(theme.text_primary()),
        );

        ui.add_space(8.0);

        ui.checkbox(
            &mut state.config.display.show_pitch_guide,
            egui::RichText::new("Show Pitch Guide").color(theme.text_primary()),
        );

        ui.add_space(8.0);

        ui.checkbox(
            &mut state.config.display.fullscreen,
            egui::RichText::new("Start in Fullscreen").color(theme.text_primary()),
        );
    });

    ui.add_space(16.0);

    render_settings_card(ui, theme, "LYRICS DISPLAY", |ui, theme| {
        ui.label(
            egui::RichText::new("• Lyrics automatically loaded from .lrc files")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Current/upcoming lines highlighted")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.label(
            egui::RichText::new("• Color customization coming soon")
                .color(theme.text_muted())
                .size(12.0),
        );
    });

    ui.add_space(24.0);

    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("[ RESET TO FACTORY ]").color(theme.alert()))
            .clicked()
        {
            action = Some(SettingsAction::ResetConfig);
        }

        ui.add_space(8.0);

        if ui
            .button(egui::RichText::new("[ SAVE CONFIG ]").color(theme.primary()))
            .clicked()
        {
            action = Some(SettingsAction::SaveConfig);
        }
    });

    action
}

fn render_library_settings(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    let mut action = None;
    ui.add_space(16.0);

    render_settings_card(ui, theme, "LIBRARY PATHS", |ui, theme| {
        ui.label(
            egui::RichText::new("Managed Folders:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        // Display existing paths
        let mut to_remove = None;
        for (idx, path) in state.config.library.paths.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("📁 {}", path.display()))
                        .color(theme.text_primary())
                        .size(11.0),
                );

                if ui
                    .button(egui::RichText::new("[ X ]").color(theme.alert()))
                    .on_hover_text("Remove folder")
                    .clicked()
                {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            state.config.library.paths.remove(idx);
            info!("Removed library path at index {}", idx);
        }

        if state.config.library.paths.is_empty() {
            ui.label(
                egui::RichText::new("No folders added yet")
                    .color(theme.text_muted())
                    .italics()
                    .size(11.0),
            );
        }

        ui.add_space(8.0);

        // Add new path
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.new_library_path);

            if ui
                .button(egui::RichText::new("[ Browse ]").color(theme.primary()))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.new_library_path = path.display().to_string();
                }
            }

            if ui
                .button(egui::RichText::new("[ Add ]").color(theme.accent()))
                .clicked()
                && !state.new_library_path.is_empty()
            {
                let path = PathBuf::from(&state.new_library_path);
                if path.exists() {
                    state.config.library.paths.push(path);
                    state.new_library_path.clear();
                    info!("Added new library path");
                }
            }
        });
    });

    ui.add_space(16.0);

    render_settings_card(ui, theme, "SCAN SETTINGS", |ui, theme| {
        ui.checkbox(
            &mut state.config.library.auto_scan,
            egui::RichText::new("Auto-scan on startup").color(theme.text_primary()),
        );

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Supported File Types:")
                .color(theme.text_muted())
                .size(12.0),
        );
        ui.add_space(4.0);

        ui.horizontal_wrapped(|ui| {
            for file_type in &state.config.library.file_types {
                ui.label(
                    egui::RichText::new(format!(".{}", file_type))
                        .color(theme.accent())
                        .monospace(),
                );
            }
        });

        ui.add_space(8.0);

        if ui
            .button(egui::RichText::new("[ RESCAN NOW ]").color(theme.primary()))
            .clicked()
        {
            action = Some(SettingsAction::RescanLibrary);
        }
    });

    ui.add_space(24.0);

    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("[ RESET TO FACTORY ]").color(theme.alert()))
            .clicked()
        {
            action = Some(SettingsAction::ResetConfig);
        }

        ui.add_space(8.0);

        if ui
            .button(egui::RichText::new("[ SAVE CONFIG ]").color(theme.primary()))
            .clicked()
        {
            action = Some(SettingsAction::SaveConfig);
        }
    });

    action
}

fn render_network_settings(ui: &mut egui::Ui, theme: Theme) -> Option<SettingsAction> {
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

    None
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
