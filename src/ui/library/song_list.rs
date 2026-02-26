// Song list with pagination
use super::super::actions::LibraryAction;
use super::super::components::layout;
use super::super::theme::Theme;
use super::song_item;
use crate::app::AppState;
use crate::library::Song;
use eframe::egui;

const SONGS_PER_PAGE: usize = 10;

pub fn render(
    ui: &mut egui::Ui,
    library: &[&Song],
    filter: &str,
    mut page: usize,
    app_state: &mut AppState,
    theme: Theme,
    playlists: &crate::playlist::PlaylistCollection,
) -> (usize, Option<LibraryAction>) {
    let filtered = filter_songs(library, filter);
    let total_pages = (filtered.len().max(1) - 1) / SONGS_PER_PAGE + 1;

    page = page.min(total_pages.saturating_sub(1));

    let mut action = None;

    render_pagination_top(ui, page, total_pages, filtered.len(), theme);
    layout::space(ui, layout::SPACE_SM);

    let list_action = render_song_list(ui, &filtered, page, app_state, theme, playlists);
    if list_action.is_some() {
        action = list_action;
    }

    layout::space(ui, layout::SPACE_SM);
    page = render_pagination_controls(ui, page, total_pages, theme);

    (page, action)
}

fn filter_songs<'a>(library: &[&'a Song], filter: &str) -> Vec<&'a Song> {
    if filter.is_empty() {
        library.to_vec()
    } else {
        let filter_lower = filter.to_lowercase();
        library
            .iter()
            .filter(|song| {
                song.display_title().to_lowercase().contains(&filter_lower)
                    || song
                        .artist()
                        .is_some_and(|a| a.to_lowercase().contains(&filter_lower))
            })
            .copied()
            .collect()
    }
}

fn render_pagination_top(
    ui: &mut egui::Ui,
    page: usize,
    total_pages: usize,
    total: usize,
    theme: Theme,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Page {} of {}", page + 1, total_pages))
                .color(theme.text_muted())
                .size(11.0),
        );

        ui.label(
            egui::RichText::new(format!("({total} songs)"))
                .color(theme.text_muted())
                .size(10.0),
        );
    });
}

fn render_song_list(
    ui: &mut egui::Ui,
    songs: &[&Song],
    page: usize,
    app_state: &mut AppState,
    theme: Theme,
    playlists: &crate::playlist::PlaylistCollection,
) -> Option<LibraryAction> {
    let mut action = None;

    egui::ScrollArea::vertical()
        .id_salt("library_scroll")
        .max_height(400.0)
        .show(ui, |ui| {
            let start = page * SONGS_PER_PAGE;
            let end = (start + SONGS_PER_PAGE).min(songs.len());

            for song in &songs[start..end] {
                if let Some(song_action) = song_item::render(ui, song, theme, app_state, playlists)
                {
                    action = Some(song_action);
                }
                ui.separator();
            }
        });

    action
}

fn render_pagination_controls(
    ui: &mut egui::Ui,
    page: usize,
    total_pages: usize,
    theme: Theme,
) -> usize {
    let mut new_page = page;

    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                page > 0,
                egui::Button::new(
                    egui::RichText::new("[ ◀ Previous ]")
                        .color(if page > 0 {
                            theme.primary()
                        } else {
                            theme.text_muted()
                        })
                        .size(11.0),
                ),
            )
            .clicked()
        {
            new_page = page.saturating_sub(1);
        }

        ui.add_space(8.0);

        if ui
            .add_enabled(
                page < total_pages - 1,
                egui::Button::new(
                    egui::RichText::new("[ Next ▶ ]")
                        .color(if page < total_pages - 1 {
                            theme.primary()
                        } else {
                            theme.text_muted()
                        })
                        .size(11.0),
                ),
            )
            .clicked()
        {
            new_page = (page + 1).min(total_pages - 1);
        }
    });

    new_page
}
