use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub display: DisplayConfig,
    pub library: LibraryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub output_device: Option<String>,
    pub input_gain: f32,
    pub noise_gate_enabled: bool,
    pub noise_gate_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub font_size: f32,
    pub show_waveform: bool,
    pub show_pitch_guide: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryConfig {
    pub paths: Vec<PathBuf>,
    pub auto_scan: bool,
    pub file_types: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            display: DisplayConfig::default(),
            library: LibraryConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            output_device: None,
            input_gain: 0.75,
            noise_gate_enabled: true,
            noise_gate_threshold: 0.02,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            show_waveform: true,
            show_pitch_guide: true,
            fullscreen: false,
        }
    }
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            auto_scan: true,
            file_types: vec![
                "mp3".to_string(),
                "flac".to_string(),
                "ogg".to_string(),
                "wav".to_string(),
                "m4a".to_string(),
            ],
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Self::config_path();

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    info!("Loaded configuration from: {}", config_path.display());
                    config
                },
                Err(e) => {
                    error!("Failed to parse config file: {}", e);
                    info!("Using default configuration");
                    Self::default()
                },
            },
            Err(_) => {
                info!("No config file found, using defaults");
                Self::default()
            },
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("Failed to create config directory: {}", e));
            }
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        info!("Saved configuration to: {}", config_path.display());
        Ok(())
    }

    fn config_path() -> PathBuf {
        let config_dir = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("PWE-Karaoke")
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("pwe-karaoke")
        };

        config_dir.join("config.toml")
    }
}
