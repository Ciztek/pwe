// Audio player - manages audio output and playback
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use tracing::error;

pub struct AudioPlayer {
    _output_stream: Option<OutputStream>,
    sink: Option<Arc<Sink>>,
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
                };
            }
        };

        let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

        Self {
            _output_stream: Some(_stream),
            sink: Some(sink),
        }
    }

    pub fn sink(&self) -> Option<&Arc<Sink>> {
        self.sink.as_ref()
    }

    pub fn is_available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.as_ref().map_or(true, |s| s.empty())
    }

    pub fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
