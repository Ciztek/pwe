use super::theme::Theme;
use crate::library::Song;
use eframe::egui;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioAction {
    None,
    OpenFile,
    Play,
    PlayPause,
    Stop,
}

#[derive(Debug, Clone)]
pub enum LibraryAction {
    None,
    ScanFolder,
    PlaySong(std::path::PathBuf),
}

/// Renders the file playback control panel with load/play/pause/stop buttons.
///
/// # Parameters
/// - `ui`: egui UI context
/// - `is_playing`: Current playback state
/// - `current_file`: Path to currently loaded file (if any)
/// - `error_message`: Error message to display (if any)
/// - `theme`: Color theme for UI elements
///
/// # Returns
/// AudioAction indicating which button was clicked (None if no interaction)
pub fn render_file_playback_section(
    ui: &mut egui::Ui,
    is_playing: bool,
    current_file: Option<&Path>,
    error_message: Option<&str>,
    theme: Theme,
) -> AudioAction {
    ui.horizontal(|ui| {
        ui.colored_label(theme.primary(), "■");
        ui.heading(
            egui::RichText::new("PLAYBACK CONTROL")
                .color(theme.text_primary())
                .strong(),
        );
    });
    ui.add_space(10.0);

    let mut action = AudioAction::None;

    if let Some(path) = current_file {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(theme.active(), "▶");
                ui.label(
                    egui::RichText::new("TRACK:")
                        .color(theme.text_muted())
                        .small(),
                );
                ui.add_space(5.0);
                ui.monospace(
                    egui::RichText::new(
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("UNKNOWN"),
                    )
                    .color(theme.text_primary()),
                );
            });
        });
        ui.add_space(10.0);
    }

    if let Some(error) = error_message {
        ui.colored_label(theme.primary(), format!("⚠ ERROR: {}", error));
        ui.add_space(10.0);
    }

    ui.horizontal(|ui| {
        let open_btn = egui::Button::new(egui::RichText::new("⊡ LOAD FILE").color(theme.primary()));
        if ui.add(open_btn).clicked() {
            action = AudioAction::OpenFile;
        }

        ui.add_space(10.0);

        let play_pause_text = if is_playing { "⊟ PAUSE" } else { "▶ PLAY" };
        let play_enabled = current_file.is_some();
        let play_color = if play_enabled {
            theme.active()
        } else {
            egui::Color32::DARK_GRAY
        };

        let play_btn = egui::Button::new(egui::RichText::new(play_pause_text).color(play_color));
        if ui.add_enabled(play_enabled, play_btn).clicked() {
            action = AudioAction::PlayPause;
        }

        ui.add_space(10.0);

        let stop_color = if current_file.is_some() {
            theme.primary()
        } else {
            egui::Color32::DARK_GRAY
        };
        let stop_btn = egui::Button::new(egui::RichText::new("⊠ STOP").color(stop_color));
        if ui.add_enabled(current_file.is_some(), stop_btn).clicked() {
            action = AudioAction::Stop;
        }
    });

    ui.add_space(10.0);

    if current_file.is_none() && error_message.is_none() {
        ui.label(
            egui::RichText::new("[SYSTEM READY] Select audio file to begin")
                .color(theme.text_muted())
                .italics(),
        );
    }

    action
}

pub fn render_text_section(ui: &mut egui::Ui, user_text: &mut String) {
    ui.heading("Welcome to PWE Karaoke!");
    ui.add_space(20.0);

    ui.horizontal(|ui| {
        ui.label("Current text:");
        ui.monospace(&*user_text);
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Edit text:");
        ui.text_edit_singleline(user_text);
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);
}

pub fn render_counter_section(ui: &mut egui::Ui, counter: &mut i32) {
    ui.heading("Counter Demo");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        if ui.button("➖ Decrement").clicked() {
            *counter -= 1;
            info!("Counter decremented to {}", counter);
        }

        ui.add_space(10.0);
        ui.label(format!("Value: {}", counter));
        ui.add_space(10.0);

        if ui.button("➕ Increment").clicked() {
            *counter += 1;
            info!("Counter incremented to {}", counter);
        }

        ui.add_space(10.0);

        if ui.button("🔄 Reset").clicked() {
            *counter = 0;
            info!("Counter reset");
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);
}

pub fn render_audio_section(ui: &mut egui::Ui, is_playing: bool) -> AudioAction {
    ui.heading("Audio Test");
    ui.add_space(10.0);

    let mut action = AudioAction::None;

    ui.horizontal(|ui| {
        let button_text = if is_playing {
            "🔊 Playing..."
        } else {
            "🔔 Play Beep"
        };

        if ui
            .add_enabled(!is_playing, egui::Button::new(button_text))
            .clicked()
        {
            action = AudioAction::Play;
        }

        ui.add_space(10.0);

        if ui
            .add_enabled(is_playing, egui::Button::new("⏹ Stop"))
            .clicked()
        {
            action = AudioAction::Stop;
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);

    action
}

pub fn render_info_section(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("ℹ️ About this demo:");
        ui.add_space(5.0);
        ui.label("• Edit text in the input field");
        ui.label("• Use counter buttons to increment/decrement");
        ui.label("• Play a test beep sound (440Hz A4 note)");
        ui.label("• Close the window to exit the application");
    });
}

pub fn render_library_section(
    ui: &mut egui::Ui,
    library: &[Song],
    library_path: Option<&Path>,
    filter: &mut String,
    theme: Theme,
) -> LibraryAction {
    ui.horizontal(|ui| {
        ui.colored_label(theme.primary(), "■");
        ui.heading(
            egui::RichText::new("[Library folder doesn't exist]")
                .color(theme.text_primary())
                .strong(),
        );
    });
    ui.add_space(10.0);

    let mut action = LibraryAction::None;

    ui.horizontal(|ui| {
        if let Some(path) = library_path {
            ui.label(
                egui::RichText::new(format!("PATH: {}", path.display()))
                    .color(theme.text_muted())
                    .small(),
            );
            ui.add_space(5.0);
            ui.colored_label(theme.active(), format!("[{} TRACKS]", library.len()));
        } else {
            ui.label(
                egui::RichText::new("[NO DATABASE LOADED]")
                    .color(theme.text_muted())
                    .italics(),
            );
        }

        ui.add_space(10.0);

        let scan_btn =
            egui::Button::new(egui::RichText::new("⊡ SCAN DIRECTORY").color(theme.primary()));
        if ui.add(scan_btn).clicked() {
            action = LibraryAction::ScanFolder;
        }
    });

    ui.add_space(10.0);

    if !library.is_empty() {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("FILTER:")
                    .color(theme.text_muted())
                    .small(),
            );
            ui.text_edit_singleline(filter);
        });

        ui.add_space(10.0);

        let filtered_songs: Vec<&Song> = if filter.is_empty() {
            library.iter().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            library
                .iter()
                .filter(|song| song.name.to_lowercase().contains(&filter_lower))
                .collect()
        };

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                if filtered_songs.is_empty() {
                    ui.label(
                        egui::RichText::new("[NO MATCHES FOUND]")
                            .color(theme.text_muted())
                            .italics(),
                    );
                    return;
                }

                for (idx, song) in filtered_songs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:03}", idx + 1))
                                .color(theme.text_muted())
                                .small(),
                        );

                        ui.add_space(5.0);

                        let track_btn = egui::Button::new(
                            egui::RichText::new(&song.name).color(theme.text_primary()),
                        );
                        if ui.add(track_btn).clicked() {
                            action = LibraryAction::PlaySong(song.path.clone());
                        }

                        ui.label(
                            egui::RichText::new(format!("[{}]", song.extension.to_uppercase()))
                                .small()
                                .color(theme.text_muted()),
                        );
                    });
                }
            });
    }

    action
}
