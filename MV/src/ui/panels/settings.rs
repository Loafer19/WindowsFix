//! Settings panel (F2)

use crate::app::{App, build_plugin_groups};
use crate::config::settings::{BeatSensitivity, ColorScheme};

impl App {
    pub fn show_settings_panel(&self, ctx: &egui::Context, settings_copy: &mut crate::config::settings::AppSettings) {
        let plugin_groups = {
            let names: Vec<String> = self.state.gpu.as_ref()
                .map(|g| g.plugins.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            build_plugin_groups(&names)
        };

        egui::Window::new("⚙ Settings")
            .open(&mut settings_copy.show_settings)
            .resizable(true)
            .collapsible(false)
            .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
            .show(ctx, |ui| {
                ui.collapsing("🎤 Audio Processing", |ui| {
                    ui.add(egui::Slider::new(&mut settings_copy.smoothing_factor, 0.01..=0.3).text("Smoothing"));
                    ui.add(egui::Slider::new(&mut settings_copy.gain, 0.5..=5.0).text("Gain"));
                    ui.add(egui::Slider::new(&mut settings_copy.bass_boost, 0.0..=2.0).text("Bass Boost"));
                    ui.horizontal(|ui| {
                        ui.label("Beat Sensitivity:");
                        ui.selectable_value(&mut settings_copy.beat_sensitivity, BeatSensitivity::Low,    "Low");
                        ui.selectable_value(&mut settings_copy.beat_sensitivity, BeatSensitivity::Medium, "Medium");
                        ui.selectable_value(&mut settings_copy.beat_sensitivity, BeatSensitivity::High,   "High");
                    });
                });

                ui.collapsing("�� Visual", |ui| {
                    ui.add(egui::Slider::new(&mut settings_copy.transparency, 0.1..=1.0).text("Opacity"));
                    ui.horizontal(|ui| {
                        ui.label("Color Scheme:");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Classic, "Classic");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Neon,    "Neon");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Pastel,  "Pastel");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Fire,    "Fire");
                    });
                });

                ui.collapsing("▶ Playback", |ui| {
                    ui.checkbox(&mut settings_copy.auto_switch_modes, "Auto-switch modes");
                    if settings_copy.auto_switch_modes {
                        ui.add(egui::Slider::new(&mut settings_copy.mode_switch_seconds, 5.0..=120.0).text("Switch interval (s)"));
                    }
                });

                ui.collapsing("✨ Effects", |ui| {
                    for (group_name, names) in &plugin_groups {
                        ui.collapsing(group_name.as_str(), |ui| {
                            for name in names {
                                let mut enabled = !settings_copy.disabled_plugins.contains(name);
                                if ui.checkbox(&mut enabled, name.as_str()).changed() {
                                    if enabled {
                                        settings_copy.disabled_plugins.remove(name);
                                    } else {
                                        settings_copy.disabled_plugins.insert(name.clone());
                                    }
                                }
                            }
                        });
                    }
                });
            });
    }
}
