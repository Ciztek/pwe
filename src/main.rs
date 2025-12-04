mod app;
mod audio;
mod library;
mod ui;

use app::KaraokeApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("PWE Karaoke"),
        ..Default::default()
    };

    eframe::run_native(
        "PWE Karaoke",
        options,
        Box::new(|cc| Ok(Box::<KaraokeApp>::new(KaraokeApp::new(cc)))),
    )
}
