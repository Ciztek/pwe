use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Downloader {
    yt_dlp_path: String,
    output_dir: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub title: String,
    pub progress: f32,
    pub status: DownloadStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Converting,
    Completed,
    Failed(String),
}

impl Downloader {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            yt_dlp_path: Self::find_yt_dlp(),
            output_dir,
        }
    }

    fn find_yt_dlp() -> String {
        // Try common locations
        let candidates = vec![
            "yt-dlp",
            "yt-dlp.exe",
            "python3 -m yt_dlp",
            "python -m yt_dlp",
        ];

        for candidate in candidates {
            if Self::check_command(candidate) {
                info!("Found yt-dlp: {}", candidate);
                return candidate.to_string();
            }
        }

        warn!("yt-dlp not found in PATH");
        "yt-dlp".to_string()
    }

    fn check_command(cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        Command::new(parts[0])
            .args(&parts[1..])
            .arg("--version")
            .output()
            .is_ok()
    }

    pub fn is_available(&self) -> bool {
        Self::check_command(&self.yt_dlp_path)
    }

    // Used in spawned thread - compiler can't detect through closure boundary
    #[allow(dead_code)]
    pub async fn download_youtube_video(&self, video_id: &str) -> Result<PathBuf, String> {
        if !self.is_available() {
            return Err(
                "yt-dlp is not installed. Please install it to download videos.".to_string(),
            );
        }

        let url = format!("https://www.youtube.com/watch?v={}", video_id);

        info!("Downloading YouTube video: {}", video_id);

        let output_template = self.output_dir.join("%(title)s.%(ext)s");
        let output_template_str = output_template.to_string_lossy();

        let parts: Vec<&str> = self.yt_dlp_path.split_whitespace().collect();
        let mut cmd = if parts.len() > 1 {
            let mut c = Command::new(parts[0]);
            c.args(&parts[1..]);
            c
        } else {
            Command::new(parts[0])
        };

        let output = cmd
            .arg("--extract-audio")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("0")
            .arg("--embed-metadata")  // Embed metadata in the audio file
            .arg("--embed-thumbnail")  // Embed album art
            .arg("--convert-thumbnails")
            .arg("jpg")  // Convert thumbnails to jpg for compatibility
            .arg("--parse-metadata")
            .arg("title:%(title)s")  // Parse title
            .arg("--parse-metadata")
            .arg("artist:%(artist)s,uploader:%(uploader)s")  // Parse artist
            .arg("--write-subs")  // Download subtitles if available (may have lyrics)
            .arg("--sub-langs")
            .arg("en.*,ja.*,fr.*,es.*")  // Common languages
            .arg("--embed-subs")  // Embed subtitles
            .arg("--output")
            .arg(output_template_str.as_ref())
            .arg("--no-playlist")
            .arg("--print")
            .arg("after_move:filepath")
            .arg(&url)
            .output()
            .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("yt-dlp failed: {}", error_msg);
            return Err(format!("Download failed: {}", error_msg));
        }

        let output_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if output_path.is_empty() {
            return Err("Failed to get output path".to_string());
        }

        let path = PathBuf::from(output_path);
        info!("Downloaded to: {}", path.display());
        Ok(path)
    }

    // Reserved for Spotify integration
    #[allow(dead_code)]
    pub async fn download_spotify_track(
        &self,
        track_name: &str,
        artist: &str,
    ) -> Result<PathBuf, String> {
        // Spotify tracks need to be searched on YouTube
        // We'll search for "track_name artist" and download the first result

        if !self.is_available() {
            return Err(
                "yt-dlp is not installed. Please install it to download tracks.".to_string(),
            );
        }

        let search_query = format!("{} {}", track_name, artist);
        let search_url = format!("ytsearch1:{}", search_query);

        info!("Searching and downloading: {}", search_query);

        let output_template = self.output_dir.join("%(title)s.%(ext)s");
        let output_template_str = output_template.to_string_lossy();

        let parts: Vec<&str> = self.yt_dlp_path.split_whitespace().collect();
        let mut cmd = if parts.len() > 1 {
            let mut c = Command::new(parts[0]);
            c.args(&parts[1..]);
            c
        } else {
            Command::new(parts[0])
        };

        let output = cmd
            .arg("--extract-audio")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("0")
            .arg("--embed-metadata")  // Embed metadata
            .arg("--embed-thumbnail")  // Embed album art
            .arg("--convert-thumbnails")
            .arg("jpg")
            .arg("--parse-metadata")
            .arg("title:%(title)s")
            .arg("--parse-metadata")
            .arg("artist:%(artist)s,uploader:%(uploader)s")
            .arg("--write-subs")
            .arg("--sub-langs")
            .arg("en.*,ja.*,fr.*,es.*")
            .arg("--embed-subs")
            .arg("--output")
            .arg(output_template_str.as_ref())
            .arg("--print")
            .arg("after_move:filepath")
            .arg(&search_url)
            .output()
            .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("yt-dlp failed: {}", error_msg);
            return Err(format!("Download failed: {}", error_msg));
        }

        let output_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if output_path.is_empty() {
            return Err("Failed to get output path".to_string());
        }

        let path = PathBuf::from(output_path);
        info!("Downloaded to: {}", path.display());
        Ok(path)
    }

    #[allow(dead_code)]
    pub fn set_output_dir(&mut self, dir: PathBuf) {
        self.output_dir = dir;
    }

    /// Get list of video IDs and titles from a YouTube playlist
    pub async fn get_playlist_videos(
        &self,
        playlist_url: &str,
    ) -> Result<Vec<(String, String)>, String> {
        if !self.is_available() {
            return Err("yt-dlp is not installed".to_string());
        }

        info!("📋 Fetching playlist info from: {}", playlist_url);

        let parts: Vec<&str> = self.yt_dlp_path.split_whitespace().collect();
        let mut cmd = if parts.len() > 1 {
            let mut c = Command::new(parts[0]);
            c.args(&parts[1..]);
            c
        } else {
            Command::new(parts[0])
        };

        let output = cmd
            .arg("--flat-playlist")
            .arg("--print")
            .arg("%(id)s|||%(title)s")
            .arg(playlist_url)
            .output()
            .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("❌ yt-dlp failed: {}", error_msg);
            return Err(format!("Failed to fetch playlist: {}", error_msg));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let videos: Vec<(String, String)> = output_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split("|||").collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        info!("✅ Found {} videos in playlist", videos.len());
        Ok(videos)
    }

    /// Get list of tracks from a Spotify playlist URL
    /// Note: Requires yt-dlp with Spotify extractor support
    pub async fn get_spotify_playlist_tracks(
        &self,
        playlist_url: &str,
    ) -> Result<Vec<(String, String)>, String> {
        if !self.is_available() {
            return Err("yt-dlp is not installed".to_string());
        }

        info!("📋 Fetching Spotify playlist info from: {}", playlist_url);

        let parts: Vec<&str> = self.yt_dlp_path.split_whitespace().collect();
        let mut cmd = if parts.len() > 1 {
            let mut c = Command::new(parts[0]);
            c.args(&parts[1..]);
            c
        } else {
            Command::new(parts[0])
        };

        let output = cmd
            .arg("--flat-playlist")
            .arg("--print")
            .arg("%(title)s|||%(artist)s,%(uploader)s")
            .arg(playlist_url)
            .output()
            .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("❌ yt-dlp failed: {}", error_msg);

            // Check if it's a Spotify-specific error
            if error_msg.contains("Spotify") || error_msg.contains("spotify") {
                return Err(
                    "Spotify extraction failed. Note: yt-dlp's Spotify support is limited. \
                    Consider using 'spotdl' for better Spotify support (pip install spotdl)."
                        .to_string(),
                );
            }

            return Err(format!("Failed to fetch playlist: {}", error_msg));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let tracks: Vec<(String, String)> = output_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split("|||").collect();
                if parts.len() == 2 {
                    let title = parts[0].trim().to_string();
                    let artist = parts[1]
                        .split(',')
                        .next()
                        .unwrap_or("Unknown")
                        .trim()
                        .to_string();
                    Some((title, artist))
                } else {
                    None
                }
            })
            .collect();

        if tracks.is_empty() {
            return Err("No tracks found. yt-dlp may not support this Spotify URL. \
                Install 'spotdl' for better Spotify support: pip install spotdl"
                .to_string());
        }

        info!("✅ Found {} tracks in Spotify playlist", tracks.len());
        Ok(tracks)
    }
}
