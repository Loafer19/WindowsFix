use super::*;
use crate::visualization::ShaderInfo;
use crate::config::settings::{BeatSensitivity, ColorScheme};

impl App {
    pub fn init_gpu(&mut self, window: Arc<Window>) {
        let gpu = pollster::block_on(GpuResources::new(window)).expect("Failed to initialize GPU");
        self.state.gpu = Some(gpu);
    }

    pub fn init_audio(&mut self, device_index: usize) -> AppResult<()> {
        if let Some(device) = self.state.devices.get(device_index) {
            let audio_handler = AudioHandler::new(device.clone())?;
            self.state.audio = Some(audio_handler);
            if let Some(name) = device.name().ok() {
                self.state.settings.selected_device = Some(name);
                self.state.settings.save().ok();
            }
            Ok(())
        } else {
            Err(crate::common::error::AppError::Audio("Invalid device index".to_string()))
        }
    }

    pub(crate) fn rebuild_plugin_cache(&mut self) {
        if let Some(gpu) = &self.state.gpu {
            self.state.enabled_plugin_cache = gpu.plugins.iter()
                .enumerate()
                .filter(|(_, p)| !self.state.settings.disabled_plugins.contains(&p.name))
                .map(|(i, _)| i)
                .collect();
        }
    }

    pub(crate) fn navigate_visualization(&mut self, step: isize) {
        if self.state.enabled_plugin_cache.is_empty() {
            self.rebuild_plugin_cache();
        }
        if self.state.enabled_plugin_cache.is_empty() { return; }

        let current_pos = self.state.enabled_plugin_cache.iter()
            .position(|&i| i == self.state.current_plugin_index)
            .unwrap_or(0);

        let n = self.state.enabled_plugin_cache.len();
        let next_pos = ((current_pos as isize + step).rem_euclid(n as isize)) as usize;
        self.state.current_plugin_index = self.state.enabled_plugin_cache[next_pos];

        self.state.transition_active = true;
        self.state.transition_time = 0.0;
        self.state.last_mode_switch = Instant::now();

        #[cfg(debug_assertions)]
        if let Some(gpu) = &self.state.gpu {
            eprintln!("Switched to plugin: {}", gpu.plugins[self.state.current_plugin_index].name);
        }
    }

    pub fn update(&mut self) {
        if let Some(timer) = self.state.info_timer {
            if timer.elapsed() > Duration::from_secs(10) {
                self.state.show_info = false;
                self.state.info_timer = None;
            }
        }

        if let Some(index) = self.state.pending_device_index.take() {
            if let Err(e) = self.init_audio(index) {
                eprintln!("Failed to initialize audio: {:?}", e);
            }
        }

        // Sync opacity slider → transparency_level and re-apply if transparent
        #[cfg(target_os = "windows")]
        {
            let slider_level = (self.state.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32)
                .clamp(MIN_TRANSPARENCY_ALPHA as f32, MAX_TRANSPARENCY_ALPHA as f32) as u8;
            if slider_level != self.state.transparency_level {
                self.state.transparency_level = slider_level;
                if self.state.window_mode.needs_layered() {
                    if let Some(window) = self.state.window.as_ref().map(Arc::clone) {
                        self.apply_transparency(&window);
                    }
                }
            }
        }

        // Auto-switch modes (skip disabled plugins)
        if self.state.settings.auto_switch_modes {
            let switch_dur = Duration::from_secs_f32(self.state.settings.mode_switch_seconds);
            if self.state.last_mode_switch.elapsed() > switch_dur {
                self.navigate_visualization(1);
            }
        }

        // Advance transition using actual elapsed time
        if self.state.transition_active {
            let dt = self.state.last_frame_time.elapsed().as_secs_f32();
            self.state.transition_time += dt;
            if self.state.transition_time >= 0.5 {
                self.state.transition_active = false;
            }
        }
        self.state.last_frame_time = Instant::now();

        if let Some(gpu) = &mut self.state.gpu {
            self.state.uniforms.mode = self.state.current_plugin_index as u32;
            self.state.uniforms.smoothing_factor = self.state.settings.smoothing_factor;
            self.state.uniforms.gain = self.state.settings.gain;
            self.state.uniforms.color = self.state.settings.scheme_color();

            if let Some(audio) = &self.state.audio {
                let audio_guard = audio.buffer.lock().unwrap();
                let beat_threshold = match self.state.settings.beat_sensitivity {
                    BeatSensitivity::Low    => BEAT_THRESHOLD_HIGH,
                    BeatSensitivity::Medium => BEAT_THRESHOLD_MED,
                    BeatSensitivity::High   => BEAT_THRESHOLD_LOW,
                };
                gpu.update(&self.state.uniforms, &audio_guard, beat_threshold);
            } else {
                let silence = vec![0.0f32; crate::config::constants::SAMPLE_SIZE];
                gpu.update(&self.state.uniforms, &silence, BEAT_THRESHOLD_MED);
            }
        }
    }

    pub fn render(&mut self) -> AppResult<()> {
        let (width, height) = self.state.gpu.as_ref()
            .map(|g| (g.config.width, g.config.height))
            .unwrap_or((DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT));

        let plugin_name = self.state.gpu.as_ref()
            .and_then(|g| g.plugins.get(self.state.current_plugin_index))
            .map(|p| p.name.clone())
            .unwrap_or_default();

        let beat_intensity = self.state.gpu.as_ref().map(|g| g.beat_intensity).unwrap_or(0.0);

        let plugin_groups = {
            let names: Vec<String> = self.state.gpu.as_ref()
                .map(|g| g.plugins.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            build_plugin_groups(&names)
        };

        // Collect shader browser entries before closure
        let shader_browser_entries: Vec<(usize, String, Option<&'static ShaderInfo>)> = self.state.gpu.as_ref()
            .map(|g| g.plugins.iter().enumerate()
                .map(|(i, p)| (i, p.name.clone(), p.info))
                .collect())
            .unwrap_or_default();

        let current_plugin_idx = self.state.current_plugin_index;

        let mut settings_copy = self.state.settings.clone();
        let mut show_info = self.state.show_info;
        let window_mode = self.state.window_mode;
        let mut show_shader_browser = self.state.show_shader_browser;
        let mut new_plugin_index: Option<usize> = None;

        let mut raw_input = std::mem::take(&mut self.state.egui_raw_input);
        if raw_input.screen_rect.is_none() {
            raw_input.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            ));
        }
        let full_output = self.state.egui_ctx.run(raw_input, |ctx| {
            // ── Controls / Help panel (F1) ────────────────────────────────────
            egui::Window::new("ℹ Controls")
                .open(&mut show_info)
                .resizable(false)
                .collapsible(false)
                .default_width(340.0)
                .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new(&plugin_name).strong());
                    });
                    ui.horizontal(|ui| {
                        let device_name = settings_copy.selected_device.as_deref().unwrap_or("None");
                        ui.label(egui::RichText::new("🎤").size(14.0));
                        ui.label(device_name);
                        ui.separator();
                        ui.label(egui::RichText::new("🪟").size(14.0));
                        ui.label(window_mode.label());
                        ui.separator();
                        ui.label(egui::RichText::new("🥁").size(14.0));
                        ui.label(settings_copy.beat_sensitivity.label());
                    });
                    ui.separator();

                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            let mut last_category = "";
                            for shortcut in crate::input::SHORTCUTS {
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

            // ── Settings panel (F2) ───────────────────────────────────────────
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

            // ── Audio device selector (F3) ────────────────────────────────────
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

            // ── Shader Browser (F4) ──────────────────────────────────────────
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

            // ── Always-visible status overlay (bottom-left) ───────────────────
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
                        let (mode_color, mode_icon) = match window_mode {
                            WindowMode::Normal      => (egui::Color32::from_rgb(150, 150, 150), "◻"),
                            WindowMode::Transparent => (egui::Color32::from_rgb(100, 180, 255), "◈"),
                            WindowMode::Overlay     => (egui::Color32::from_rgb(255, 180, 80),  "◉"),
                        };
                        ui.label(egui::RichText::new(mode_icon).size(11.0).color(mode_color));
                        ui.label(
                            egui::RichText::new(window_mode.label())
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
        });

        self.state.show_info = show_info;
        self.state.settings = settings_copy;
        self.state.show_shader_browser = show_shader_browser;
        if let Some(idx) = new_plugin_index {
            self.state.current_plugin_index = idx;
            self.state.transition_active = true;
            self.state.transition_time = 0.0;
            self.state.last_mode_switch = Instant::now();
        }

        let ppp = full_output.pixels_per_point;
        let paint_jobs = self.state.egui_ctx.tessellate(full_output.shapes, ppp);
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: ppp,
        };

        if let Some(gpu) = &mut self.state.gpu {
            gpu.render(self.state.current_plugin_index, &paint_jobs, &screen_desc, &full_output.textures_delta)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu) = &mut self.state.gpu {
            gpu.resize(new_size);
            self.state.uniforms.resolution = [new_size.width as f32, new_size.height as f32];
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn apply_transparency(&self, window: &Window) {
        if let Ok(window_handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = window_handle.as_ref() {
                let hwnd = HWND(win32_handle.hwnd.get() as isize);
                unsafe {
                    if self.state.window_mode.needs_layered() {
                        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        ex_style |= WS_EX_LAYERED.0 as isize;
                        if self.state.window_mode.needs_click_through() {
                            ex_style |= WS_EX_TRANSPARENT.0 as isize;
                        } else {
                            ex_style &= !(WS_EX_TRANSPARENT.0 as isize);
                        }
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
                        let alpha = (self.state.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32)
                            .clamp(MIN_TRANSPARENCY_ALPHA as f32, MAX_TRANSPARENCY_ALPHA as f32) as u8;
                        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
                    } else {
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        SetWindowLongPtrW(
                            hwnd,
                            GWL_EXSTYLE,
                            ex_style & !(WS_EX_LAYERED.0 as isize) & !(WS_EX_TRANSPARENT.0 as isize),
                        );
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn set_topmost(&self, window: &Window, topmost: bool) {
        if let Ok(window_handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = window_handle.as_ref() {
                let hwnd = HWND(win32_handle.hwnd.get() as isize);
                let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
                unsafe {
                    let _ = SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn adjust_transparency_level(&mut self, increase: bool) {
        let step = TRANSPARENCY_STEP as f32 / MAX_TRANSPARENCY_ALPHA as f32;
        if increase {
            self.state.settings.transparency = (self.state.settings.transparency + step).min(1.0);
        } else {
            self.state.settings.transparency = (self.state.settings.transparency - step).max(0.1);
        }
        self.state.transparency_level = (self.state.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32) as u8;
        #[cfg(debug_assertions)]
        eprintln!("Opacity: {}%", (self.state.settings.transparency * 100.0) as u32);
        if self.state.window_mode.needs_layered() {
            if let Some(window) = self.state.window.as_ref().map(Arc::clone) {
                self.apply_transparency(&window);
            }
        }
    }
}
