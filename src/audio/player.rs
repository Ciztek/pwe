// Audio player - manages audio output and playback
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;

pub struct AudioPlayer {
    _output_stream: Option<OutputStream>,
    sink: Option<Arc<Sink>>,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    accumulated_time: Duration,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to initialize audio output: {}", e);
                return Self {
                    _output_stream: None,
                    sink: None,
                    start_time: None,
                    pause_time: None,
                    accumulated_time: Duration::ZERO,
                };
            },
        };

        // Create a sink - an audio queue that manages playback of audio sources
        // The sink handles mixing, volume control, and playback state (play/pause/stop)
        // We wrap it in Arc to allow shared ownership across the application
        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                error!("Failed to create audio sink: {}", e);
                return Self {
                    _output_stream: Some(_stream),
                    sink: None,
                    start_time: None,
                    pause_time: None,
                    accumulated_time: Duration::ZERO,
                };
            },
        };

        Self {
            _output_stream: Some(_stream),
            sink: Some(sink),
            start_time: None,
            pause_time: None,
            accumulated_time: Duration::ZERO,
        }
    }

    pub fn start_tracking(&mut self) {
        self.start_time = Some(Instant::now());
        self.pause_time = None;
        self.accumulated_time = Duration::ZERO;
    }

    pub fn get_position(&self) -> Duration {
        if self.pause_time.is_some() {
            // Paused: return accumulated time up to pause
            self.accumulated_time
        } else if let Some(start) = self.start_time {
            // Playing: return accumulated + current elapsed
            self.accumulated_time + start.elapsed()
        } else {
            Duration::ZERO
        }
    }

    pub fn reset_position(&mut self) {
        self.start_time = None;
        self.pause_time = None;
        self.accumulated_time = Duration::ZERO;
    }

    pub fn sink(&self) -> Option<&Arc<Sink>> {
        self.sink.as_ref()
    }

    pub fn is_available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.as_ref().is_none_or(|s| s.empty())
    }

    pub fn is_paused(&self) -> bool {
        self.sink.as_ref().is_some_and(|s| s.is_paused())
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            // Save accumulated time when pausing
            if self.pause_time.is_none() {
                if let Some(start) = self.start_time {
                    self.accumulated_time += start.elapsed();
                    self.pause_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            // Resume timing when resuming playback
            if self.pause_time.is_some() {
                self.start_time = Some(Instant::now());
                self.pause_time = None;
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.reset_position();
    }

    pub fn clear(&mut self) {
        if let Some(sink) = &self.sink {
            sink.clear();
        }
        self.reset_position();
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
