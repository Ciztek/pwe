// Library view modules
mod header;
mod song_item;
mod song_list;
mod toolbar;

use super::actions::LibraryAction;
use super::components::{card, layout};
use super::theme::Theme;
use crate::app::AppState;
use crate::library::Song;
use eframe::egui;
use std::path::Path;

/// Main library view entry point
pub fn render_library_section(
    ui: &mut egui::Ui,
    library: &[&Song],
    _library_path: Option<&Path>,
    filter: &str,
    add_song_path_input: &mut String,
    app_state: &mut AppState,
    theme: Theme,
    playlists: &crate::playlist::PlaylistCollection,
) -> LibraryAction {
    let mut action = LibraryAction::None;

    card::armor_card(ui, theme, |ui| {
        header::render(ui, library, theme);
        layout::space(ui, layout::SPACE_SM);

        if let Some(toolbar_action) = toolbar::render(ui, add_song_path_input, theme) {
            action = toolbar_action;
        }

        layout::separator(ui);

        let (new_page, list_action) = song_list::render(
            ui,
            library,
            filter,
            app_state.song_pagination,
            app_state,
            theme,
            playlists,
        );

        app_state.song_pagination = new_page;

        if let Some(list_action) = list_action {
            action = list_action;
        }
    });

    action
}
