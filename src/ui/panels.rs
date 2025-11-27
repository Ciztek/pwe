// UI Panels - top, bottom, and central panel rendering
use eframe::egui;

pub fn render_top_panel(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🎤 PWE Karaoke");
            ui.separator();
            ui.label("Desktop Karaoke Application");
        });
    });
}

pub fn render_bottom_panel(ctx: &egui::Context, is_playing: bool, counter: i32) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Status:");
            if is_playing {
                ui.colored_label(egui::Color32::GREEN, "🔊 Playing");
            } else {
                ui.colored_label(egui::Color32::GRAY, "⏸ Idle");
            }
            ui.separator();
            ui.label(format!("Counter: {}", counter));
        });
    });
}
