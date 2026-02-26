// Library content filtering

use crate::app::{KaraokeApp, LibraryViewFilter};
use crate::library::Song;

pub fn get_filtered_library(app: &KaraokeApp) -> Vec<Song> {
    match app.library.library_view_filter {
        LibraryViewFilter::AllSongs => app.library.library.clone(),
        LibraryViewFilter::Favorites => get_favorites(&app.library.library),
        LibraryViewFilter::History => get_history_songs(app),
        LibraryViewFilter::Playlist(idx) => get_playlist_songs(app, idx),
    }
}

fn get_favorites(library: &[Song]) -> Vec<Song> {
    library.iter().filter(|s| s.is_favorite).cloned().collect()
}

fn get_history_songs(app: &KaraokeApp) -> Vec<Song> {
    let mut history_songs = Vec::new();
    for history_path in &app.library.play_history {
        if let Some(song) = app.library.library.iter().find(|s| &s.path == history_path) {
            history_songs.push(song.clone());
        }
    }
    history_songs
}

fn get_playlist_songs(app: &KaraokeApp, idx: usize) -> Vec<Song> {
    app.library
        .playlists
        .playlists
        .get(idx)
        .map_or_else(Vec::new, |playlist| {
            let mut playlist_songs = Vec::new();
            for song_path in &playlist.songs {
                if let Some(song) = app.library.library.iter().find(|s| &s.path == song_path) {
                    playlist_songs.push(song.clone());
                }
            }
            playlist_songs
        })
}
