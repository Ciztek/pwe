// Playback action handling

use std::time::Duration;
use tracing::info;

use crate::app::KaraokeApp;
use crate::ui::panels;

pub fn handle_playback_action(app: &mut KaraokeApp, action: panels::PlaybackAction) {
    match action {
        panels::PlaybackAction::PlayPause => handle_play_pause(app),
        panels::PlaybackAction::Stop => handle_stop(app),
        panels::PlaybackAction::SkipForward => handle_skip_forward(app),
        panels::PlaybackAction::SkipBackward => handle_skip_backward(app),
        panels::PlaybackAction::Seek(ratio) => handle_seek(app, ratio),
        panels::PlaybackAction::None => {},
    }
}

fn handle_play_pause(app: &mut KaraokeApp) {
    if app.audio.current_file.is_some() {
        if app.audio.is_playing {
            app.audio.audio_player.pause();
            app.audio.is_playing = false;
            info!("Playback paused");
        } else {
            app.audio.audio_player.resume();
            app.audio.is_playing = true;
            info!("Playback resumed");
        }
    }
}

fn handle_stop(app: &mut KaraokeApp) {
    app.audio.audio_player.clear();
    app.audio.is_playing = false;
    app.audio.current_file = None;
    app.karaoke.clear();
    info!("Playback stopped");
}

fn handle_skip_forward(app: &mut KaraokeApp) {
    if let Some(current_path) = &app.audio.current_file {
        if let Some(current_index) = app
            .library
            .library
            .iter()
            .position(|s| &s.path == current_path)
        {
            let next_index = if current_index + 1 < app.library.library.len() {
                current_index + 1
            } else {
                0
            };

            if app.library.library.len() > 1 || current_index != next_index {
                let next_song_path = app.library.library[next_index].path.clone();
                info!("Skipping to next song: {}", next_song_path.display());
                app.karaoke.clear();
                app.audio.load_and_play_file(next_song_path.clone());
                app.karaoke.load_lyrics(&next_song_path);
            }
        }
    }
}

fn handle_skip_backward(app: &mut KaraokeApp) {
    if let Some(current_path) = &app.audio.current_file {
        if let Some(current_index) = app
            .library
            .library
            .iter()
            .position(|s| &s.path == current_path)
        {
            let prev_index = if current_index > 0 {
                current_index - 1
            } else {
                app.library.library.len().saturating_sub(1)
            };

            if app.library.library.len() > 1 || current_index != prev_index {
                let prev_song_path = app.library.library[prev_index].path.clone();
                info!("Skipping to previous song: {}", prev_song_path.display());
                app.karaoke.clear();
                app.audio.load_and_play_file(prev_song_path.clone());
                app.karaoke.load_lyrics(&prev_song_path);
            }
        }
    }
}

fn handle_seek(app: &mut KaraokeApp, ratio: f32) {
    if let Some(duration) = app.audio.song_duration {
        let new_position = Duration::from_secs_f32(duration.as_secs_f32() * ratio);
        app.seek_to_position(new_position);
        #[allow(clippy::cast_possible_truncation)]
        let percent = (ratio * 100.0) as i32;
        info!("Seeked to {:?} ({}%)", new_position, percent);
    }
}
