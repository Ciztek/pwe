// Audio playback management

use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::audio::{generator, loader, player::AudioPlayer};

pub struct Audio {
    #[allow(clippy::struct_field_names)]
    pub audio_player: AudioPlayer,
    pub is_playing: bool,
    pub current_file: Option<PathBuf>,
    pub error_message: Option<String>,
    pub song_duration: Option<Duration>,
}

impl Audio {
    pub fn new() -> Self {
        let audio_player = AudioPlayer::new();

        if audio_player.is_available() {
            info!("Audio player initialized successfully");
        } else {
            warn!("Audio player initialized without audio support");
        }
        Self {
            audio_player,
            is_playing: false,
            current_file: None,
            error_message: None,
            song_duration: None,
        }
    }

    #[allow(dead_code)]
    pub fn toggle_playback(&mut self) {
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
    pub fn stop_audio(&mut self) {
        self.audio_player.stop();
        self.is_playing = false;
        self.current_file = None;
        info!("Audio stopped");
    }

    #[allow(dead_code)]
    pub fn play_beep(&mut self) {
        if let Some(sink) = self.audio_player.sink() {
            info!("Playing test sound");

            if let Some(source) = generator::create_beep(440.0, 200) {
                sink.append(source);
                self.is_playing = true;
            }
        }
    }

    pub fn load_and_play_file(&mut self, path: PathBuf) {
        self.error_message = None;
        self.audio_player.clear();
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
    pub fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .pick_file()
        {
            info!("Selected file: {}", path.display());
            self.load_and_play_file(path);
        }
    }

    pub fn seek_to_position(&mut self, position: Duration) {
        if let Some(path) = self.current_file.clone() {
            let was_playing = self.is_playing;
            self.audio_player.clear();

            match loader::load_audio_file(&path) {
                Ok(decoder) => {
                    if let Some(sink) = self.audio_player.sink() {
                        use rodio::Source;
                        let seeked_source = decoder.skip_duration(position);
                        sink.append(seeked_source);

                        if was_playing {
                            sink.play();
                        } else {
                            sink.pause();
                        }

                        self.audio_player.set_position(position);
                        info!("Seeked to position: {:?}", position);
                    }
                },
                Err(e) => {
                    error!("Failed to seek: {}", e);
                },
            }
        }
    }
}
