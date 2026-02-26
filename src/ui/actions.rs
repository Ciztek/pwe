// Consolidated action enums for UI interactions

use std::path::PathBuf;

/// Library management actions
#[derive(Debug, Clone)]
pub enum LibraryAction {
    None,
    PlaySong(PathBuf),
    AddSong,
    RemoveSong(PathBuf),
    RefreshLibrary,
    ToggleFavorite(PathBuf),
    AddToPlaylist(String, PathBuf),
    RemoveFromPlaylist(String, PathBuf),
    TranscribeLyrics(PathBuf),
}

/// Settings actions
#[derive(Debug, Clone, Copy)]
pub enum SettingsAction {
    SaveConfig,
    ResetConfig,
    RescanLibrary,
    DownloadYouTubePlaylist,
    DownloadSpotifyPlaylist,
}
