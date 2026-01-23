mod library;
mod song;

use eframe::egui;
use std::time::Instant;

struct KaraokeApp {
    #[cfg(debug_assertions)]
    last_frame_time: Instant,
    #[cfg(debug_assertions)]
    frame_times: Vec<f32>,
}

impl KaraokeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            #[cfg(debug_assertions)]
            last_frame_time: Instant::now(),
            #[cfg(debug_assertions)]
            frame_times: Vec::with_capacity(60),
        }
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, _ctx: &egui::Context, _: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        {
            let now = Instant::now();
            let delta = now.duration_since(self.last_frame_time).as_secs_f32();
            let fps = 1.0 / delta;

            self.frame_times.push(fps);
            if self.frame_times.len() > 60 {
                self.frame_times.remove(0);
            }

            let avg_fps = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.last_frame_time = now;

            // Display FPS counter in top-left corner
            egui::Window::new("Debug Info")
                .resizable(false)
                .collapsible(false)
                .default_pos(egui::pos2(10.0, 10.0))
                .show(_ctx, |ui| {
                    ui.label(format!("FPS: {:.1}", avg_fps));
                    ui.label(format!("Frame Time: {:.2}ms", delta * 1000.0));
                });
        }

        egui::CentralPanel::default().show(_ctx, |ui| {
            ui.label("Welcome to PWE Karaoke v2!");
        });
    }
}

fn test_metadata() {
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::Path;
    while (true) {
        print!("Enter a file or directory path: ");
        io::stdout().flush().unwrap(); // Ensure prompt is printed before input

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            return;
        }

        // Trim whitespace and newline characters
        let trimmed = input.trim();

        // Validate empty input
        if trimmed.is_empty() {
            eprintln!("No path provided.");
            return;
        }

        let path = Path::new(trimmed);

        // Check if the path exists
        if !path.exists() || !path.is_file() {
            println!("Path does not exist or is not a file: {}", trimmed);
            return;
        }
        match song::metadata::extract_metadata(path) {
            Ok(metadata) => {
                println!("Extracted Metadata: {:#?}", metadata);
            },
            Err(e) => {
                eprintln!("Error extracting metadata: {}", e);
            },
        }
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("PWE Karaoke"),
        ..Default::default()
    };

    test_metadata();

    eframe::run_native(
        "PWE Karaoke",
        options,
        Box::new(|cc| Ok(Box::<KaraokeApp>::new(KaraokeApp::new(cc)))),
    )
}
