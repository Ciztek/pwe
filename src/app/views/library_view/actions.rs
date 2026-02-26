// Library action handling

use std::path::{Path, PathBuf};
use tracing::{error, info};

use crate::app::KaraokeApp;
use crate::ui::widgets;

pub fn handle_library_action(app: &mut KaraokeApp, action: widgets::LibraryAction) {
    match action {
        widgets::LibraryAction::PlaySong(ref path) => handle_play_song(app, path),
        widgets::LibraryAction::AddSong => app.library.add_song_dialog(),
        widgets::LibraryAction::RemoveSong(ref path) => handle_remove_song(app, path),
        widgets::LibraryAction::RefreshLibrary => app.library.refresh_library(),
        widgets::LibraryAction::ToggleFavorite(path) => app.library.toggle_favorite(&path),
        widgets::LibraryAction::AddToPlaylist(playlist_name, song_path) => {
            handle_add_to_playlist(app, &playlist_name, &song_path);
        },
        widgets::LibraryAction::RemoveFromPlaylist(playlist_name, song_path) => {
            handle_remove_from_playlist(app, &playlist_name, &song_path);
        },
        widgets::LibraryAction::TranscribeLyrics(path) => app.transcribe_lyrics(path),
        widgets::LibraryAction::None => {},
    }
}

fn handle_play_song(app: &mut KaraokeApp, path: &PathBuf) {
    app.library.add_to_history(path);

    app.settings_state.config.library.play_history = app
        .library
        .play_history
        .iter()
        .filter_map(|p| p.to_str().map(String::from))
        .collect();

    if let Err(e) = app.settings_state.config.save() {
        error!("Failed to save config with play history: {}", e);
    }

    app.karaoke.clear();
    app.audio.load_and_play_file(path.clone());
    app.karaoke.load_lyrics(path);
}

fn handle_remove_song(app: &mut KaraokeApp, path: &PathBuf) {
    if let Some(song) = app
        .library
        .library
        .iter()
        .find(|s| s.path == *path)
        .cloned()
    {
        app.library.remove_from_library(&song);
    }
}

fn handle_add_to_playlist(app: &mut KaraokeApp, playlist_name: &str, song_path: &Path) {
    if let Some(playlist) = app.library.playlists.get_playlist_mut(playlist_name) {
        playlist.add_song(song_path.to_path_buf());
        if let Err(e) = app.library.playlists.save() {
            error!("Failed to save playlists: {}", e);
        }
        info!(
            "Added song to playlist '{}': {}",
            playlist_name,
            song_path.display()
        );
    }
}

fn handle_remove_from_playlist(app: &mut KaraokeApp, playlist_name: &str, song_path: &Path) {
    if let Some(playlist) = app.library.playlists.get_playlist_mut(playlist_name) {
        playlist.remove_song(&song_path.to_path_buf());
        if let Err(e) = app.library.playlists.save() {
            error!("Failed to save playlists: {}", e);
        }
        info!(
            "Removed song from playlist '{}': {}",
            playlist_name,
            song_path.display()
        );
    }
}
