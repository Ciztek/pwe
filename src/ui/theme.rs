use eframe::egui;
use enum_cycling::EnumCycle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Tekkadan, // Dark mode - "Iron Flower"
    Barbatos, // Light mode - "White Devil"
}

impl EnumCycle for Theme {
    fn up(&self) -> Self {
        match self {
            Theme::Tekkadan => Theme::Barbatos,
            Theme::Barbatos => Theme::Tekkadan,
        }
    }

    fn down(&self) -> Self {
        self.up()
    }
}

impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Theme::Tekkadan => "TEKKADAN",
            Theme::Barbatos => "BARBATOS",
        }
    }

    pub fn background(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(17, 19, 17), // Void Green
            Theme::Barbatos => egui::Color32::from_rgb(240, 242, 245), // Hangar Wall
        }
    }

    pub fn card_surface(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(34, 41, 36), // Uniform Green
            Theme::Barbatos => egui::Color32::WHITE,                // Ceramic Armor
        }
    }

    pub fn primary(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(168, 32, 40), // Flower Red
            Theme::Barbatos => egui::Color32::from_rgb(24, 69, 139), // Cobalt Blue
        }
    }

    pub fn secondary(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(58, 64, 60), // Gunmetal
            Theme::Barbatos => egui::Color32::from_rgb(229, 231, 235), // Inner Frame
        }
    }

    pub fn accent(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(212, 141, 59), // Mars Dust
            Theme::Barbatos => egui::Color32::from_rgb(235, 201, 52), // V-Fin Yellow
        }
    }

    pub fn alert(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(212, 141, 59), // Mars Dust (same as accent for dark)
            Theme::Barbatos => egui::Color32::from_rgb(201, 26, 37),  // Chin Red
        }
    }

    pub fn text_primary(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(232, 230, 227), // Bone White
            Theme::Barbatos => egui::Color32::from_rgb(31, 41, 55),    // Oil Black
        }
    }

    pub fn text_muted(self) -> egui::Color32 {
        match self {
            Theme::Tekkadan => egui::Color32::from_rgb(149, 155, 150), // Faded Canvas
            Theme::Barbatos => egui::Color32::from_rgb(107, 114, 128), // Grey
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            Theme::Tekkadan => egui::Visuals::dark(),
            Theme::Barbatos => egui::Visuals::light(),
        };

        // Set background colors
        visuals.window_fill = self.background();
        visuals.panel_fill = self.background();
        visuals.faint_bg_color = self.card_surface();
        visuals.extreme_bg_color = self.secondary();

        // Update widget colors
        visuals.widgets.noninteractive.bg_fill = self.card_surface();
        visuals.widgets.inactive.bg_fill = self.secondary();
        visuals.widgets.hovered.bg_fill = self.accent();
        visuals.widgets.active.bg_fill = self.primary();

        ctx.set_visuals(visuals);
    }
}
