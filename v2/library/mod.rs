use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver};
use notify::{event::ModifyKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::song::Song;

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

impl Serialize for Library {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Library", 1)?;
        state.serialize_field("entries", &self.songs)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Library {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Data {
            entries: Vec<Song>,
        }

        let data = Data::deserialize(deserializer)?;

        let dir = Library::get_library_directory().map_err(de::Error::custom)?;
        let repaint = Arc::new(Mutex::new(None));
        let (rx, watcher) =
            Library::start_watcher(dir.clone(), repaint.clone()).map_err(de::Error::custom)?;

        let songs = data
            .entries
            .into_iter()
            .filter(|s| s.path().exists())
            .collect();

        Ok(Library {
            songs,
            _path: dir,
            _rx: rx,
            _watcher: watcher,
            _repaint_hook: repaint,
        })
    }
}

impl Library {
    pub fn try_new() -> Result<Self> {
        match Self::load() {
            Ok(lib) => Ok(lib),
            Err(_) => {
                let mut lib = Self::empty_runtime()?;
                lib.try_scan()?;
                lib.save()?;
                Ok(lib)
            },
        }
    }

    fn empty_runtime() -> Result<Self> {
        let dir = Self::get_library_directory()?;
        let repaint = Arc::new(Mutex::new(None));
        let (rx, watcher) = Self::start_watcher(dir.clone(), repaint.clone())?;

        Ok(Self {
            songs: Vec::new(),
            _path: dir,
            _rx: rx,
            _watcher: watcher,
            _repaint_hook: repaint,
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = self._path.join("library.json");
        let file = std::fs::File::create(&path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn load() -> Result<Self> {
        let dir = Self::get_library_directory()?;
        let path = dir.join("library.json");

        let file = std::fs::File::open(path)?;
        let lib: Library = serde_json::from_reader(file)?;
        Ok(lib)
    }

    pub fn songs(&self) -> &[Song] {
        &self.songs
    }

    pub fn set_egui_ctx(&self, ctx: egui::Context) {
        if let Ok(mut slot) = self._repaint_hook.lock() {
            *slot = Some(ctx);
        }
    }

    pub fn poll(&mut self) -> bool {
        let mut dirty = false;
        while let Ok(event) = self._rx.try_recv() {
            self.handle_event(event);
            dirty = true;
        }
        dirty
    }

    fn handle_event(&mut self, event: LibraryEvent) {
        match event {
            LibraryEvent::Add(path) => self.add(path),
            LibraryEvent::Remove(path) => self.remove(&path),
            LibraryEvent::Modify(path) => {
                self.remove(&path);
                self.add(path);
            },
        }

        if let Err(e) = self.save() {
            error!("Failed to save library: {:?}", e);
        }
    }

    fn add(&mut self, path: PathBuf) {
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
    fn remove(&mut self, path: &Path) {
        let before = self.songs.len();
        self.songs.retain(|s| s.path() != path);

        if self.songs.len() != before {
            info!("Removed song: {}", path.display());
        }
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

    fn get_library_directory() -> Result<PathBuf> {
        #[cfg(debug_assertions)]
        {
            let dir = PathBuf::from("dev_library_v2");
            std::fs::create_dir_all(&dir)?;
            Ok(dir)
        }

        #[cfg(not(debug_assertions))]
        {
            let dir = dirs::data_dir()
                .context("No data dir")?
                .join("pwe-karaoke")
                .join("Library");

            std::fs::create_dir_all(&dir)?;
            Ok(dir)
        }
    }

    fn try_scan(&mut self) -> Result<()> {
        for entry in walkdir::WalkDir::new(&self._path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_file() && is_audio_file(path) {
                if let Some(song) = Song::from_path(path.to_path_buf()) {
                    self.songs.push(song);
                }
            }
        }
        Ok(())
    }
}
