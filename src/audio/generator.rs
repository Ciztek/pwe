// Audio generator - creates synthesized sounds
use rodio::Decoder;
use std::io::Cursor;
use tracing::error;

pub fn create_beep(frequency: f32, duration_ms: u32) -> Option<Decoder<Cursor<Vec<u8>>>> {
    let sample_rate = 44100;

    let samples: Vec<i16> = (0..sample_rate * duration_ms / 1000)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let value = (t * frequency * 2.0 * std::f32::consts::PI).sin();
            (value * i16::MAX as f32 * 0.3) as i16
        })
        .collect();

    let cursor = Cursor::new(create_wav_bytes(&samples, sample_rate));

    match Decoder::new(cursor) {
        Ok(source) => Some(source),
        Err(e) => {
            error!("Failed to create audio source: {}", e);
            None
        }
    }
}

fn create_wav_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (num_samples * 2) as u32;

    let mut bytes = Vec::new();

    // RIFF header
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");

    // fmt chunk
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&num_channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    // PCM data
    for &sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
}
