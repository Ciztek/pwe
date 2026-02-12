use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info};

/// Transcriber using OpenAI Whisper for audio-to-lyrics conversion
#[derive(Clone)]
pub struct Transcriber {
    model: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TranscriptionProgress {
    pub status: TranscriptionStatus,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptionStatus {
    Processing,
    Completed,
    Failed(String),
}

impl Transcriber {
    pub fn new(model: String) -> Self {
        Self { model }
    }

    /// Check if whisper or openlrc is available
    pub fn is_available() -> bool {
        Self::check_openlrc() || Self::check_whisper()
    }

    fn check_openlrc() -> bool {
        // Try direct command first
        if Command::new("openlrc").arg("--version").output().is_ok() {
            return true;
        }
        // Try Python module syntax (Windows compatibility)
        Command::new("python")
            .args(["-m", "openlrc", "--version"])
            .output()
            .is_ok()
    }

    fn check_whisper() -> bool {
        // Try direct command first
        if Command::new("whisper").arg("--version").output().is_ok() {
            return true;
        }
        // Try Python module syntax (Windows compatibility)
        Command::new("python")
            .args(["-m", "whisper", "--version"])
            .output()
            .is_ok()
    }

    /// Transcribe audio file to LRC format
    /// Returns the path to the generated .lrc file
    pub async fn transcribe_to_lrc(&self, audio_path: &Path) -> Result<PathBuf, String> {
        if !audio_path.exists() {
            return Err(format!("Audio file not found: {}", audio_path.display()));
        }

        info!("Starting transcription for: {}", audio_path.display());

        // Try openlrc first (preferred as it's designed for lyrics)
        if Self::check_openlrc() {
            self.transcribe_with_openlrc(audio_path).await
        } else if Self::check_whisper() {
            self.transcribe_with_whisper(audio_path).await
        } else {
            Err(
                "Neither openlrc nor whisper is installed. Please install one of them:\n\
                - openlrc: pip install openlrc\n\
                - whisper: pip install openai-whisper"
                    .to_string(),
            )
        }
    }

    async fn transcribe_with_openlrc(&self, audio_path: &Path) -> Result<PathBuf, String> {
        info!("Using openlrc for transcription with model: {}", self.model);

        // Try direct command first, fall back to Python module syntax
        let mut cmd = Command::new("openlrc");
        let mut use_python_module = false;

        if Command::new("openlrc").arg("--version").output().is_err() {
            // Use Python module syntax instead
            cmd = Command::new("python");
            cmd.args(["-m", "openlrc"]);
            use_python_module = true;
        }

        let output = cmd
            .arg(audio_path.to_string_lossy().as_ref())
            .arg("--whisper-model")
            .arg(&self.model)
            .arg("--format")
            .arg("lrc")
            .output()
            .map_err(|e| {
                if use_python_module {
                    format!("Failed to execute python -m openlrc: {}", e)
                } else {
                    format!("Failed to execute openlrc: {}", e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("openlrc failed: {}", error_msg);
            return Err(format!("Transcription failed: {}", error_msg));
        }

        // openlrc typically creates the .lrc file next to the audio file
        let lrc_path = audio_path.with_extension("lrc");

        if lrc_path.exists() {
            info!("Transcription completed: {}", lrc_path.display());
            Ok(lrc_path)
        } else {
            Err("LRC file was not created".to_string())
        }
    }

    async fn transcribe_with_whisper(&self, audio_path: &Path) -> Result<PathBuf, String> {
        info!("Using whisper for transcription with model: {}", self.model);

        let output_dir = audio_path
            .parent()
            .ok_or("Cannot determine output directory")?;

        // Try direct command first, fall back to Python module syntax
        let mut cmd = Command::new("whisper");
        let mut use_python_module = false;

        if Command::new("whisper").arg("--version").output().is_err() {
            // Use Python module syntax instead
            cmd = Command::new("python");
            cmd.args(["-m", "whisper"]);
            use_python_module = true;
        }

        let output = cmd
            .arg(audio_path.to_string_lossy().as_ref())
            .arg("--model")
            .arg(&self.model)
            .arg("--output_format")
            .arg("srt") // Whisper doesn't support LRC directly, so we use SRT
            .arg("--output_dir")
            .arg(output_dir.to_string_lossy().as_ref())
            .arg("--verbose")
            .arg("False")
            .output()
            .map_err(|e| {
                if use_python_module {
                    format!("Failed to execute python -m whisper: {}", e)
                } else {
                    format!("Failed to execute whisper: {}", e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("whisper failed: {}", error_msg);
            return Err(format!("Transcription failed: {}", error_msg));
        }

        // Whisper creates .srt file, we need to convert it to .lrc
        let srt_path = audio_path.with_extension("srt");
        let lrc_path = audio_path.with_extension("lrc");

        if srt_path.exists() {
            // Convert SRT to LRC format
            self.convert_srt_to_lrc(&srt_path, &lrc_path)?;

            // Clean up the SRT file
            let _ = std::fs::remove_file(&srt_path);

            info!(
                "Transcription completed and converted: {}",
                lrc_path.display()
            );
            Ok(lrc_path)
        } else {
            Err("Transcription file was not created".to_string())
        }
    }

    /// Convert SRT subtitle format to LRC lyrics format
    fn convert_srt_to_lrc(&self, srt_path: &PathBuf, lrc_path: &PathBuf) -> Result<(), String> {
        let srt_content = std::fs::read_to_string(srt_path)
            .map_err(|e| format!("Failed to read SRT file: {}", e))?;

        let lrc_lines = vec![
            "[ar:Unknown Artist]".to_string(),
            "[ti:Transcribed Lyrics]".to_string(),
            "[by:PWE Karaoke - Whisper]".to_string(),
            String::new(),
        ];

        let mut lrc_lines = lrc_lines;

        // Parse SRT format
        let blocks: Vec<&str> = srt_content.split("\n\n").collect();

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() >= 3 {
                // SRT format:
                // 1
                // 00:00:00,000 --> 00:00:05,000
                // Text content

                let timestamp_line = lines[1];
                if let Some(start_time) = timestamp_line.split(" --> ").next() {
                    // Convert SRT timestamp (HH:MM:SS,mmm) to LRC format [MM:SS.xx]
                    if let Some(lrc_timestamp) = self.srt_to_lrc_timestamp(start_time) {
                        let text = lines[2..].join(" ");
                        lrc_lines.push(format!("{}{}", lrc_timestamp, text));
                    }
                }
            }
        }

        let lrc_content = lrc_lines.join("\n");
        std::fs::write(lrc_path, lrc_content)
            .map_err(|e| format!("Failed to write LRC file: {}", e))?;

        Ok(())
    }

    /// Convert SRT timestamp (HH:MM:SS,mmm) to LRC format [MM:SS.xx]
    fn srt_to_lrc_timestamp(&self, srt_time: &str) -> Option<String> {
        // SRT format: 00:01:23,456
        // LRC format: [01:23.45]

        let parts: Vec<&str> = srt_time.trim().split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        let hours: u32 = parts[0].parse().ok()?;
        let minutes: u32 = parts[1].parse().ok()?;

        let sec_parts: Vec<&str> = parts[2].split(',').collect();
        if sec_parts.len() != 2 {
            return None;
        }

        let seconds: u32 = sec_parts[0].parse().ok()?;
        let milliseconds: u32 = sec_parts[1].parse().ok()?;

        // Convert to total minutes and seconds
        let total_minutes = hours * 60 + minutes;
        let centiseconds = milliseconds / 10;

        Some(format!(
            "[{:02}:{:02}.{:02}]",
            total_minutes, seconds, centiseconds
        ))
    }

    /// Get list of available Whisper models
    #[allow(dead_code)]
    pub fn available_models() -> Vec<String> {
        vec![
            "tiny".to_string(),
            "base".to_string(),
            "small".to_string(),
            "medium".to_string(),
            "large".to_string(),
            "turbo".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_lrc_timestamp() {
        let transcriber = Transcriber::new("turbo".to_string());

        assert_eq!(
            transcriber.srt_to_lrc_timestamp("00:01:23,456"),
            Some("[01:23.45]".to_string())
        );

        assert_eq!(
            transcriber.srt_to_lrc_timestamp("00:00:05,100"),
            Some("[00:05.10]".to_string())
        );

        assert_eq!(
            transcriber.srt_to_lrc_timestamp("01:30:00,000"),
            Some("[90:00.00]".to_string())
        );
    }
}
