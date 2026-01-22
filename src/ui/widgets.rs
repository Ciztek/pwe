use super::theme::Theme;
use crate::library::Song;
use eframe::egui;
use std::path::Path;

pub fn render_armor_card<R>(
    ui: &mut egui::Ui,
    theme: Theme,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let frame = egui::Frame::none()
        .fill(theme.card_surface())
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, theme.secondary()));

    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(3.0, ui.available_height()),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.primary());

                ui.add_space(8.0);

                ui.vertical(|ui| add_contents(ui)).inner
            })
            .inner
        })
        .inner
}

#[allow(dead_code)]
pub fn render_file_playback_section(
    ui: &mut egui::Ui,
    is_playing: bool,
    current_file: Option<&Path>,
    error_message: Option<&str>,
    theme: Theme,
) -> AudioAction {
    let mut action = AudioAction::None;

    render_armor_card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let status_text = if is_playing { "▶" } else { "■" };
            let status_color = if is_playing {
                theme.accent()
            } else {
                theme.secondary()
            };

            ui.label(
                egui::RichText::new(status_text)
                    .size(18.0)
                    .color(status_color)
                    .strong(),
            );

            ui.label(
                egui::RichText::new("PLAYBACK CONTROL")
                    .size(16.0)
                    .color(theme.primary())
                    .strong(),
            );
        });

        ui.add_space(12.0);

        if let Some(path) = current_file {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("TRACK:")
                        .color(theme.text_muted())
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new(
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("[UNKNOWN]"),
                    )
                    .color(theme.text_primary())
                    .size(13.0)
                    .monospace(),
                );
            });

            ui.add_space(8.0);
        } else if error_message.is_none() {
            ui.label(
                egui::RichText::new("[ SYSTEM READY - SELECT AUDIO FILE ]")
                    .color(theme.text_muted())
                    .italics()
                    .size(12.0),
            );
            ui.add_space(8.0);
        }

        if let Some(error) = error_message {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").color(theme.alert()).size(14.0));
                ui.label(egui::RichText::new(error).color(theme.alert()).size(12.0));
            });
            ui.add_space(8.0);
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let load_btn = egui::Button::new(
                egui::RichText::new("[ LOAD FILE ]")
                    .color(theme.primary())
                    .size(13.0),
            );

            if ui.add(load_btn).clicked() {
                action = AudioAction::OpenFile;
            }

            ui.add_space(12.0);

            let play_enabled = current_file.is_some();
            let play_text = if is_playing { "[ PAUSE ]" } else { "[ PLAY ]" };
            let play_color = if play_enabled {
                theme.accent()
            } else {
                theme.text_muted()
            };

            let play_btn =
                egui::Button::new(egui::RichText::new(play_text).color(play_color).size(13.0));

            if ui.add_enabled(play_enabled, play_btn).clicked() {
                action = AudioAction::PlayPause;
            }

            ui.add_space(12.0);

            let stop_enabled = current_file.is_some();
            let stop_color = if stop_enabled {
                theme.secondary()
            } else {
                theme.text_muted()
            };

            let stop_btn =
                egui::Button::new(egui::RichText::new("[ STOP ]").color(stop_color).size(13.0));

            if ui.add_enabled(stop_enabled, stop_btn).clicked() {
                action = AudioAction::Stop;
            }
        });
    });

    action
}

pub fn render_library_section(
    ui: &mut egui::Ui,
    library: &[Song],
    _library_path: Option<&Path>,
    filter: &mut str,
    add_song_path_input: &mut String,
    current_page: usize,
    theme: Theme,
) -> (LibraryAction, usize) {
    let mut action = LibraryAction::None;
    let mut page = current_page;

    render_armor_card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let art_size = egui::vec2(80.0, 80.0);
            let (rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, theme.card_surface().gamma_multiply(0.7));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "[ Art ]",
                egui::FontId::proportional(10.0),
                theme.text_muted(),
            );

            ui.add_space(16.0);

            ui.vertical(|ui| {
                if library.is_empty() {
                    ui.label(
                        egui::RichText::new("No Library Loaded")
                            .size(18.0)
                            .color(theme.text_muted())
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Scan a directory to get started")
                            .size(12.0)
                            .color(theme.text_muted())
                            .italics(),
                    );
                } else {
                    let first_song = &library[0];

                    ui.horizontal(|ui| {
                        // Show album art if available
                        if let Some(metadata) = &first_song.metadata {
                            if let Some(cover_data) = &metadata.cover_art {
                                if let Ok(dynamic_image) = image::load_from_memory(cover_data) {
                                    let rgba_image = dynamic_image.to_rgba8();
                                    let size = 48.0;
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        [rgba_image.width() as _, rgba_image.height() as _],
                                        rgba_image.as_raw(),
                                    );
                                    let texture = ui.ctx().load_texture(
                                        format!("library_header_art_{}", first_song.path.display()),
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    );
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(size, size),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().image(
                                        texture.id(),
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                    ui.add_space(8.0);
                                }
                            }
                        }

                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(first_song.display_title())
                                    .size(18.0)
                                    .color(theme.text_primary())
                                    .strong(),
                            );
                            ui.add_space(4.0);

                            // Show artist and album if available
                            if let Some(artist) = first_song.artist() {
                                ui.label(
                                    egui::RichText::new(artist)
                                        .size(13.0)
                                        .color(theme.text_muted()),
                                );
                            }
                            if let Some(album) = first_song.album() {
                                ui.label(
                                    egui::RichText::new(format!("♫ {}", album))
                                        .size(11.0)
                                        .color(theme.text_muted()),
                                );
                            }
                            ui.label(
                                egui::RichText::new(format!("{} tracks loaded", library.len()))
                                    .size(12.0)
                                    .color(theme.text_muted()),
                            );
                        });
                    });
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("[ PLAY ]")
                                .color(theme.accent())
                                .size(13.0),
                        )
                        .clicked()
                        && !library.is_empty()
                    {
                        action = LibraryAction::PlaySong(library[0].path.clone());
                    }

                    ui.add_space(8.0);

                    if ui
                        .button(
                            egui::RichText::new("[ Add to Q ]")
                                .color(theme.secondary())
                                .size(13.0),
                        )
                        .clicked()
                    {
                        tracing::info!("Add to queue - to be implemented");
                    }
                });
            });
        });
    });

    ui.add_space(20.0);

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

            if ui
                .button(
                    egui::RichText::new("[ + Add Song ]")
                        .color(theme.accent())
                        .size(11.0),
                )
                .clicked()
            {
                action = LibraryAction::AddSong;
            }

            ui.add_space(8.0);

            if ui
                .button(
                    egui::RichText::new("[ ↻ Refresh ]")
                        .color(theme.primary())
                        .size(11.0),
                )
                .on_hover_text("Sync library with file system")
                .clicked()
            {
                action = LibraryAction::RefreshLibrary;
            }
        });
    });

    ui.add_space(8.0);

    // Add song path input
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Or add by path:")
                .color(theme.text_muted())
                .size(10.0),
        );

        let path_edit = ui.add(
            egui::TextEdit::singleline(add_song_path_input)
                .desired_width(ui.available_width() - 100.0)
                .hint_text("/path/to/song.mp3")
                .font(egui::TextStyle::Monospace),
        );

        if ui
            .button(
                egui::RichText::new("[ Add ]")
                    .color(theme.accent())
                    .size(10.0),
            )
            .clicked()
            || (path_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            action = LibraryAction::AddSongFromPath;
        }

        ui.add_space(4.0);
    });

    ui.add_space(8.0);

    if !library.is_empty() {
        let filtered_songs: Vec<&Song> = if filter.is_empty() {
            library.iter().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            library
                .iter()
                .filter(|song| {
                    // Search in filename
                    song.name.to_lowercase().contains(&filter_lower)
                    // Search in metadata title
                    || song.display_title().to_lowercase().contains(&filter_lower)
                    // Search in artist
                    || song.artist().map(|a| a.to_lowercase().contains(&filter_lower)).unwrap_or(false)
                    // Search in album
                    || song.album().map(|a| a.to_lowercase().contains(&filter_lower)).unwrap_or(false)
                })
                .collect()
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(ui.available_height().max(100.0))
            .show(ui, |ui| {
                if filtered_songs.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("[ NO MATCHES FOUND ]")
                                .color(theme.text_muted())
                                .italics(),
                        );
                    });
                } else {
                    let page_size = 8;
                    let total_pages = filtered_songs.len().div_ceil(page_size);

                    if page >= total_pages {
                        page = total_pages.saturating_sub(1);
                    }

                    let start_idx = page * page_size;
                    let end_idx = (start_idx + page_size).min(filtered_songs.len());
                    let current_page_songs = &filtered_songs[start_idx..end_idx];

                    for (local_idx, song) in current_page_songs.iter().enumerate() {
                        let idx = start_idx + local_idx;
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{:02}.", idx + 1))
                                        .color(theme.text_muted())
                                        .size(12.0)
                                        .monospace(),
                                );

                                ui.add_space(8.0);

                                // Album art thumbnail if available
                                if let Some(metadata) = &song.metadata {
                                    if let Some(cover_art) = &metadata.cover_art {
                                        if let Ok(image) = image::load_from_memory(cover_art) {
                                            let size = 32.0;
                                            let rgba_image = image.to_rgba8();
                                            let pixels = rgba_image.as_flat_samples();
                                            let color_image =
                                                egui::ColorImage::from_rgba_unmultiplied(
                                                    [
                                                        rgba_image.width() as _,
                                                        rgba_image.height() as _,
                                                    ],
                                                    pixels.as_slice(),
                                                );
                                            let texture = ui.ctx().load_texture(
                                                format!("album_art_{}", song.path.display()),
                                                color_image,
                                                egui::TextureOptions::LINEAR,
                                            );
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(size, size),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().image(
                                                texture.id(),
                                                rect,
                                                egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                ),
                                                egui::Color32::WHITE,
                                            );
                                            ui.add_space(8.0);
                                        }
                                    }
                                }

                                ui.vertical(|ui| {
                                    // Song title
                                    let song_label = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(song.display_title())
                                                .color(theme.text_primary())
                                                .size(13.0),
                                        )
                                        .sense(egui::Sense::click()),
                                    );

                                    if song_label.clicked() {
                                        action = LibraryAction::PlaySong(song.path.clone());
                                    }

                                    if song_label.hovered() {
                                        ui.painter().rect_stroke(
                                            song_label.rect.expand(2.0),
                                            2.0,
                                            egui::Stroke::new(1.0, theme.accent()),
                                        );
                                    }

                                    // Artist info if available
                                    if let Some(artist) = song.artist() {
                                        ui.label(
                                            egui::RichText::new(artist)
                                                .color(theme.text_muted())
                                                .size(10.0),
                                        );
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .button(
                                                egui::RichText::new("[×]")
                                                    .size(14.0)
                                                    .color(theme.error()),
                                            )
                                            .clicked()
                                        {
                                            action = LibraryAction::RemoveSong(song.path.clone());
                                        }

                                        ui.add_space(8.0);

                                        // Show lyrics indicator
                                        let lyrics_icon =
                                            if song.has_lyrics { "🎤" } else { "♪" };
                                        let lyrics_color = if song.has_lyrics {
                                            theme.accent()
                                        } else {
                                            theme.text_muted()
                                        };

                                        ui.label(
                                            egui::RichText::new(lyrics_icon)
                                                .size(13.0)
                                                .color(lyrics_color),
                                        )
                                        .on_hover_text(
                                            if song.has_lyrics {
                                                "Has lyrics file"
                                            } else {
                                                "No lyrics file"
                                            },
                                        );

                                        ui.add_space(8.0);

                                        // Show duration if available
                                        if let Some(duration) = song.duration() {
                                            let minutes = duration / 60;
                                            let seconds = duration % 60;
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}:{:02}",
                                                    minutes, seconds
                                                ))
                                                .size(11.0)
                                                .color(theme.text_muted())
                                                .monospace(),
                                            );
                                        }
                                    },
                                );
                            });
                        });

                        if local_idx < current_page_songs.len() - 1 {
                            ui.add_space(8.0);
                        }
                    }

                    // Pagination controls
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let prev_enabled = page > 0;
                        let prev_color = if prev_enabled {
                            theme.primary()
                        } else {
                            theme.text_muted()
                        };

                        if ui
                            .add_enabled(
                                prev_enabled,
                                egui::Button::new(
                                    egui::RichText::new("Prev").color(prev_color).size(12.0),
                                ),
                            )
                            .clicked()
                        {
                            page = page.saturating_sub(1);
                        }

                        ui.add_space(8.0);

                        // Page indicator
                        ui.label(
                            egui::RichText::new(format!("< {} / {} >", page + 1, total_pages))
                                .color(theme.text_primary())
                                .size(13.0)
                                .monospace(),
                        );

                        ui.add_space(8.0);

                        let next_enabled = page < total_pages - 1;
                        let next_color = if next_enabled {
                            theme.primary()
                        } else {
                            theme.text_muted()
                        };

                        if ui
                            .add_enabled(
                                next_enabled,
                                egui::Button::new(
                                    egui::RichText::new("Next").color(next_color).size(12.0),
                                ),
                            )
                            .clicked()
                        {
                            page = (page + 1).min(total_pages - 1);
                        }

                        if total_pages > 8 {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("Go to page:")
                                    .color(theme.text_muted())
                                    .size(11.0),
                            );

                            let mut page_input_str = (page + 1).to_string();
                            let page_input_edit = ui.add(
                                egui::TextEdit::singleline(&mut page_input_str)
                                    .desired_width(40.0)
                                    .font(egui::TextStyle::Monospace),
                            );

                            if page_input_edit.changed() {
                                if let Ok(input_page) = page_input_str.parse::<usize>() {
                                    if input_page > 0 && input_page <= total_pages {
                                        page = input_page - 1;
                                    }
                                }
                            }
                        }
                    });
                }
            });
    }

    (action, page)
}

#[allow(dead_code)]
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
    PlaySong(std::path::PathBuf),
    AddSong,
    AddSongFromPath,
    RemoveSong(std::path::PathBuf),
    RefreshLibrary,
}
