// Theme system for color palettes
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    IronFlower, // Dark mode
    WhiteFrame, // Light mode
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Theme::IronFlower => Theme::WhiteFrame,
            Theme::WhiteFrame => Theme::IronFlower,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Theme::IronFlower => "Iron Flower",
            Theme::WhiteFrame => "White Frame",
        }
    }

    // Primary Action colors
    pub fn primary(self) -> egui::Color32 {
        match self {
            Theme::IronFlower => egui::Color32::from_rgb(196, 30, 58), // Tekkadan Red
            Theme::WhiteFrame => egui::Color32::from_rgb(29, 78, 216), // Reactor Blue
        }
    }

    // Active/Select colors
    pub fn active(self) -> egui::Color32 {
        match self {
            Theme::IronFlower => egui::Color32::from_rgb(242, 201, 76), // Golden
            Theme::WhiteFrame => egui::Color32::from_rgb(245, 158, 11), // Orange
        }
    }

    // Text Primary colors
    pub fn text_primary(self) -> egui::Color32 {
        match self {
            Theme::IronFlower => egui::Color32::WHITE,
            Theme::WhiteFrame => egui::Color32::from_rgb(17, 24, 39), // High contrast black
        }
    }

    // Text Muted colors
    pub fn text_muted(self) -> egui::Color32 {
        match self {
            Theme::IronFlower => egui::Color32::from_rgb(139, 148, 158), // Gray
            Theme::WhiteFrame => egui::Color32::from_rgb(107, 114, 128), // Lighter gray
        }
    }

    // Apply theme to egui context
    pub fn apply(self, ctx: &egui::Context) {
        let mut visuals = match self {
            Theme::IronFlower => egui::Visuals::dark(),
            Theme::WhiteFrame => egui::Visuals::light(),
        };

        // Set background colors
        match self {
            Theme::IronFlower => {
                visuals.window_fill = egui::Color32::from_rgb(15, 17, 21); // App Background
                visuals.panel_fill = egui::Color32::from_rgb(15, 17, 21); // Panel Background
                visuals.faint_bg_color = egui::Color32::from_rgb(28, 33, 40); // Card Surface
            },
            Theme::WhiteFrame => {
                visuals.window_fill = egui::Color32::from_rgb(243, 244, 246); // App Background
                visuals.panel_fill = egui::Color32::from_rgb(243, 244, 246); // Panel Background
                visuals.faint_bg_color = egui::Color32::WHITE; // Card Surface
            },
        }

        ctx.set_visuals(visuals);
    }
}
