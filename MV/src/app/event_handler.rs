use super::*;

impl App {
    pub fn forward_to_egui(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = egui::pos2(position.x as f32, position.y as f32);
                self.egui_pointer_pos = pos;
                self.egui_raw_input.events.push(egui::Event::PointerMoved(pos));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let egui_button = match button {
                    winit::event::MouseButton::Left   => egui::PointerButton::Primary,
                    winit::event::MouseButton::Right  => egui::PointerButton::Secondary,
                    winit::event::MouseButton::Middle => egui::PointerButton::Middle,
                    _ => return,
                };
                self.egui_raw_input.events.push(egui::Event::PointerButton {
                    pos: self.egui_pointer_pos,
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
                self.egui_raw_input.events.push(egui::Event::Scroll(scroll));
            }
            WindowEvent::Resized(size) => {
                self.egui_raw_input.screen_rect = Some(egui::Rect::from_min_size(
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
                if self.current_modifiers.shift_key() {
                    self.navigate_visualization(-1);
                } else {
                    self.navigate_visualization(1);
                }
            }
            ShortcutAction::PrevVisualization => {
                self.navigate_visualization(-1);
            }
            ShortcutAction::CycleWindowMode => {
                self.window_mode = self.window_mode.next();
                #[cfg(debug_assertions)]
                eprintln!("Window mode: {}", self.window_mode.label());
                if let Some(window) = self.window.as_ref().map(Arc::clone) {
                    #[cfg(target_os = "windows")]
                    {
                        self.apply_transparency(&window);
                        self.set_topmost(&window, self.window_mode.needs_topmost());
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = window.set_transparent(self.window_mode.needs_layered());
                    }
                }
            }
            ShortcutAction::CycleBeatSensitivity => {
                self.settings.beat_sensitivity = self.settings.beat_sensitivity.next();
                #[cfg(debug_assertions)]
                eprintln!("Beat sensitivity: {}", self.settings.beat_sensitivity.label());
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
                self.uniforms.intensity = (self.uniforms.intensity + INTENSITY_STEP).min(10.0);
            }
            ShortcutAction::DecreaseIntensity => {
                self.uniforms.intensity = (self.uniforms.intensity - INTENSITY_STEP).max(0.0);
            }
            ShortcutAction::ToggleFullscreen => {
                if let Some(window) = &self.window {
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
                self.show_device_selection = !self.show_device_selection;
            }
            ShortcutAction::ToggleShaderBrowser => {
                self.show_shader_browser = !self.show_shader_browser;
            }
            // These are handled directly in window_event with access to event_loop
            ShortcutAction::ToggleInfo | ShortcutAction::ToggleSettings | ShortcutAction::Exit => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let icon = {
            let image = image::load_from_memory(include_bytes!("../../assets/logo.png")).unwrap().to_rgba8();
            let (width, height) = image.dimensions();
            Icon::from_rgba(image.into_raw(), width, height).unwrap()
        };
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::PhysicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT))
            .with_window_icon(Some(icon));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(Arc::clone(&window));
        self.init_gpu(window);
        self.show_info = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() != window_id { return; }
        } else {
            return;
        }

        self.forward_to_egui(&event);

        match event {
            WindowEvent::Resized(new_size) => self.resize(new_size),
            WindowEvent::ModifiersChanged(modifiers) => self.current_modifiers = modifiers.state(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Escape), state: ElementState::Pressed, .. },
                ..
            } => {
                if let Some(window) = &self.window {
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                        window.set_cursor_visible(true);
                        window.set_minimized(true);
                    } else {
                        event_loop.exit();
                    }
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::F1), state: ElementState::Pressed, .. },
                ..
            } => {
                let old_show_info = self.show_info;
                self.show_info = !old_show_info;
                if self.show_info {
                    self.info_timer = Some(Instant::now());
                } else {
                    self.info_timer = None;
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::F2), state: ElementState::Pressed, .. },
                ..
            } => {
                self.settings.show_settings = !self.settings.show_settings;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::F4), state: ElementState::Pressed, .. },
                ..
            } => {
                self.show_shader_browser = !self.show_shader_browser;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key, state: ElementState::Pressed, .. },
                ..
            } => {
                self.handle_key_press(physical_key);
            }
            WindowEvent::RedrawRequested => {
                self.update();
                if let Err(e) = self.render() {
                    eprintln!("Render error: {:?}", e);
                    match e {
                        crate::error::AppError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            self.resize(self.window.as_ref().unwrap().inner_size());
                        }
                        crate::error::AppError::Surface(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
