//! Status overlay (bottom-left)

use crate::app::{App, state::WindowMode};

impl App {
    pub fn show_status_overlay(&mut self, ctx: &egui::Context) {
        let plugin_name = self.state.gpu.as_ref()
            .and_then(|g| g.plugins.get(self.state.current_plugin_index))
            .map(|p| p.name.clone())
            .unwrap_or_default();

        let beat_intensity = self.state.gpu.as_ref().map(|g| g.beat_intensity).unwrap_or(0.0);

        egui::Window::new("##status")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .auto_sized()
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0))
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(120))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .rounding(egui::Rounding::same(4.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(
                        egui::RichText::new(&plugin_name)
                            .monospace()
                            .size(11.0)
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                    ui.label(egui::RichText::new("│").size(11.0).color(egui::Color32::DARK_GRAY));
                    let (mode_color, mode_icon) = match self.state.window_mode {
                        WindowMode::Normal      => (egui::Color32::from_rgb(150, 150, 150), "◻"),
                        WindowMode::Transparent => (egui::Color32::from_rgb(100, 180, 255), "◈"),
                        WindowMode::Overlay     => (egui::Color32::from_rgb(255, 180, 80),  "◉"),
                    };
                    ui.label(egui::RichText::new(mode_icon).size(11.0).color(mode_color));
                    ui.label(
                        egui::RichText::new(self.state.window_mode.label())
                            .size(11.0)
                            .color(mode_color),
                    );
                    ui.label(egui::RichText::new("│").size(11.0).color(egui::Color32::DARK_GRAY));
                    let segments = 8usize;
                    let filled = (beat_intensity.clamp(0.0, 1.0) * segments as f32)
                        .round() as usize;
                    let bar: String = (0..segments)
                        .map(|i| if i < filled { '█' } else { '░' })
                        .collect();
                    let beat_color = if beat_intensity > 0.7 {
                        egui::Color32::from_rgb(255, 80, 80)
                    } else if beat_intensity > 0.3 {
                        egui::Color32::from_rgb(255, 200, 60)
                    } else {
                        egui::Color32::from_rgb(80, 200, 80)
                    };
                    ui.label(egui::RichText::new(bar).monospace().size(11.0).color(beat_color));
                });
            });
    }
}
