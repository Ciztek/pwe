// Library settings section
use super::super::actions::SettingsAction;
use super::super::components::{button, card, input, layout};
use super::super::theme::Theme;
use crate::config::LibraryConfig;
use eframe::egui;
use std::path::PathBuf;

/// Render library settings section
pub fn render(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut LibraryConfig,
    new_path: &mut String,
) -> Option<SettingsAction> {
    layout::space(ui, layout::SPACE_LG);
    render_paths_card(ui, theme, config, new_path);
    layout::space(ui, layout::SPACE_LG);
    render_scan_settings(ui, theme, config)
}

fn render_paths_card(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut LibraryConfig,
    new_path: &mut String,
) {
    card::settings_card(ui, theme, "LIBRARY PATHS", |ui, theme| {
        layout::subsection_label(ui, "Managed Folders:", theme);
        let to_remove = render_path_list(ui, theme, &config.paths);

        if let Some(idx) = to_remove {
            config.paths.remove(idx);
        }

        layout::space(ui, layout::SPACE_SM);
        render_add_path_controls(ui, theme, config, new_path);
    });
}

fn render_path_list(ui: &mut egui::Ui, theme: Theme, paths: &[PathBuf]) -> Option<usize> {
    let mut to_remove = None;

    if paths.is_empty() {
        layout::hint_text(ui, "No folders added yet", theme);
        layout::space(ui, layout::SPACE_XS);
    } else {
        for (idx, path) in paths.iter().enumerate() {
            ui.horizontal(|ui| {
                layout::info_text(ui, &format!("📁 {}", path.display()), theme);
                if button::StyledButton::new("[ X ]")
                    .style(button::ButtonStyle::Alert)
                    .tooltip("Remove folder")
                    .show(ui, theme)
                    .clicked()
                {
                    to_remove = Some(idx);
                }
            });
        }
    }

    to_remove
}

fn render_add_path_controls(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut LibraryConfig,
    new_path: &mut String,
) {
    ui.horizontal(|ui| {
        ui.text_edit_singleline(new_path);

        if button::StyledButton::new("[ Browse ]")
            .style(button::ButtonStyle::Primary)
            .show(ui, theme)
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                *new_path = path.display().to_string();
            }
        }

        if button::StyledButton::new("[ Add ]")
            .style(button::ButtonStyle::Accent)
            .enabled(!new_path.is_empty())
            .show(ui, theme)
            .clicked()
        {
            let path = PathBuf::from(&*new_path);
            if path.exists() {
                config.paths.push(path);
                new_path.clear();
            }
        }
    });
}

fn render_scan_settings(
    ui: &mut egui::Ui,
    theme: Theme,
    config: &mut LibraryConfig,
) -> Option<SettingsAction> {
    let mut action = None;

    card::settings_card(ui, theme, "SCAN SETTINGS", |ui, theme| {
        input::checkbox(ui, "Auto-scan on startup", &mut config.auto_scan, theme);
        layout::space(ui, layout::SPACE_MD);

        layout::subsection_label(ui, "Supported File Types:", theme);
        ui.horizontal_wrapped(|ui| {
            for file_type in &config.file_types {
                ui.label(
                    egui::RichText::new(format!(".{file_type}"))
                        .color(theme.accent())
                        .monospace(),
                );
            }
        });

        layout::space(ui, layout::SPACE_SM);

        if button::StyledButton::new("[ RESCAN NOW ]")
            .style(button::ButtonStyle::Primary)
            .show(ui, theme)
            .clicked()
        {
            action = Some(SettingsAction::RescanLibrary);
        }
    });

    if action.is_none() {
        action = super::action_buttons(ui, theme);
    }

    action
}
