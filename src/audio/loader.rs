// Audio file loader - loads audio files using symphonia
use anyhow::{Context, Result};
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{info, warn};

/// Load an audio file and return a Decoder ready for playback
pub fn load_audio_file<P: AsRef<Path>>(path: P) -> Result<Decoder<BufReader<File>>> {
    let path = path.as_ref();

    info!("Loading audio file: {}", path.display());

    // Open the file
    let file =
        File::open(path).context(format!("Failed to open audio file: {}", path.display()))?;

    let buf_reader = BufReader::new(file);

    // Create decoder (rodio will use symphonia internally)
    let decoder = Decoder::new(buf_reader)
        .context(format!("Failed to decode audio file: {}", path.display()))?;

    info!("Successfully loaded audio file: {}", path.display());

    Ok(decoder)
}

/// Get a user-friendly error message for audio loading failures
pub fn format_load_error(err: &anyhow::Error) -> String {
    warn!("Audio loading error: {}", err);
    format!("Could not load audio file:\n{}", err)
}
