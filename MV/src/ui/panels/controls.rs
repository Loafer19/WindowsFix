//! Controls and help panel (F1)

use crate::app::App;
use crate::input::SHORTCUTS;

impl App {
    pub fn show_controls_panel(&mut self, ctx: &egui::Context) {
        let plugin_name = self.state.gpu.as_ref()
            .and_then(|g| g.plugins.get(self.state.current_plugin_index))
            .map(|p| p.name.clone())
            .unwrap_or_default();

        let device_name = self.state.settings.selected_device.as_deref().unwrap_or("None");

        egui::Window::new("ℹ Controls")
            .open(&mut self.state.show_info)
            .resizable(false)
            .collapsible(false)
            .default_width(340.0)
            .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new(&plugin_name).strong());
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎤").size(14.0));
                    ui.label(device_name);
                    ui.separator();
                    ui.label(egui::RichText::new("🪟").size(14.0));
                    ui.label(self.state.window_mode.label());
                    ui.separator();
                    ui.label(egui::RichText::new("🥁").size(14.0));
                    ui.label(self.state.settings.beat_sensitivity.label());
                });
                ui.separator();

                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        let mut last_category = "";
                        for shortcut in SHORTCUTS {
                            if shortcut.category != last_category {
                                ui.label(egui::RichText::new(format!("── {} ──", shortcut.category))
                                    .strong().color(egui::Color32::from_rgb(180, 180, 90)));
                                ui.end_row();
                                last_category = shortcut.category;
                            }
                            ui.label(egui::RichText::new(shortcut.key_label).monospace().strong());
                            ui.label(shortcut.description);
                            ui.end_row();
                        }
                    });
            });
    }
}
