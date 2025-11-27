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

/// Get duration of an audio file
pub fn get_audio_duration<P: AsRef<Path>>(path: P) -> Option<std::time::Duration> {
    let path = path.as_ref();

    // Try to get duration using symphonia
    let file = File::open(path).ok()?;
    let mss = symphonia::default::get_probe()
        .format(
            &Default::default(),
            symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default()),
            &Default::default(),
            &Default::default(),
        )
        .ok()?;

    let track = mss.format.default_track()?;
    let time_base = track.codec_params.time_base?;
    let n_frames = track.codec_params.n_frames?;

    let seconds = time_base.calc_time(n_frames).seconds as f64 + time_base.calc_time(n_frames).frac;

    Some(std::time::Duration::from_secs_f64(seconds))
}

/// Get a user-friendly error message for audio loading failures
pub fn format_load_error(err: &anyhow::Error) -> String {
    warn!("Audio loading error: {}", err);
    format!("Could not load audio file:\n{}", err)
}
