# PWE Karaoke - Architecture Documentation

## Overview

PWE Karaoke is a desktop application built with Rust, featuring a native GUI and Python integration for audio source separation. This document outlines the technical architecture and design decisions.

## Technology Stack

### Frontend

- **egui/eframe**: Immediate-mode GUI framework
  - Cross-platform (Linux, macOS, Windows)
  - GPU-accelerated rendering
  - Native look and feel

### Audio Engine

- **rodio**: High-level audio playback
- **symphonia**: Audio decoding (MP3, FLAC, WAV, OGG, etc.)
- **cpal**: Low-level cross-platform audio I/O

### Python Integration

- **PyO3**: Rust-Python bindings
- **Spleeter**: Audio source separation (vocals/instruments)

### Async Runtime

- **Tokio**: Async I/O for background tasks
  - File scanning
  - Audio processing
  - Spleeter operations

## Architecture Diagram

```txt
┌─────────────────────────────────────────────────────────────┐
│                        PWE Karaoke                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                    UI Layer (egui)                    │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │  PlayerView  │  LibraryView  │  SettingsView  │ ...  │ │
│  └───────────────────────────────────────────────────────┘ │
│                           │                                 │
│                           ▼                                 │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                Application State                      │ │
│  │  - Current track                                      │ │
│  │  - Playback state                                     │ │
│  │  - Library data                                       │ │
│  │  - Settings                                           │ │
│  └───────────────────────────────────────────────────────┘ │
│           │              │              │                   │
│  ┌────────┴────┐  ┌──────┴──────┐  ┌────┴────┐            │
│  │             │  │             │  │         │             │
│  ▼             ▼  ▼             ▼  ▼         ▼             │
│  ┌───────┐   ┌──────────┐   ┌────────┐   ┌─────────┐     │
│  │ Audio │   │ Spleeter │   │Library │   │ Lyrics  │     │
│  │Engine │   │Integration│   │Manager │   │ Parser  │     │
│  └───────┘   └──────────┘   └────────┘   └─────────┘     │
│      │             │              │            │           │
│      ▼             ▼              ▼            ▼           │
│  ┌──────────────────────────────────────────────────┐     │
│  │           Tokio Async Runtime                    │     │
│  └──────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                         │                   │
         ┌───────────────┴──────┐    ┌──────┴────────┐
         ▼                      ▼    ▼               ▼
    ┌─────────┐          ┌──────────────┐      ┌────────┐
    │ rodio   │          │    PyO3      │      │  File  │
    │symphonia│          │  (Python)    │      │ System │
    │  cpal   │          └──────────────┘      └────────┘
    └─────────┘                  │
         │                       ▼
         ▼               ┌──────────────┐
    ┌─────────┐         │   Spleeter   │
    │ Audio   │         │  (Python)    │
    │Hardware │         └──────────────┘
    └─────────┘                  │
                                 ▼
                          ┌────────────┐
                          │  TensorFlow│
                          │   FFmpeg   │
                          └────────────┘
```

## Module Structure

### `main.rs`

- Application entry point
- Initializes logging
- Sets up eframe native options
- Launches the GUI

### `app.rs`

- Main application state (`KaraokeApp`)
- UI rendering logic
- Event handling
- State management

### `audio/` module

- **`player.rs`**: Audio playback control
  - Play/pause/stop
  - Seek functionality
  - Volume control
  - Pitch/tempo shifting
- **`processor.rs`**: Audio effects and processing
  - Real-time effects
  - Audio stream management

### `spleeter/` module

- **`separator.rs`**: Spleeter integration via PyO3
  - Vocal/instrumental separation
  - Model management (2stems, 4stems, 5stems)
  - Background processing with progress tracking
  - Caching of separated tracks

### `library/` module

- **`database.rs`**: Song library persistence
  - Metadata storage (SQLite or JSON)
  - Search indexing
  - Playlist management
- **`scanner.rs`**: File system scanning
  - Recursive directory scanning
  - Metadata extraction
  - Format detection

### `lyrics/` module

- **`parser.rs`**: LRC format parser
  - Timestamp parsing
  - Lyrics synchronization
  - Multi-language support

### `ui/` module

- **`player_view.rs`**: Main player interface
  - Playback controls
  - Waveform visualization
  - Lyrics display
- **`library_view.rs`**: Library browser
  - Song list/grid view
  - Search/filter UI
  - Sorting options
- **`settings_view.rs`**: Settings panel
  - Audio device selection
  - Spleeter model configuration
  - UI preferences

## Data Flow

### Audio Playback Flow

```sh
1. User selects song from library
2. LibraryManager retrieves song metadata
3. AudioPlayer loads audio file via symphonia
4. rodio streams audio to audio device (cpal)
5. UI updates with playback position
6. LyricsParser synchronizes lyrics display
```

### Vocal Separation Flow

```sh
1. User enables vocal separation for a track
2. SpleetorSeparator checks cache for pre-separated audio
3. If not cached:
   a. Spawn async task
   b. Call Spleeter via PyO3
   c. Spleeter processes audio (Python/TensorFlow)
   d. Save separated tracks to cache
   e. Update UI with progress
4. AudioPlayer loads separated tracks
5. User can mix vocals/instrumentals independently
```

### Library Scanning Flow

```sh
1. User adds folder to library
2. LibraryScanner traverses directory tree (async)
3. For each audio file:
   a. Extract metadata (symphonia)
   b. Generate thumbnail (if video)
   c. Store in database
4. UI updates with new songs
```

## Threading Model

### Main Thread

- GUI rendering (egui)
- Event handling
- State updates

### Tokio Runtime

- File I/O operations
- Spleeter processing
- Library scanning
- Network operations (future: download lyrics, covers)

### Audio Thread

- Managed by rodio/cpal
- Real-time audio streaming
- Low-latency requirements

### Python GIL

- Spleeter calls acquire Python GIL
- Runs in background Tokio tasks
- Non-blocking for main UI

## Error Handling

### Strategy

- Use `anyhow::Result` for functions that can fail
- Use `thiserror` for custom error types
- Propagate errors with `?` operator
- Log errors with `tracing`

### Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum AudioError {
    #[error("Failed to load audio file: {0}")]
    LoadError(String),

    #[error("Playback error: {0}")]
    PlaybackError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(thiserror::Error, Debug)]
pub enum SpleeterError {
    #[error("Separation failed: {0}")]
    SeparationError(String),

    #[error("Python error: {0}")]
    PythonError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}
```

## Performance Considerations

### Audio Playback

- Use buffered I/O for smooth playback
- Target 256-512 sample buffer size for low latency
- Pre-load next track for gapless playback

### Spleeter Processing

- Process tracks in background (don't block UI)
- Cache separated audio to disk
- Limit concurrent separations (memory-intensive)
- Show progress indicators

### UI Rendering

- egui is efficient (retained mode internally)
- Avoid heavy computations in UI code
- Use async tasks for I/O
- Update UI at 60 FPS

### Memory Management

- Stream audio instead of loading entirely into memory
- Limit cache size for separated tracks
- Use memory-mapped files for large libraries

## Security Considerations

### Python Integration²

- Validate all inputs to Python
- Use virtual environment for isolation
- Don't execute arbitrary Python code

### File Handling

- Validate file paths (no directory traversal)
- Check file extensions
- Limit maximum file size

### User Data

- Store library database locally
- Encrypt sensitive settings if needed
- Respect user privacy (no telemetry by default)

## Future Extensions

### Planned Features

- Online lyrics fetching (Genius, LyricWiki APIs)
- Cloud library sync
- MIDI karaoke file support (.kar, .midi)
- Video playback with libVLC
- Plugin system for effects
- Recording functionality
- Social features (sharing, leaderboards)

### Scalability

- Multi-user support
- Server/client architecture option
- Mobile companion app
- Web interface

## Development Guidelines

### Code Style

- Follow Rust standard conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` for lints
- Write tests for critical functionality

### Commit Guidelines

- Use conventional commits
- Write clear commit messages
- Keep commits focused and atomic

### Testing Strategy

- Unit tests for business logic
- Integration tests for modules
- Manual testing for UI/UX
- Test on all target platforms

### Documentation

- Document public APIs with rustdoc
- Keep architecture docs updated
- Write examples for complex features

## References

- [egui documentation](https://docs.rs/egui/)
- [rodio documentation](https://docs.rs/rodio/)
- [PyO3 guide](https://pyo3.rs/)
- [Spleeter repository](https://github.com/deezer/spleeter)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
