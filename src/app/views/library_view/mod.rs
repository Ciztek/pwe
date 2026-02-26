// Library view module - simplified coordinator

mod actions;
mod filters;
mod sidebar;
mod transcription;

use eframe::egui;

use crate::app::KaraokeApp;
use crate::library::Song;
use crate::ui::widgets;

pub fn render_library_view(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.horizontal_top(|ui| {
        sidebar::render_sidebar(app, ui);

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        render_main_content(app, ui);
    });
    ui.add_space(8.0);
}

fn render_main_content(app: &mut KaraokeApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.set_min_width(ui.available_width());
        ui.set_max_height(ui.available_height());

        transcription::render_transcription_status(app, ui);

        let filtered_library = filters::get_filtered_library(app);
        let filtered_refs: Vec<&Song> = filtered_library.iter().collect();

        let library_action = widgets::render_library_section(
            ui,
            &filtered_refs,
            app.library.library_path.as_deref(),
            &app.library.library_filter,
            &mut app.library.add_song_path_input,
            &mut app.app_state,
            app.ui.theme,
            &app.library.playlists,
        );

        actions::handle_library_action(app, library_action);
    });
}
