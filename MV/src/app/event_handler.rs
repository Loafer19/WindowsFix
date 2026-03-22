use super::*;

impl App {
    pub fn forward_to_egui(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = egui::pos2(position.x as f32, position.y as f32);
                self.state.egui_pointer_pos = pos;
                self.state.egui_raw_input.events.push(egui::Event::PointerMoved(pos));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let egui_button = match button {
                    winit::event::MouseButton::Left   => egui::PointerButton::Primary,
                    winit::event::MouseButton::Right  => egui::PointerButton::Secondary,
                    winit::event::MouseButton::Middle => egui::PointerButton::Middle,
                    _ => return,
                };
                self.state.egui_raw_input.events.push(egui::Event::PointerButton {
                    pos: self.state.egui_pointer_pos,
                    button: egui_button,
                    pressed: matches!(state, ElementState::Pressed),
                    modifiers: egui::Modifiers::default(),
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(x, y) => egui::vec2(*x * 20.0, *y * 20.0),
                    MouseScrollDelta::PixelDelta(pos) => egui::vec2(pos.x as f32, pos.y as f32),
                };
                self.state.egui_raw_input.events.push(egui::Event::Scroll(scroll));
            }
            WindowEvent::Resized(size) => {
                self.state.egui_raw_input.screen_rect = Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(size.width as f32, size.height as f32),
                ));
            }
            _ => {}
        }
    }

    pub fn handle_key_press(&mut self, physical_key: PhysicalKey) {
        let key = match physical_key {
            PhysicalKey::Code(k) => k,
            _ => return,
        };

        use crate::input::ShortcutAction;
        let action = match crate::input::key_to_action(key) {
            Some(a) => a,
            None => return,
        };

        match action {
            ShortcutAction::NextVisualization => {
                if self.state.current_modifiers.shift_key() {
                    self.navigate_visualization(-1);
                } else {
                    self.navigate_visualization(1);
                }
            }
            ShortcutAction::PrevVisualization => {
                self.navigate_visualization(-1);
            }
            ShortcutAction::CycleWindowMode => {
                self.state.window_mode = self.state.window_mode.next();
                #[cfg(debug_assertions)]
                eprintln!("Window mode: {}", self.state.window_mode.label());
                if let Some(window) = self.state.window.as_ref().map(Arc::clone) {
                    #[cfg(target_os = "windows")]
                    {
                        self.apply_transparency(&window);
                        self.set_topmost(&window, self.state.window_mode.needs_topmost());
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = window.set_transparent(self.state.window_mode.needs_layered());
                    }
                }
            }
            ShortcutAction::CycleBeatSensitivity => {
                self.state.settings.beat_sensitivity = self.state.settings.beat_sensitivity.next();
                #[cfg(debug_assertions)]
                eprintln!("Beat sensitivity: {}", self.state.settings.beat_sensitivity.label());
            }
            ShortcutAction::DecreaseOpacity => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(false);
            }
            ShortcutAction::IncreaseOpacity => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(true);
            }
            ShortcutAction::IncreaseIntensity => {
                self.state.uniforms.intensity = (self.state.uniforms.intensity + INTENSITY_STEP).min(10.0);
            }
            ShortcutAction::DecreaseIntensity => {
                self.state.uniforms.intensity = (self.state.uniforms.intensity - INTENSITY_STEP).max(0.0);
            }
            ShortcutAction::ToggleFullscreen => {
                if let Some(window) = &self.state.window {
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                        window.set_cursor_visible(true);
                    } else {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        window.set_cursor_visible(false);
                    }
                }
            }
            ShortcutAction::ToggleDeviceSelector => {
                self.state.show_device_selection = !self.state.show_device_selection;
            }
            ShortcutAction::ToggleShaderBrowser => {
                self.state.show_shader_browser = !self.state.show_shader_browser;
            }
            // These are handled directly in window_event with access to event_loop
            ShortcutAction::ToggleInfo | ShortcutAction::ToggleSettings | ShortcutAction::Exit => {}
        }
    }
}


