use crate::app::AppView;
use crate::ui::theme::Theme;
use eframe::egui;

pub fn render_top_panel(
    ctx: &egui::Context,
    theme: Theme,
    current_view: AppView,
    fps: f32,
) -> (bool, Option<AppView>) {
    theme.apply(ctx);

    let mut theme_switched = false;
    let mut view_change = None;

    egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame::none().fill(theme.card_surface()))
        .min_height(50.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(32.0);
                render_logo(ui, theme);
                ui.add_space(32.0);

                if let Some(view) = render_navigation(ui, theme, current_view) {
                    view_change = Some(view);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    theme_switched = render_theme_switcher(ui, theme);
                    if cfg!(debug_assertions) {
                        render_fps_counter(ui, theme, fps);
                    }
                });
            });
            ui.add_space(8.0);
        });

    (theme_switched, view_change)
}

fn render_logo(ui: &mut egui::Ui, theme: Theme) {
    ui.label(
        egui::RichText::new("IRON-VOX")
            .size(20.0)
            .color(theme.primary())
            .strong(),
    );
}

fn render_navigation(ui: &mut egui::Ui, theme: Theme, current_view: AppView) -> Option<AppView> {
    let mut view_change = None;

    let library_color = if current_view == AppView::Library {
        theme.primary()
    } else {
        theme.text_muted()
    };
    if ui
        .add(egui::Button::new(
            egui::RichText::new("[ Library ]").color(library_color),
        ))
        .clicked()
    {
        view_change = Some(AppView::Library);
    }

    ui.add_space(8.0);

    let karaoke_color = if current_view == AppView::Karaoke {
        theme.primary()
    } else {
        theme.text_muted()
    };
    if ui
        .add(egui::Button::new(
            egui::RichText::new("[ Karaoke ]").color(karaoke_color),
        ))
        .clicked()
    {
        view_change = Some(AppView::Karaoke);
    }

    ui.add_space(8.0);

    let settings_color = if current_view == AppView::Settings {
        theme.primary()
    } else {
        theme.text_muted()
    };
    if ui
        .add(egui::Button::new(
            egui::RichText::new("[ Settings ]").color(settings_color),
        ))
        .clicked()
    {
        view_change = Some(AppView::Settings);
    }

    view_change
}

fn render_theme_switcher(ui: &mut egui::Ui, theme: Theme) -> bool {
    let theme_text = format!("[ {} ]", theme.name());
    ui.add(egui::Button::new(
        egui::RichText::new(theme_text)
            .color(theme.accent())
            .small(),
    ))
    .clicked()
}

fn render_fps_counter(ui: &mut egui::Ui, theme: Theme, fps: f32) {
    if fps > 0.0 {
        ui.label(
            egui::RichText::new(format!("{fps:.0} FPS"))
                .color(theme.text_muted())
                .size(12.0)
                .monospace(),
        );
        ui.add_space(12.0);
    }
}
