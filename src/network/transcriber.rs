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

        // Try whisper first (more stable and widely compatible)
        if Self::check_whisper() {
            match self.transcribe_with_whisper(audio_path).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    info!("Whisper transcription failed, trying openlrc: {}", e);
                },
            }
        }

        // Fall back to openlrc if whisper isn't available or failed
        if Self::check_openlrc() {
            return self.transcribe_with_openlrc(audio_path).await;
        }

        Err(
            "Neither whisper nor openlrc is installed. Please install one of them:\n\
                - Recommended: pip install openai-whisper\n\
                - Alternative: pip install openlrc"
                .to_string(),
        )
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

        // Log whisper output for debugging
        let stdout_msg = String::from_utf8_lossy(&output.stdout);
        let stderr_msg = String::from_utf8_lossy(&output.stderr);
        info!("Whisper stdout: {}", stdout_msg);
        if !stderr_msg.is_empty() {
            info!("Whisper stderr: {}", stderr_msg);
        }

        // Whisper creates a .srt file in the output directory with the audio file's stem name
        let audio_stem = audio_path.file_stem().ok_or("Cannot get audio file name")?;
        let stem_str = audio_stem.to_string_lossy();
        let lrc_path = audio_path.with_extension("lrc");

        info!("Looking for SRT file with stem: {}", stem_str);

        // Wait a moment for file to be written
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Search for .srt file that matches the audio stem (whisper might add language code like .en.srt)
        let mut srt_path: Option<PathBuf> = None;
        if let Ok(entries) = std::fs::read_dir(output_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                // Look for files that start with the stem and end with .srt
                if file_name_str.starts_with(stem_str.as_ref()) && file_name_str.ends_with(".srt") {
                    srt_path = Some(entry.path());
                    info!("Found SRT file: {}", file_name_str);
                    break;
                }
            }
        }

        if let Some(srt_path) = srt_path {
            info!("Found SRT file at: {}", srt_path.display());
            // Convert SRT to LRC format
            self.convert_srt_to_lrc(&srt_path, &lrc_path)?;

            // Keep the SRT file for now for debugging (don't delete it)
            // let _ = std::fs::remove_file(&srt_path);

            info!(
                "Transcription completed and converted: {}",
                lrc_path.display()
            );
            Ok(lrc_path)
        } else {
            // List files in output directory for debugging
            if let Ok(entries) = std::fs::read_dir(output_dir) {
                let files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().to_str().map(String::from))
                    .collect();
                error!("Files in output directory: {:?}", files);
                error!("Looking for files starting with: {}", stem_str);
            }
            Err(format!("SRT file not found with stem: {}", stem_str))
        }
    }

    /// Convert SRT subtitle format to LRC lyrics format
    fn convert_srt_to_lrc(&self, srt_path: &PathBuf, lrc_path: &PathBuf) -> Result<(), String> {
        let srt_content = std::fs::read_to_string(srt_path)
            .map_err(|e| format!("Failed to read SRT file: {}", e))?;

        info!("SRT file size: {} bytes", srt_content.len());

        // Debug: Save first 500 chars to see format
        let preview = if srt_content.len() > 500 {
            &srt_content[..500]
        } else {
            &srt_content
        };
        info!("SRT preview (first 500 chars): {:?}", preview);

        let mut lrc_lines = vec![
            "[ar:Unknown Artist]".to_string(),
            "[ti:Transcribed Lyrics]".to_string(),
            "[by:PWE Karaoke - Whisper]".to_string(),
            String::new(),
        ];

        // Normalize line endings to \n
        let normalized_content = srt_content.replace("\r\n", "\n");

        // Parse SRT format - blocks are separated by blank lines
        let blocks: Vec<&str> = normalized_content.split("\n\n").collect();
        info!("Found {} SRT blocks to convert", blocks.len());

        for block in blocks {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }

            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 3 {
                info!(
                    "Skipping block with only {} lines: {:?}",
                    lines.len(),
                    lines
                );
                continue;
            }

            // SRT format:
            // 1
            // 00:00:00,000 --> 00:00:05,000
            // Text content (can be multiple lines)

            // Skip the sequence number (first line) - verify it's a number
            if lines[0].trim().parse::<u32>().is_err() {
                info!("Skipping block - first line is not a number: {}", lines[0]);
                continue;
            }

            let timestamp_line = lines[1];

            // Verify timestamp line has the expected format
            if !timestamp_line.contains(" --> ") {
                info!(
                    "Skipping block - invalid timestamp line: {}",
                    timestamp_line
                );
                continue;
            }

            // Extract start timestamp
            if let Some(start_time) = timestamp_line.split(" --> ").next() {
                // Convert SRT timestamp (HH:MM:SS,mmm) to LRC format [MM:SS.xx]
                if let Some(lrc_timestamp) = self.srt_to_lrc_timestamp(start_time) {
                    // Join all text lines (from line 2 onwards)
                    let text = lines[2..].join(" ").trim().to_string();
                    if !text.is_empty() {
                        lrc_lines.push(format!("{}{}", lrc_timestamp, text));
                    }
                } else {
                    info!("Failed to convert timestamp: {}", start_time);
                }
            }
        }

        info!("Converted {} SRT entries to LRC", lrc_lines.len() - 4);

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
