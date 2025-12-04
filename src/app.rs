use eframe::egui;
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};
use crate::library::{scanner, Song};
use crate::ui::{panels, settings::SettingsState, theme::Theme, widgets};

pub struct KaraokeApp {
    theme: Theme,
    current_view: AppView,
    settings_state: SettingsState,

    audio_player: AudioPlayer,
    is_playing: bool,
    current_file: Option<PathBuf>,
    error_message: Option<String>,
    song_duration: Option<std::time::Duration>,

    library: Vec<Song>,
    library_path: Option<PathBuf>,
    library_filter: String,
    library_path_input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Library,
    Karaoke,
    Settings,
}

impl KaraokeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        let audio_player: AudioPlayer = AudioPlayer::new();

        if audio_player.is_available() {
            info!("PWE Karaoke initialized successfully with audio");
        } else {
            warn!("PWE Karaoke initialized without audio support");
        }

        Self {
            theme: Theme::Tekkadan,
            current_view: AppView::Library,
            settings_state: SettingsState::default(),
            audio_player,
            is_playing: false,
            current_file: None,
            error_message: None,
            song_duration: None,
            library: Vec::new(),
            library_path: None,
            library_filter: String::new(),
            library_path_input: String::new(),
        }
    }

    fn scan_library(&mut self) {
        match rfd::FileDialog::new()
            .set_title("Select Library Folder")
            .pick_folder()
        {
            Some(path) => {
                info!("Scanning library: {}", path.display());
                self.library = scanner::scan_directory(&path);
                self.library_path = Some(path.clone());
                self.library_path_input = path.display().to_string();
                info!("Library loaded with {} songs", self.library.len());
            },
            None => {
                warn!("File dialog unavailable - use manual path input");
            },
        }
    }

    fn scan_library_from_input(&mut self) {
        if !self.library_path_input.is_empty() {
            let path = PathBuf::from(&self.library_path_input);
            if path.exists() && path.is_dir() {
                info!("Scanning library: {}", path.display());
                self.library = scanner::scan_directory(&path);
                self.library_path = Some(path);
                info!("Library loaded with {} songs", self.library.len());
            } else {
                error!("Invalid path: {}", self.library_path_input);
            }
        }
    }

    #[allow(dead_code)]
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            info!("Selected file: {}", path.display());
            self.load_and_play_file(path);
        }
    }

    fn load_and_play_file(&mut self, path: PathBuf) {
        self.error_message = None;

        self.song_duration = loader::get_audio_duration(&path);

        match loader::load_audio_file(&path) {
            Ok(decoder) => {
                if self.audio_player.is_available() {
                    self.audio_player.clear();

                    if let Some(sink) = self.audio_player.sink() {
                        sink.append(decoder);
                        sink.play();
                    }

                    self.audio_player.start_tracking();

                    self.current_file = Some(path);
                    self.is_playing = true;
                    info!("Started playback");
                }
            },
            Err(e) => {
                error!("Failed to load file: {}", e);
                self.error_message = Some(loader::format_load_error(&e));
            },
        }
    }

    #[allow(dead_code)]
    fn toggle_playback(&mut self) {
        if self.is_playing {
            self.audio_player.pause();
            self.is_playing = false;
            info!("Playback paused");
        } else if self.audio_player.is_paused() {
            self.audio_player.resume();
            self.is_playing = true;
            info!("Playback resumed");
        }
    }

    #[allow(dead_code)]
    fn stop_audio(&mut self) {
        self.audio_player.stop();
        self.is_playing = false;
        self.current_file = None;
        info!("Audio stopped");
    }

    #[allow(dead_code)]
    fn play_beep(&mut self) {
        if let Some(sink) = self.audio_player.sink() {
            info!("Playing test sound");

            if let Some(source) = generator::create_beep(440.0, 200) {
                sink.append(source);
                self.is_playing = true;
            }
        }
    }

    fn setup_fonts(ctx: &egui::Context) {
        #[allow(unused_mut)]
        let mut fonts = egui::FontDefinitions::default();

        #[cfg(feature = "custom-font")]
        {
            fonts.font_data.insert(
                "CaskaydiaMono".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/CaskaydiaMono.ttf")),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "CaskaydiaMono".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "CaskaydiaMono".to_owned());
        }

        #[cfg(not(feature = "custom-font"))]
        {
            info!("Using default egui fonts (custom font not enabled)");
        }

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_playing && self.audio_player.is_empty() {
            self.is_playing = false;
        }

        let current_position = self.audio_player.get_position();

        let current_song_name = self
            .current_file
            .as_ref()
            .and_then(|path| path.file_stem().and_then(|s| s.to_str()));

        let (theme_switched, view_change) =
            panels::render_top_panel(ctx, self.theme, self.current_view);
        if theme_switched {
            self.theme = self.theme.toggle();
            info!("Theme switched to {:?}", self.theme);
        }
        if let Some(new_view) = view_change {
            self.current_view = new_view;
            info!("View changed to {:?}", new_view);
        }

        let playback_action = panels::render_bottom_panel(
            ctx,
            self.is_playing,
            current_position,
            self.song_duration,
            self.theme,
            current_song_name,
        );

        match playback_action {
            panels::PlaybackAction::PlayPause => {
                if self.current_file.is_some() {
                    if self.is_playing {
                        self.audio_player.pause();
                        self.is_playing = false;
                        info!("Playback paused");
                    } else {
                        self.audio_player.resume();
                        self.is_playing = true;
                        info!("Playback resumed");
                    }
                }
            },
            panels::PlaybackAction::Stop => {
                self.audio_player.clear();
                self.is_playing = false;
                self.current_file = None;
                info!("Playback stopped");
            },
            panels::PlaybackAction::SkipForward => {
                info!("Skip forward - to be implemented");
            },
            panels::PlaybackAction::SkipBackward => {
                info!("Skip backward - to be implemented");
            },
            panels::PlaybackAction::None => {},
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.background())
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui: &mut egui::Ui| match self.current_view {
                AppView::Library => self.render_library_view(ui),
                AppView::Karaoke => self.render_karaoke_view(ui),
                AppView::Settings => self.render_settings_view(ui),
            });
    }
}

impl KaraokeApp {
    fn render_library_view(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.library_filter)
                            .desired_width(170.0)
                            .hint_text("Search..."),
                    );
                    ui.label(
                        egui::RichText::new("Q")
                            .color(self.theme.text_muted())
                            .size(11.0),
                    );
                });

                ui.add_space(16.0);

                render_sidebar_section(
                    ui,
                    self.theme,
                    "MY LIBRARY",
                    &["All Songs", "Favorites", "History"],
                );

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("PLAYLISTS")
                        .color(self.theme.text_muted())
                        .size(11.0)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("(to be implemented)")
                        .color(self.theme.text_muted())
                        .size(10.0)
                        .italics(),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                ui.set_max_height(ui.available_height());

                let library_action = widgets::render_library_section(
                    ui,
                    &self.library,
                    self.library_path.as_deref(),
                    &mut self.library_filter,
                    &mut self.library_path_input,
                    self.theme,
                );

                match library_action {
                    widgets::LibraryAction::ScanFolder => self.scan_library(),
                    widgets::LibraryAction::ScanFromInput => self.scan_library_from_input(),
                    widgets::LibraryAction::PlaySong(path) => self.load_and_play_file(path),
                    widgets::LibraryAction::None => {},
                }
            });
        });
        ui.add_space(8.0);
    }

    fn render_karaoke_view(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.label(
                    egui::RichText::new("KARAOKE MODE")
                        .size(32.0)
                        .color(self.theme.primary())
                        .strong(),
                );

                ui.add_space(24.0);

                ui.label(
                    egui::RichText::new("Lyrics display and karaoke HUD")
                        .size(16.0)
                        .color(self.theme.text_muted())
                        .italics(),
                );

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("TO BE IMPLEMENTED")
                        .size(14.0)
                        .color(self.theme.accent())
                        .monospace(),
                );

                ui.add_space(32.0);

                if ui
                    .button(
                        egui::RichText::new("[ Start Karaoke Session ]")
                            .size(16.0)
                            .color(self.theme.primary()),
                    )
                    .clicked()
                {
                    info!("Karaoke session start - to be implemented");
                }
            });
        });
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        crate::ui::settings::render_settings_panel(ui, self.theme, &mut self.settings_state);
    }
}

fn render_sidebar_section(ui: &mut egui::Ui, theme: Theme, title: &str, items: &[&str]) {
    ui.label(
        egui::RichText::new(title)
            .color(theme.text_muted())
            .size(11.0)
            .strong(),
    );

    ui.add_space(8.0);

    for item in items {
        if ui
            .button(
                egui::RichText::new(format!("> {}", item))
                    .color(theme.text_muted())
                    .size(12.0),
            )
            .clicked()
        {
            info!("Sidebar item '{}' clicked - to be implemented", item);
        }
    }
}
