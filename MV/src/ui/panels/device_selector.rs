//! Audio device selector panel (F3)

use crate::app::App;
use cpal::traits::DeviceTrait;

impl App {
    pub fn show_device_selector(&mut self, ctx: &egui::Context) {
        let mut pending = self.state.pending_device_index;

        egui::Window::new("🎤 Audio Device")
            .open(&mut self.state.show_device_selection)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
            .show(ctx, |ui| {
                ui.label("Choose an audio input device:");
                ui.separator();
                for (i, device) in self.state.devices.iter().enumerate() {
                    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                    let is_selected = self.state.settings.selected_device.as_ref() == Some(&name);
                    if ui.selectable_label(is_selected, &name).clicked() {
                        pending = Some(i);
                    }
                }
            });

        self.state.pending_device_index = pending;
    }
}
