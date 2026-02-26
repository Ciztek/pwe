// Transcription helpers

use std::path::PathBuf;
use tracing::{error, info};

use crate::app::KaraokeApp;
use crate::network::transcriber::Transcriber;

pub fn transcribe_lyrics(app: &mut KaraokeApp, song_path: PathBuf) {
    info!("Starting lyrics transcription for: {}", song_path.display());

    if !Transcriber::is_available() {
        error!(
            "Transcription tools not available. Please install openlrc or whisper:\n\
            - Recommended: pip install openai-whisper\n\
            - Alternative: pip install openlrc (may have issues on Windows Store Python)"
        );
        return;
    }

    let song_name = song_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    app.transcription_state.is_transcribing = true;
    app.transcription_state.song_name = song_name;
    app.transcription_state.song_path = Some(song_path.clone());

    let model = app.settings_state.config.network.whisper_model.clone();
    let transcriber = Transcriber::new(model);

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Failed to create Tokio runtime: {}", e);
                return;
            },
        };

        runtime.block_on(async move {
            match transcriber.transcribe_to_lrc(&song_path) {
                Ok(lrc_path) => {
                    info!("✓ Lyrics transcription completed: {}", lrc_path.display());
                },
                Err(e) => {
                    error!("✗ Lyrics transcription failed: {}", e);
                },
            }
        });
    });

    info!("Transcription started in background");
}
