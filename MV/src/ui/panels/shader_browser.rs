//! Shader browser panel (F4)

use crate::app::App;
use crate::visualization::ShaderInfo;

impl App {
    pub fn show_shader_browser(&mut self, ctx: &egui::Context) {
        // Collect shader browser entries before closure
        let shader_browser_entries: Vec<(usize, String, Option<&'static ShaderInfo>)> = self.state.gpu.as_ref()
            .map(|g| g.plugins.iter().enumerate()
                .map(|(i, p)| (i, p.name.clone(), p.info))
                .collect())
            .unwrap_or_default();

        let current_plugin_idx = self.state.current_plugin_index;
        let mut show_shader_browser = self.state.show_shader_browser;
        let mut new_plugin_index: Option<usize> = None;

        egui::Window::new("🎭 Shaders")
            .open(&mut show_shader_browser)
            .resizable(true)
            .collapsible(false)
            .default_width(400.0)
            .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut grouped: std::collections::HashMap<&'static str, Vec<(usize, String, Option<&'static ShaderInfo>)>> = std::collections::HashMap::new();
                    for (idx, name, info) in &shader_browser_entries {
                        let cat: &'static str = if let Some(si) = info { si.category.label() } else { "✨ Abstract" };
                        grouped.entry(cat).or_default().push((*idx, name.clone(), *info));
                    }

                    const CAT_ORDER: &[&str] = &["🎵 Spectrum", "🌡 Heatmap", "🌊 Waveform", "🔮 3D Effects", "✨ Abstract"];
                    for cat in CAT_ORDER {
                        if let Some(entries) = grouped.get(*cat) {
                            ui.collapsing(*cat, |ui| {
                                for (idx, name, info) in entries {
                                    let is_active = *idx == current_plugin_idx;
                                    ui.horizontal(|ui| {
                                        let label = if is_active {
                                            egui::RichText::new(name.as_str()).strong().color(egui::Color32::from_rgb(100, 200, 100))
                                        } else {
                                            egui::RichText::new(name.as_str())
                                        };
                                        if ui.selectable_label(is_active, label).clicked() {
                                            new_plugin_index = Some(*idx);
                                        }
                                        if let Some(info) = info {
                                            ui.label(egui::RichText::new(info.performance.label()).small().color(egui::Color32::GRAY));
                                        }
                                    });
                                    if let Some(info) = info {
                                        ui.label(egui::RichText::new(info.description).small().italics().color(egui::Color32::GRAY));
                                    }
                                }
                            });
                        }
                    }
                });
            });

        self.state.show_shader_browser = show_shader_browser;
        if let Some(idx) = new_plugin_index {
            self.state.current_plugin_index = idx;
            self.state.transition_active = true;
            self.state.transition_time = 0.0;
            self.state.last_mode_switch = std::time::Instant::now();
        }
    }
}
