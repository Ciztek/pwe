mod library;
mod song;

use eframe::egui;
use library::Library;
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

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("PWE Karaoke"),
        ..Default::default()
    };

    let lib = Library::try_new().and_then(|mut library| library.try_scan());
    match lib {
        Ok(library) => {
            tracing::info!(
                "Library scanned successfully with {} songs.",
                library.songs().len()
            );
        },
        Err(e) => {
            tracing::error!("Failed to initialize library: {:?}", e);
        },
    }

    eframe::run_native(
        "PWE Karaoke",
        options,
        Box::new(|cc| Ok(Box::<KaraokeApp>::new(KaraokeApp::new(cc)))),
    )
}
