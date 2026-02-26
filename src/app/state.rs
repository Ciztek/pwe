// Application state structures

use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::ui::theme::Theme;

#[derive(Default)]
pub struct AppState {
    pub song_pagination: usize,
    pub thumbnail_texture_cache: HashMap<PathBuf, egui::TextureHandle>,
    pub fps_smooth: f32,
    pub last_frame_time: Option<std::time::Instant>,
    pub textures_loaded_this_frame: usize,
}

pub struct UI {
    pub theme: Theme,
    pub current_view: AppView,
}

impl UI {
    pub const fn new() -> Self {
        Self {
            theme: Theme::Tekkadan,
            current_view: AppView::Library,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Library,
    Karaoke,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryViewFilter {
    AllSongs,
    Favorites,
    History,
    Playlist(usize), // Index into playlists collection
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}
