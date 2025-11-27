# PWE Karaoke - Desktop Karaoke Application

A desktop karaoke application inspired by Karafun, featuring automatic vocal separation using Spleeter.

## 📋 Project Overview

This application allows users to:

- Play audio/video karaoke files
- Automatically separate vocals from instrumental tracks using Spleeter
- Display synchronized lyrics
- Control audio playback (play, pause, volume, pitch, tempo)
- Manage a karaoke song library
- Search and filter songs

## 🏗️ Architecture

- **Frontend**: egui/eframe (native Rust GUI framework)
- **Audio Engine**: rodio + symphonia for playback, cpal for low-level audio
- **Vocal Separation**: Spleeter (Python) via PyO3 bindings
- **Async Runtime**: Tokio for background tasks

## 🔧 Prerequisites

### System Requirements

#### 1. Rust Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Python 3.8+ with Spleeter

```bash
# Install Python (Debian/Ubuntu)
sudo apt-get update
sudo apt-get install python3 python3-pip python3-dev python3-venv

# Or on macOS with Homebrew
brew install python@3.11

# Install Spleeter in a virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install spleeter
```

#### 3. System Audio Libraries

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get install -y \
    libasound2-dev \
    libgtk-3-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libspeechd-dev \
    libxkbcommon-dev \
    libssl-dev \
    libpython3-dev \
    pkg-config \
    ffmpeg
```

**Linux (Fedora):**

```bash
sudo dnf install -y \
    alsa-lib-devel \
    gtk3-devel \
    python3-devel \
    openssl-devel \
    pkg-config \
    ffmpeg
```

**macOS:**

```bash
brew install python3 pkg-config ffmpeg
```

**Windows:**

- Install [Python 3.8+](https://www.python.org/downloads/)
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Install [ffmpeg](https://ffmpeg.org/download.html) and add to PATH

#### 4. FFmpeg (Required by Spleeter)

FFmpeg must be installed and available in your system PATH for Spleeter to work properly.

## 🚀 Getting Started

### 1. Clone and Setup

```bash
# Clone the repository
git clone <repository-url>
cd pwe

# Set up Python virtual environment for Spleeter
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install Spleeter
pip install spleeter

# Note: Keep the virtual environment activated when running the app
```

### 2. Build the Project

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### 3. Run the Application

```bash
# Make sure Python virtual environment is activated
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Run in development mode
cargo run

# Run release version
cargo run --release
```

## 📦 Dependencies

### Rust Dependencies

#### GUI Framework

- **eframe** (0.29): Main GUI framework
- **egui** (0.29): Immediate mode GUI library
- **egui_extras** (0.29): Additional widgets and utilities

#### Audio Processing

- **rodio** (0.19): High-level audio playback
- **symphonia** (0.5): Audio decoding for multiple formats
- **cpal** (0.15): Cross-platform audio I/O

#### Python Integration

- **pyo3** (0.22): Rust bindings for Python (for Spleeter integration)

#### Async & Utilities

- **tokio** (1.40): Async runtime for background tasks
- **serde** (1.0) + **serde_json**: Serialization/deserialization
- **anyhow** (1.0): Error handling
- **thiserror** (1.0): Custom error types
- **tracing** (0.1) + **tracing-subscriber** (0.3): Logging
- **rfd** (0.15): Native file dialogs
- **walkdir** (2.5): Directory traversal

### Python Dependencies

- **spleeter**: Vocal separation engine by Deezer

## 🎯 Planned Features

### Core Features

- [ ] Audio file playback (MP3, WAV, FLAC, OGG, etc.)
- [ ] Video file support (MP4, MKV, AVI with embedded audio)
- [ ] Real-time vocal separation using Spleeter (2stems, 4stems, 5stems)
- [ ] Synchronized lyrics display (LRC format)
- [ ] Playback controls (play, pause, stop, seek)
- [ ] Audio effects (pitch shift, tempo change, reverb)
- [ ] Volume control (master, vocals, instrumentals separately)

### Library Management

- [ ] Song library with metadata (title, artist, duration, etc.)
- [ ] Search and filter functionality
- [ ] Playlist creation and management
- [ ] Favorites system
- [ ] Import songs from folders

### User Interface

- [ ] Modern, responsive GUI
- [ ] Waveform/spectrum visualization
- [ ] Lyrics display with highlighting
- [ ] Queue management
- [ ] Settings panel

## 🔨 Development

### Project Structure

```zsh
pwe/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Main app state and UI
│   ├── audio/               # Audio engine module
│   │   ├── mod.rs
│   │   ├── player.rs        # Audio playback
│   │   └── processor.rs     # Audio processing
│   ├── spleeter/            # Spleeter integration
│   │   ├── mod.rs
│   │   └── separator.rs     # Vocal separation logic
│   ├── library/             # Song library management
│   │   ├── mod.rs
│   │   ├── database.rs      # Library database
│   │   └── scanner.rs       # Folder scanning
│   ├── lyrics/              # Lyrics handling
│   │   ├── mod.rs
│   │   └── parser.rs        # LRC parser
│   └── ui/                  # UI components
│       ├── mod.rs
│       ├── player_view.rs   # Main player UI
│       ├── library_view.rs  # Library browser
│       └── settings_view.rs # Settings panel
├── assets/                  # Application assets
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Locked dependencies
├── venv/                   # Python virtual environment (not in git)
└── README.md               # This file
```

### Environment Variables

Set `PYTHON_SYS_EXECUTABLE` if PyO3 cannot find your Python installation:

```bash
export PYTHON_SYS_EXECUTABLE=/path/to/your/python3
```

### Testing Spleeter Integration

```bash
# Activate virtual environment
source venv/bin/activate

# Test Spleeter directly
spleeter separate -i path/to/audio.mp3 -o output_folder

# The app will use the same Spleeter installation via PyO3
```

## 🐛 Troubleshooting

### PyO3 Build Issues

- Ensure Python development headers are installed (`python3-dev` or `python3-devel`)
- Set `PYTHON_SYS_EXECUTABLE` environment variable
- Make sure Python version is 3.8 or higher

### Audio Issues on Linux

- Install ALSA development libraries: `sudo apt-get install libasound2-dev`
- Check audio permissions: Add user to `audio` group

### Spleeter Not Found

- Verify Spleeter is installed: `pip list | grep spleeter`
- Make sure virtual environment is activated
- Check FFmpeg is installed: `ffmpeg -version`

### GTK Issues on Linux

- Install GTK3 development files: `sudo apt-get install libgtk-3-dev`

## 📝 License

C.F `LICENSE` file for details.

## 👥 Contributors

Hosquet Gabriel
Bregent Julien

## 🔗 Resources

- [Spleeter Documentation](https://github.com/deezer/spleeter)
- [egui Documentation](https://docs.rs/egui/)
- [rodio Documentation](https://docs.rs/rodio/)
- [PyO3 Documentation](https://pyo3.rs/)
