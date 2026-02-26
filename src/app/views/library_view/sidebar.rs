// Library view sidebar components

use eframe::egui;
use tracing::{error, info};

use crate::app::{KaraokeApp, LibraryViewFilter};
use crate::ui::theme::Theme;

pub fn render_sidebar(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.set_width(200.0);

        render_search_box(app, ui);
        ui.add_space(16.0);

        render_library_filters(app, ui);
        ui.add_space(16.0);

        render_playlists(app, ui);
    });

    if app.library.show_playlist_dialog {
        render_new_playlist_dialog(app, ui);
    }
}

fn render_search_box(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.library.library_filter)
                .desired_width(170.0)
                .hint_text("Search..."),
        );
        ui.label(
            egui::RichText::new("Q")
                .color(app.ui.theme.text_muted())
                .size(11.0),
        );
    });
}

fn render_library_filters(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    let changed = render_filter_section(
        ui,
        app.ui.theme,
        "MY LIBRARY",
        &["All Songs", "Favorites", "History"],
        &mut app.library.library_view_filter,
    );

    if changed {
        info!(
            "Library view changed to: {:?}",
            app.library.library_view_filter
        );
    }
}

fn render_filter_section(
    ui: &mut egui::Ui,
    theme: Theme,
    title: &str,
    items: &[&str],
    current_filter: &mut LibraryViewFilter,
) -> bool {
    let mut changed = false;

    ui.label(
        egui::RichText::new(title)
            .color(theme.text_muted())
            .size(11.0)
            .strong(),
    );

    ui.add_space(8.0);

    for (idx, item) in items.iter().enumerate() {
        let filter = match idx {
            1 => LibraryViewFilter::Favorites,
            2 => LibraryViewFilter::History,
            _ => LibraryViewFilter::AllSongs,
        };

        let is_active = *current_filter == filter;
        let text_color = if is_active {
            theme.primary()
        } else {
            theme.text_muted()
        };

        let prefix = if is_active { "[ > ] " } else { "[   ] " };

        if ui
            .button(
                egui::RichText::new(format!("{prefix}{item}"))
                    .color(text_color)
                    .size(12.0),
            )
            .clicked()
        {
            *current_filter = filter;
            changed = true;
        }
    }

    changed
}

fn render_playlists(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("PLAYLISTS")
            .color(app.ui.theme.text_muted())
            .size(11.0)
            .strong(),
    );
    ui.add_space(8.0);

    let playlist_info = collect_playlist_info(app);
    let (to_select, to_delete) = render_playlist_list(app, ui, &playlist_info);

    handle_playlist_selection(app, to_select, &playlist_info);
    handle_playlist_deletion(app, to_delete);

    ui.add_space(8.0);
    render_new_playlist_button(app, ui);
}

fn collect_playlist_info(app: &KaraokeApp) -> Vec<(usize, String, usize)> {
    app.library
        .playlists
        .playlists
        .iter()
        .enumerate()
        .map(|(idx, p)| (idx, p.name.clone(), p.songs.len()))
        .collect()
}

fn render_playlist_list(
    app: &KaraokeApp,
    ui: &mut egui::Ui,
    playlist_info: &[(usize, String, usize)],
) -> (Option<usize>, Option<String>) {
    let mut to_select = None;
    let mut to_delete = None;

    egui::ScrollArea::vertical()
        .id_salt("playlist_scroll")
        .max_height(200.0)
        .show(ui, |ui| {
            for (idx, name, song_count) in playlist_info {
                let is_selected = matches!(
                    app.library.library_view_filter,
                    LibraryViewFilter::Playlist(i) if i == *idx
                );
                let text_color = if is_selected {
                    app.ui.theme.primary()
                } else {
                    app.ui.theme.text_muted()
                };

                ui.horizontal(|ui| {
                    let label = format!("♫ {name} ({song_count})");
                    if ui
                        .selectable_label(
                            is_selected,
                            egui::RichText::new(label).color(text_color).size(11.0),
                        )
                        .clicked()
                    {
                        to_select = Some(*idx);
                    }

                    if ui
                        .button(
                            egui::RichText::new("✕")
                                .color(app.ui.theme.alert())
                                .size(10.0),
                        )
                        .on_hover_text("Delete playlist")
                        .clicked()
                    {
                        to_delete = Some(name.clone());
                    }
                });
            }

            if playlist_info.is_empty() {
                ui.label(
                    egui::RichText::new("No playlists")
                        .color(app.ui.theme.text_muted())
                        .size(10.0)
                        .italics(),
                );
            }
        });

    (to_select, to_delete)
}

fn handle_playlist_selection(
    app: &mut KaraokeApp,
    to_select: Option<usize>,
    playlist_info: &[(usize, String, usize)],
) {
    if let Some(idx) = to_select {
        app.library.library_view_filter = LibraryViewFilter::Playlist(idx);
        if let Some((_, name, _)) = playlist_info.get(idx) {
            info!("Selected playlist: {}", name);
        }
    }
}

fn handle_playlist_deletion(app: &mut KaraokeApp, to_delete: Option<String>) {
    if let Some(name) = to_delete {
        app.library.playlists.delete_playlist(&name);
        if let Err(e) = app.library.playlists.save() {
            error!("Failed to save playlists: {}", e);
        }
        if matches!(
            app.library.library_view_filter,
            LibraryViewFilter::Playlist(_)
        ) {
            app.library.library_view_filter = LibraryViewFilter::AllSongs;
        }
        info!("Deleted playlist: {}", name);
    }
}

fn render_new_playlist_button(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    if ui
        .button(egui::RichText::new("[ + New Playlist ]").color(app.ui.theme.accent()))
        .clicked()
    {
        app.library.show_playlist_dialog = true;
    }
}

fn render_new_playlist_dialog(app: &mut KaraokeApp, ui: &egui::Ui) {
    egui::Window::new("Create Playlist")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label("Playlist Name:");
            ui.text_edit_singleline(&mut app.library.new_playlist_name);

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() && !app.library.new_playlist_name.is_empty() {
                    if app
                        .library
                        .playlists
                        .create_playlist(app.library.new_playlist_name.clone())
                    {
                        if let Err(e) = app.library.playlists.save() {
                            error!("Failed to save playlists: {}", e);
                        }
                        info!("Created playlist: {}", app.library.new_playlist_name);
                        app.library.new_playlist_name.clear();
                        app.library.show_playlist_dialog = false;
                    } else {
                        error!(
                            "Playlist with name '{}' already exists",
                            app.library.new_playlist_name
                        );
                    }
                }

                if ui.button("Cancel").clicked() {
                    app.library.new_playlist_name.clear();
                    app.library.show_playlist_dialog = false;
                }
            });
        });
}
