// Network operations and download state management

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use tracing::{error, info};

use crate::network::downloader::Downloader;

#[derive(Debug, Clone, Default)]
pub struct DownloadState {
    pub is_downloading: bool,
    pub current_index: usize,
    pub total_count: usize,
    pub current_song: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptionState {
    pub is_transcribing: bool,
    pub song_name: String,
    pub song_path: Option<PathBuf>,
}

pub struct NetworkState {
    pub downloader: Downloader,
    pub download_tx: Option<Sender<DownloadMessage>>,
    pub download_rx: Option<Receiver<DownloadMessage>>,
}

impl NetworkState {
    pub fn new(download_path: PathBuf) -> Self {
        Self {
            downloader: Downloader::new(download_path),
            download_tx: None,
            download_rx: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    Started {
        total: usize,
    },
    Progress {
        index: usize,
        song: String,
        status: String,
    },
    Completed,
    Error(String),
}

impl NetworkState {
    /// Extract `YouTube` playlist ID from URL
    pub fn extract_youtube_playlist_id(url: &str) -> Option<String> {
        url.find("list=").map(|start| {
            let id_start = start + 5;
            let id_end = url[id_start..]
                .find('&')
                .map_or(url.len(), |pos| id_start + pos);
            url[id_start..id_end].to_string()
        })
    }

    /// Start `YouTube` playlist download
    pub fn start_youtube_playlist_download(&mut self, playlist_id: &str) {
        info!("🚀 Starting download for playlist: {}", playlist_id);

        let (tx, rx) = channel();
        self.download_tx = Some(tx.clone());
        self.download_rx = Some(rx);

        let url = format!("https://www.youtube.com/playlist?list={playlist_id}");
        let downloader = self.downloader.clone();

        std::thread::spawn(move || {
            #[allow(clippy::expect_used)]
            let runtime = tokio::runtime::Runtime::new()
                .expect("Failed to create Tokio runtime for YouTube playlist download");
            runtime.block_on(async move {
                match downloader.get_playlist_videos(&url) {
                    Ok(videos) => {
                        let total = videos.len();
                        let _ = tx.send(DownloadMessage::Started { total });

                        for (idx, (video_id, title)) in videos.iter().enumerate() {
                            let _ = tx.send(DownloadMessage::Progress {
                                index: idx + 1,
                                song: title.clone(),
                                status: "Downloading...".to_string(),
                            });

                            match downloader.download_youtube_video(video_id) {
                                Ok(path) => {
                                    info!("✅ Downloaded: {}", path.display());
                                },
                                Err(e) => {
                                    error!("❌ Failed to download {}: {}", title, e);
                                },
                            }
                        }

                        let _ = tx.send(DownloadMessage::Completed);
                    },
                    Err(e) => {
                        error!("❌ Failed to fetch playlist: {}", e);
                        let _ = tx.send(DownloadMessage::Error(e));
                    },
                }
            });
        });
    }

    /// Start Spotify playlist download
    pub fn start_spotify_playlist_download(&mut self, playlist_url: String) {
        info!("🚀 Starting Spotify download for: {}", playlist_url);

        let (tx, rx) = channel();
        self.download_tx = Some(tx.clone());
        self.download_rx = Some(rx);

        let downloader = self.downloader.clone();

        std::thread::spawn(move || {
            #[allow(clippy::expect_used)]
            let runtime = tokio::runtime::Runtime::new()
                .expect("Failed to create Tokio runtime for Spotify playlist download");
            runtime.block_on(async move {
                match downloader.get_spotify_playlist_tracks(&playlist_url) {
                    Ok(tracks) => {
                        let total = tracks.len();
                        let _ = tx.send(DownloadMessage::Started { total });

                        for (idx, (title, artist)) in tracks.iter().enumerate() {
                            let track_info = format!("{title} - {artist}");
                            let _ = tx.send(DownloadMessage::Progress {
                                index: idx + 1,
                                song: track_info.clone(),
                                status: "Searching on YouTube...".to_string(),
                            });

                            match downloader.download_spotify_track(title, artist) {
                                Ok(path) => {
                                    info!("✅ Downloaded: {}", path.display());
                                },
                                Err(e) => {
                                    error!("❌ Failed to download {}: {}", track_info, e);
                                },
                            }
                        }

                        let _ = tx.send(DownloadMessage::Completed);
                    },
                    Err(e) => {
                        error!("❌ Failed to fetch Spotify playlist: {}", e);
                        let _ = tx.send(DownloadMessage::Error(e));
                    },
                }
            });
        });
    }
}
