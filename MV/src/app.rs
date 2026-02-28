//! Main application logic and event handling

use crate::audio::AudioHandler;
use crate::constants::*;
use crate::error::AppResult;
use crate::gpu::GpuResources;
use crate::settings::{AppSettings, ColorScheme};
use crate::types::VisUniforms;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window};
#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{COLORREF, HWND};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LAYERED_WINDOW_ATTRIBUTES_FLAGS};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED};

pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuResources>,
    audio: AudioHandler,
    uniforms: VisUniforms,
    current_plugin_index: usize,
    transparent: bool,
    transparency_level: u8,
    show_info: bool,
    info_timer: Option<Instant>,
    settings: AppSettings,
    egui_ctx: egui::Context,
    egui_raw_input: egui::RawInput,
    egui_pointer_pos: egui::Pos2,
    transition_time: f32,
    transition_active: bool,
    last_mode_switch: Instant,
    last_frame_time: Instant,
}

impl App {
    pub fn new(audio: AudioHandler) -> Self {
        Self {
            window: None,
            gpu: None,
            audio,
            uniforms: VisUniforms {
                color: DEFAULT_COLOR,
                intensity: DEFAULT_INTENSITY,
                padding1: 0.0,
                resolution: [DEFAULT_WINDOW_WIDTH as f32, DEFAULT_WINDOW_HEIGHT as f32],
                mode: 0,
                padding3a: 0,
                padding3b: 0,
                padding3c: 0,
                padding2: [0; 3],
                time: 0.0,
                bass_energy: 0.0,
                smoothing_factor: 0.1,
                gain: 1.5,
                padding4: 0.0,
            },
            current_plugin_index: 0,
            transparent: false,
            transparency_level: DEFAULT_TRANSPARENCY,
            show_info: false,
            info_timer: None,
            settings: AppSettings::new(),
            egui_ctx: egui::Context::default(),
            egui_raw_input: egui::RawInput::default(),
            egui_pointer_pos: egui::Pos2::ZERO,
            transition_time: 0.0,
            transition_active: false,
            last_mode_switch: Instant::now(),
            last_frame_time: Instant::now(),
        }
    }

    pub fn init_gpu(&mut self, window: Arc<Window>) {
        let gpu = pollster::block_on(GpuResources::new(window)).expect("Failed to initialize GPU");
        self.gpu = Some(gpu);
    }

    pub fn update(&mut self) {
        if let Some(timer) = self.info_timer {
            if timer.elapsed() > Duration::from_secs(10) {
                self.show_info = false;
                self.info_timer = None;
            }
        }

        // Sync opacity slider → transparency_level and re-apply if transparent
        #[cfg(target_os = "windows")]
        {
            let slider_level = (self.settings.transparency * 255.0).clamp(25.0, 255.0) as u8;
            if slider_level != self.transparency_level {
                self.transparency_level = slider_level;
                if self.transparent {
                    if let Some(window) = self.window.clone() {
                        self.apply_transparency(&window);
                    }
                }
            }
        }

        // Auto-switch modes (skip disabled plugins)
        if self.settings.auto_switch_modes {
            let switch_dur = Duration::from_secs_f32(self.settings.mode_switch_seconds);
            if self.last_mode_switch.elapsed() > switch_dur {
                if let Some(gpu) = &self.gpu {
                    let num = gpu.plugins.len();
                    let mut next = (self.current_plugin_index + 1) % num;
                    let mut found = false;
                    for _ in 0..num {
                        if !self.settings.disabled_plugins.contains(&gpu.plugins[next].name) {
                            found = true;
                            break;
                        }
                        next = (next + 1) % num;
                    }
                    if found {
                        self.current_plugin_index = next;
                    }
                    self.last_mode_switch = Instant::now();
                    self.transition_active = true;
                    self.transition_time = 0.0;
                }
            }
        }

        // Advance transition using actual elapsed time
        if self.transition_active {
            let dt = self.last_frame_time.elapsed().as_secs_f32();
            self.transition_time += dt;
            if self.transition_time >= 0.5 {
                self.transition_active = false;
            }
        }
        self.last_frame_time = Instant::now();

        if let Some(gpu) = &mut self.gpu {
            self.uniforms.mode = self.current_plugin_index as u32;
            self.uniforms.smoothing_factor = self.settings.smoothing_factor;
            self.uniforms.gain = self.settings.gain;
            self.uniforms.color = self.settings.scheme_color();

            let audio_data = self.audio.buffer.lock().unwrap().clone();
            gpu.update(&self.uniforms, &audio_data);
        }
    }

    pub fn render(&mut self) -> AppResult<()> {
        let (width, height) = self.gpu.as_ref()
            .map(|g| (g.config.width, g.config.height))
            .unwrap_or((DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT));

        let plugin_name = self.gpu.as_ref()
            .and_then(|g| g.plugins.get(self.current_plugin_index))
            .map(|p| p.name.clone())
            .unwrap_or_default();

        // Collect plugin groups before the closure to avoid borrowing self.gpu inside it.
        let plugin_groups = {
            let names: Vec<String> = self.gpu.as_ref()
                .map(|g| g.plugins.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            build_plugin_groups(&names)
        };

        let mut settings_copy = self.settings.clone();
        let mut show_info = self.show_info;
        // Take accumulated input; set screen_rect if not already set
        let mut raw_input = std::mem::take(&mut self.egui_raw_input);
        if raw_input.screen_rect.is_none() {
            raw_input.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            ));
        }
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // --- Info / Controls window ---
            egui::Window::new("ℹ Controls")
                .open(&mut show_info)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Mode: {}", plugin_name));
                    ui.separator();
                    ui.label("Space/P  – switch mode");
                    ui.label("F1       – settings");
                    ui.label("F        – fullscreen");
                    ui.label("T        – toggle transparency");
                    ui.label("[/]      – opacity level");
                    ui.label("Up/Down  – intensity");
                    ui.label("I        – toggle info");
                    ui.label("Esc      – exit");
                });

            // --- Settings window ---
            egui::Window::new("⚙ Settings")
                .open(&mut settings_copy.show_settings)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.add(egui::Slider::new(&mut settings_copy.smoothing_factor, 0.01..=0.3).text("Smoothing"));
                    ui.add(egui::Slider::new(&mut settings_copy.gain, 0.5..=5.0).text("Gain"));
                    ui.add(egui::Slider::new(&mut settings_copy.bass_boost, 0.0..=2.0).text("Bass Boost"));
                    ui.add(egui::Slider::new(&mut settings_copy.transparency, 0.1..=1.0).text("Opacity"));
                    ui.separator();
                    ui.label("Color Scheme:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Classic, "Classic");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Neon,    "Neon");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Pastel,  "Pastel");
                        ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Fire,    "Fire");
                    });
                    ui.separator();
                    ui.checkbox(&mut settings_copy.auto_switch_modes, "Auto-switch modes");
                    if settings_copy.auto_switch_modes {
                        ui.add(egui::Slider::new(&mut settings_copy.mode_switch_seconds, 5.0..=120.0).text("Switch interval (s)"));
                    }
                    ui.separator();
                    ui.label("Effects:");
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

        // Write back values that the egui closures may have changed.
        self.show_info = show_info;
        self.settings = settings_copy;

        let ppp = full_output.pixels_per_point;
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, ppp);
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: ppp,
        };

        if let Some(gpu) = &mut self.gpu {
            gpu.render(self.current_plugin_index, &paint_jobs, &screen_desc, &full_output.textures_delta)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu) = &mut self.gpu {
            gpu.resize(new_size);
            self.uniforms.resolution = [new_size.width as f32, new_size.height as f32];
        }
    }

    #[cfg(target_os = "windows")]
    fn apply_transparency(&self, window: &Window) {
        if let Ok(window_handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = window_handle.as_ref() {
                let hwnd = HWND(win32_handle.hwnd.get() as isize);
                unsafe {
                    if self.transparent {
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as isize);
                        let alpha = (self.settings.transparency * 255.0).clamp(25.0, 255.0) as u8;
                        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
                    } else {
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !(WS_EX_LAYERED.0 as isize));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn adjust_transparency_level(&mut self, increase: bool) {
        let step = TRANSPARENCY_STEP as f32 / 255.0;
        if increase {
            self.settings.transparency = (self.settings.transparency + step).min(1.0);
        } else {
            self.settings.transparency = (self.settings.transparency - step).max(0.1);
        }
        self.transparency_level = (self.settings.transparency * 255.0) as u8;
        println!("Opacity: {}%", (self.settings.transparency * 100.0) as u32);
        if self.transparent {
            if let Some(window) = self.window.clone() {
                self.apply_transparency(&window);
            }
        }
    }

    /// Forward winit window events to egui's raw input accumulator.
    ///
    /// `egui-winit` 0.27 requires winit 0.29 and is type-incompatible with the
    /// winit 0.30 `ApplicationHandler` API used by this app, so mouse/scroll
    /// events are forwarded manually instead.
    fn forward_to_egui(&mut self, event: &WindowEvent) {
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
        let old_show_info = self.show_info;
        self.show_info = false;
        match physical_key {
            PhysicalKey::Code(KeyCode::KeyI) => {
                self.show_info = !old_show_info;
                if self.show_info {
                    self.info_timer = Some(Instant::now());
                } else {
                    self.info_timer = None;
                }
            }
            PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::KeyP) => {
                if let Some(gpu) = &self.gpu {
                    let num = gpu.plugins.len();
                    let mut next = (self.current_plugin_index + 1) % num;
                    let mut found = false;
                    for _ in 0..num {
                        if !self.settings.disabled_plugins.contains(&gpu.plugins[next].name) {
                            found = true;
                            break;
                        }
                        next = (next + 1) % num;
                    }
                    if found {
                        self.current_plugin_index = next;
                    }
                    println!("Switched to plugin: {}", gpu.plugins[self.current_plugin_index].name);
                    self.transition_active = true;
                    self.transition_time = 0.0;
                    self.last_mode_switch = Instant::now();
                }
            }
            PhysicalKey::Code(KeyCode::KeyF) => {
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
            PhysicalKey::Code(KeyCode::KeyT) => {
                if let Some(window) = &self.window {
                    self.transparent = !self.transparent;
                    #[cfg(target_os = "windows")]
                    self.apply_transparency(window);
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = window.set_transparent(self.transparent);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::BracketLeft) => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(false);
            }
            PhysicalKey::Code(KeyCode::BracketRight) => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(true);
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.uniforms.intensity = (self.uniforms.intensity + INTENSITY_STEP).min(10.0);
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.uniforms.intensity = (self.uniforms.intensity - INTENSITY_STEP).max(0.0);
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let icon = {
            let image = image::load_from_memory(include_bytes!("../assets/logo.png")).unwrap().to_rgba8();
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
        self.show_info = true;
        self.info_timer = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() != window_id { return; }
        } else {
            return;
        }

        // Forward mouse/scroll events to egui's input accumulator
        self.forward_to_egui(&event);

        match event {
            WindowEvent::Resized(new_size) => self.resize(new_size),
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
                self.settings.show_settings = !self.settings.show_settings;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key, state: ElementState::Pressed, .. }, ..
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

// ──────────────────────────────────────────────────────────────────────────────
// Plugin grouping helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Assign a display group to a plugin based on its name.
fn plugin_group(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.ends_with("_3d") || n.contains("3d") {
        "🔮 3D Effects"
    } else if n.contains("waveform") || n.contains("oscilloscope") {
        "🌊 Waveform"
    } else if n.contains("spectrum") || n.contains("bars") || n.contains("gradient") || n.contains("circular") {
        "🎵 Spectrum"
    } else {
        "✨ Abstract"
    }
}

/// Build an ordered list of `(group_label, sorted_plugin_names)` from a flat name list.
fn build_plugin_groups(names: &[String]) -> Vec<(String, Vec<String>)> {
    const ORDER: &[&str] = &["🎵 Spectrum", "🌊 Waveform", "🔮 3D Effects", "✨ Abstract"];
    let mut map: std::collections::HashMap<&'static str, Vec<String>> = std::collections::HashMap::new();
    for name in names {
        let group = plugin_group(name);
        map.entry(group).or_default().push(name.clone());
    }
    for names in map.values_mut() {
        names.sort();
    }
    ORDER.iter()
        .filter_map(|&g| map.remove(g).map(|ns| (g.to_string(), ns)))
        .collect()
}
