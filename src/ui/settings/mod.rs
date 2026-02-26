// Settings panel modules
mod audio;
mod display;
mod library;
mod network;

use super::actions::SettingsAction;
use super::components::{button, layout};
use super::theme::Theme;
use crate::app::DownloadState;
use crate::audio::devices;
use crate::config::AppConfig;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Audio,
    Display,
    Library,
    Network,
}

pub struct SettingsState {
    current_section: Section,
    pub config: AppConfig,
    available_output_devices: Vec<devices::AudioDevice>,
    new_library_path: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            current_section: Section::Audio,
            config: AppConfig::load(),
            available_output_devices: devices::list_output_devices(),
            new_library_path: String::new(),
        }
    }
}

/// Render settings panel
pub fn render_settings_panel(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    let mut action = None;
    let mut new_section = state.current_section;

    layout::two_column(
        ui,
        200.0,
        |ui| render_sidebar(ui, theme, &mut new_section),
        |ui| {
            action = render_content(ui, theme, state, download_state);
        },
    );

    state.current_section = new_section;
    action
}

fn render_sidebar(ui: &mut egui::Ui, theme: Theme, current: &mut Section) {
    layout::space(ui, layout::SPACE_SM);

    for (section, label) in [
        (Section::Audio, "AUDIO SYSTEM"),
        (Section::Display, "DISPLAY / HUD"),
        (Section::Library, "LIBRARY PATHS"),
        (Section::Network, "ALAYA-LINK (Net)"),
    ] {
        if button::sidebar_button(ui, label, *current == section, theme).clicked() {
            *current = section;
        }
    }
}

fn render_content(
    ui: &mut egui::Ui,
    theme: Theme,
    state: &mut SettingsState,
    download_state: &DownloadState,
) -> Option<SettingsAction> {
    match state.current_section {
        Section::Audio => audio::render(
            ui,
            theme,
            &mut state.config.audio,
            &state.available_output_devices,
        ),
        Section::Display => display::render(ui, theme, &mut state.config.display),
        Section::Library => library::render(
            ui,
            theme,
            &mut state.config.library,
            &mut state.new_library_path,
        ),
        Section::Network => network::render(ui, theme, &mut state.config.network, download_state),
    }
}

/// Common action buttons for settings sections
fn action_buttons(ui: &mut egui::Ui, theme: Theme) -> Option<SettingsAction> {
    let mut action = None;

    layout::action_row(ui, |ui| {
        if button::StyledButton::new("[ RESET TO FACTORY ]")
            .style(button::ButtonStyle::Alert)
            .show(ui, theme)
            .clicked()
        {
            action = Some(SettingsAction::ResetConfig);
        }

        layout::space(ui, layout::SPACE_SM);

        if button::StyledButton::new("[ SAVE CONFIG ]")
            .style(button::ButtonStyle::Primary)
            .show(ui, theme)
            .clicked()
        {
            action = Some(SettingsAction::SaveConfig);
        }
    });

    action
}
