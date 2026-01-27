use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver};
use notify::{
    event::{EventKind, ModifyKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use tracing::{error, info, warn};

use crate::song::Song;

use std::sync::{Arc, Mutex};

type RepaintHook = Arc<Mutex<Option<egui::Context>>>;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac"];

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Debug)]
enum LibraryEvent {
    Add(PathBuf),
    Remove(PathBuf),
    Modify(PathBuf),
}

#[derive(Debug)]
pub struct Library {
    songs: Vec<Song>,
    _path: PathBuf,

    _rx: Receiver<LibraryEvent>,
    _watcher: RecommendedWatcher,
    _repaint_hook: RepaintHook,
}

impl Library {
    pub fn try_new() -> Result<Self> {
        let library_dir = Self::get_library_directory()?;
        let _repaint_hook = Arc::new(Mutex::new(None));

        let (rx, watcher) = Self::start_watcher(library_dir.clone(), _repaint_hook.clone())
            .context("failed to start watcher")?;

        Ok(Self {
            songs: Vec::new(),
            _path: library_dir,
            _rx: rx,
            _watcher: watcher,
            _repaint_hook,
        })
    }

    pub fn set_egui_ctx(&self, ctx: egui::Context) {
        match self._repaint_hook.lock() {
            Ok(mut slot) => {
                *slot = Some(ctx);
            },
            Err(poisoned) => {
                *poisoned.into_inner() = Some(ctx);
            },
        }
    }

    pub fn try_scan(&mut self) -> Result<()> {
        info!("Scanning library folder");

        let mut songs = Vec::new();

        for entry in walkdir::WalkDir::new(&self._path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            if !path.is_file() || !is_audio_file(path) {
                continue;
            }

            match Song::from_path(path.to_path_buf()) {
                Some(song) => songs.push(song),
                None => error!("Failed to parse song from: {}", path.display()),
            }
        }

        self.songs = songs;
        Ok(())
    }

    pub fn poll(&mut self) -> bool {
        let mut dirty = false;
        while let Ok(event) = self._rx.try_recv() {
            self.handle_event(event);
            dirty = true;
        }
        dirty
    }

    pub fn songs(&self) -> &[Song] {
        &self.songs
    }

    fn start_watcher(
        path: PathBuf,
        repaint: RepaintHook,
    ) -> Result<(Receiver<LibraryEvent>, RecommendedWatcher)> {
        let (tx, rx) = unbounded();

        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res| {
                let event: notify::Event = match res {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("watch error: {:?}", e);
                        return;
                    },
                };

                for path in event.paths {
                    let evt = match &event.kind {
                        EventKind::Create(_) => Some(LibraryEvent::Add(path)),
                        EventKind::Remove(_) => Some(LibraryEvent::Remove(path)),
                        EventKind::Modify(ModifyKind::Data(_))
                        | EventKind::Modify(ModifyKind::Name(_)) => {
                            Some(LibraryEvent::Modify(path))
                        },
                        _ => None,
                    };

                    if let Some(evt) = evt {
                        let _ = tx.send(evt);
                        // If lock fails: silently skip repaint
                        if let Ok(lock) = repaint.lock() {
                            if let Some(ctx) = lock.as_ref() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_secs(1))
                .with_compare_contents(true),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        info!("Watching library directory");

        Ok((rx, watcher))
    }

    fn handle_event(&mut self, event: LibraryEvent) {
        match event {
            LibraryEvent::Add(path) => self.add_song_from_path(path),
            LibraryEvent::Remove(path) => self.remove_song_by_path(&path),
            LibraryEvent::Modify(path) => self.update_song_from_path(path),
        }
    }

    fn add_song_from_path(&mut self, path: PathBuf) {
        if !is_audio_file(&path) {
            return;
        }

        if self.songs.iter().any(|s| s.path() == path) {
            return;
        }

        if let Some(song) = Song::from_path(path.clone()) {
            info!("Added song: {}", path.display());
            self.songs.push(song);
        }
    }

    fn remove_song_by_path(&mut self, path: &Path) {
        let before = self.songs.len();
        self.songs.retain(|s| s.path() != path);

        if self.songs.len() != before {
            info!("Removed song: {}", path.display());
        }
    }

    fn update_song_from_path(&mut self, path: PathBuf) {
        self.remove_song_by_path(&path);
        self.add_song_from_path(path);
    }

    // ========================
    // Directory resolution
    // ========================

    fn get_library_directory() -> Result<PathBuf> {
        #[cfg(not(debug_assertions))]
        {
            let app_data = if cfg!(target_os = "windows") {
                std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .context("Failed to get APPDATA")?
                    .join("PWE Karaoke")
            } else {
                dirs::data_dir()
                    .context("Failed to get data directory")?
                    .join("pwe-karaoke")
            };

            let dir = app_data.join("Library");
            std::fs::create_dir_all(&dir)?;
            Ok(dir)
        }

        #[cfg(debug_assertions)]
        {
            let dir = PathBuf::from("dev_library");
            std::fs::create_dir_all(&dir)?;
            Ok(dir)
        }
    }
}
