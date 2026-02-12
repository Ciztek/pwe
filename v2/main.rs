#[cfg(feature = "repl")]
mod repl;

mod library;
mod song;

use eframe::egui;
use library::Library;
use std::time::Instant;
use tracing::{error, info};

struct KaraokeApp {
    library: Library,

    #[cfg(debug_assertions)]
    last_frame_time: Instant,
    #[cfg(debug_assertions)]
    frame_times: Vec<f32>,
}

impl KaraokeApp {
    fn new(cc: &eframe::CreationContext<'_>, library: Library) -> Self {
        library.set_egui_ctx(cc.egui_ctx.clone());

        info!("Library initialized with {} songs", library.songs().len());

        Self {
            library,

            #[cfg(debug_assertions)]
            last_frame_time: Instant::now(),
            #[cfg(debug_assertions)]
            frame_times: Vec::with_capacity(60),
        }
    }
}

impl eframe::App for KaraokeApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.library.poll() {
            // library is dirty but in theory all changes already auto-save
        }

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

            egui::Window::new("Debug Info")
                .resizable(false)
                .collapsible(false)
                .default_pos(egui::pos2(10.0, 10.0))
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {:.1}", avg_fps));
                    ui.label(format!("Frame Time: {:.2}ms", delta * 1000.0));
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to PWE Karaoke v2!");
            ui.separator();
            ui.label(format!("Songs in library: {}", self.library.songs().len()));
        });
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let mut library = match Library::try_new() {
        Ok(lib) => lib,
        Err(e) => {
            error!("Failed to initialize library: {:?}", e);
            return Err(eframe::Error::AppCreation(e.to_string().into()));
        },
    };

    #[cfg(feature = "repl")]
    {
        repl::run_repl(&mut library);
        return Ok(());
    }

    #[cfg(not(feature = "repl"))]
    {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "PWE Karaoke",
            options,
            Box::new(|cc| Ok(Box::new(KaraokeApp::new(cc, library)))),
        )
    }
}
