// UI Widgets - reusable UI components
use eframe::egui;
use tracing::info;

pub fn render_text_section(ui: &mut egui::Ui, user_text: &mut String) {
    ui.heading("Welcome to PWE Karaoke!");
    ui.add_space(20.0);

    // Text display
    ui.horizontal(|ui| {
        ui.label("Current text:");
        ui.monospace(&*user_text);
    });

    ui.add_space(10.0);

    // Text input
    ui.horizontal(|ui| {
        ui.label("Edit text:");
        ui.text_edit_singleline(user_text);
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);
}

pub fn render_counter_section(ui: &mut egui::Ui, counter: &mut i32) {
    ui.heading("Counter Demo");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        if ui.button("➖ Decrement").clicked() {
            *counter -= 1;
            info!("Counter decremented to {}", counter);
        }

        ui.add_space(10.0);
        ui.label(format!("Value: {}", counter));
        ui.add_space(10.0);

        if ui.button("➕ Increment").clicked() {
            *counter += 1;
            info!("Counter incremented to {}", counter);
        }

        ui.add_space(10.0);

        if ui.button("🔄 Reset").clicked() {
            *counter = 0;
            info!("Counter reset");
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);
}

pub fn render_audio_section(ui: &mut egui::Ui, is_playing: bool) -> AudioAction {
    ui.heading("Audio Test");
    ui.add_space(10.0);

    let mut action = AudioAction::None;

    ui.horizontal(|ui| {
        let button_text = if is_playing {
            "🔊 Playing..."
        } else {
            "🔔 Play Beep"
        };

        if ui
            .add_enabled(!is_playing, egui::Button::new(button_text))
            .clicked()
        {
            action = AudioAction::Play;
        }

        ui.add_space(10.0);

        if ui
            .add_enabled(is_playing, egui::Button::new("⏹ Stop"))
            .clicked()
        {
            action = AudioAction::Stop;
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(20.0);

    action
}

pub fn render_info_section(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("ℹ️ About this demo:");
        ui.add_space(5.0);
        ui.label("• Edit text in the input field");
        ui.label("• Use counter buttons to increment/decrement");
        ui.label("• Play a test beep sound (440Hz A4 note)");
        ui.label("• Close the window to exit the application");
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioAction {
    None,
    Play,
    Stop,
}
