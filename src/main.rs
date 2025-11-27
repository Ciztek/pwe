mod app;
mod audio;
mod ui;

use app::KaraokeApp;

fn main() -> eframe::Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Configure window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("PWE Karaoke"),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "PWE Karaoke",
        options,
        Box::new(|cc| Ok(Box::<KaraokeApp>::new(KaraokeApp::new(cc)))),
    )
}
