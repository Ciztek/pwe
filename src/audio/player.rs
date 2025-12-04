use rodio::{OutputStream, Sink};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;

/// Manages audio playback with play/pause/stop controls and timing tracking.
///
/// The sink is an audio queue that handles mixing, volume, and playback state.
/// Timing is tracked separately to handle pause/resume correctly.
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
            self.accumulated_time
        } else if let Some(start) = self.start_time {
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
