//! Main application logic and event handling

use crate::audio::AudioHandler;
use crate::constants::*;
use crate::error::AppResult;
use crate::gpu::GpuResources;
use crate::settings::{AppSettings, BeatSensitivity, ColorScheme};
use crate::types::VisUniforms;
use cpal::traits::DeviceTrait;
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
use windows::Win32::UI::WindowsAndMessaging::{
    SetLayeredWindowAttributes, SetWindowPos, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
    HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

// ──────────────────────────────────────────────────────────────────────────────
// Window mode state machine
// ──────────────────────────────────────────────────────────────────────────────

/// The three distinct compositing modes the window can be in.
///
/// Cycling order: `Normal` → `Transparent` → `Overlay` → `Normal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    /// Standard opaque window, no special layering.
    #[default]
    Normal,
    /// Semi-transparent layered window; still receives mouse input.
    Transparent,
    /// Always-on-top, semi-transparent, and click-through (mouse events fall
    /// through to whatever is beneath the window).
    Overlay,
}

impl WindowMode {
    /// Advance to the next mode in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Normal       => Self::Transparent,
            Self::Transparent  => Self::Overlay,
            Self::Overlay      => Self::Normal,
        }
    }

    /// Human-readable label used in log messages and the UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal      => "Normal",
            Self::Transparent => "Transparent",
            Self::Overlay     => "Overlay",
        }
    }

    /// Returns `true` when a layered (transparent) window style is needed.
    pub fn needs_layered(self) -> bool {
        matches!(self, Self::Transparent | Self::Overlay)
    }

    /// Returns `true` when the window should be click-through.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn needs_click_through(self) -> bool {
        matches!(self, Self::Overlay)
    }

    /// Returns `true` when the window should sit above all other windows.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn needs_topmost(self) -> bool {
        matches!(self, Self::Overlay)
    }
}

pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuResources>,
    audio: Option<AudioHandler>,
    devices: Vec<cpal::Device>,
    uniforms: VisUniforms,
    current_plugin_index: usize,
    /// Current window compositing mode (Normal / Transparent / Overlay).
    window_mode: WindowMode,
    transparency_level: u8,
    show_info: bool,
    info_timer: Option<Instant>,
    show_device_selection: bool,
    pending_device_index: Option<usize>,
    settings: AppSettings,
    egui_ctx: egui::Context,
    egui_raw_input: egui::RawInput,
    egui_pointer_pos: egui::Pos2,
    current_modifiers: winit::keyboard::ModifiersState,
    transition_time: f32,
    transition_active: bool,
    last_mode_switch: Instant,
    last_frame_time: Instant,
}

impl App {
    pub fn new(devices: Vec<cpal::Device>, settings: AppSettings) -> Self {
        let mut audio = None;
        let mut show_device_selection = false;

        if let Some(selected_name) = &settings.selected_device {
            if let Some(index) = devices.iter().position(|d| d.name().ok().as_ref() == Some(selected_name)) {
                if let Ok(audio_handler) = AudioHandler::new(devices[index].clone()) {
                    audio = Some(audio_handler);
                } else {
                    // Failed to init, show selection
                    show_device_selection = true;
                }
            } else {
                // Device not found, show selection
                show_device_selection = true;
            }
        } else {
            show_device_selection = true;
        }

        Self {
            window: None,
            gpu: None,
            audio,
            devices,
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
                beat_intensity: 0.0,
            },
            current_plugin_index: 0,
            window_mode: WindowMode::Normal,
            transparency_level: DEFAULT_TRANSPARENCY,
            show_info: false,
            info_timer: None,
            show_device_selection,
            pending_device_index: None,
            settings,
            egui_ctx: egui::Context::default(),
            egui_raw_input: egui::RawInput::default(),
            egui_pointer_pos: egui::Pos2::ZERO,
            current_modifiers: winit::keyboard::ModifiersState::default(),
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

    pub fn init_audio(&mut self, device_index: usize) -> AppResult<()> {
        if let Some(device) = self.devices.get(device_index) {
            let audio_handler = AudioHandler::new(device.clone())?;
            self.audio = Some(audio_handler);
            if let Some(name) = device.name().ok() {
                self.settings.selected_device = Some(name);
                self.settings.save().ok();
            }
            Ok(())
        } else {
            Err(crate::error::AppError::Audio("Invalid device index".to_string()))
        }
    }

    pub fn update(&mut self) {
        if let Some(timer) = self.info_timer {
            if timer.elapsed() > Duration::from_secs(10) {
                self.show_info = false;
                self.info_timer = None;
            }
        }

        if let Some(index) = self.pending_device_index.take() {
            if let Err(e) = self.init_audio(index) {
                eprintln!("Failed to initialize audio: {:?}", e);
            }
        }

        // Sync opacity slider → transparency_level and re-apply if transparent
        #[cfg(target_os = "windows")]
        {
            let slider_level = (self.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32)
                .clamp(MIN_TRANSPARENCY_ALPHA as f32, MAX_TRANSPARENCY_ALPHA as f32) as u8;
            if slider_level != self.transparency_level {
                self.transparency_level = slider_level;
                if self.window_mode.needs_layered() {
                    if let Some(window) = self.window.as_ref().map(Arc::clone) {
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

            if let Some(audio) = &self.audio {
                // Hold the MutexGuard for the entire call to `gpu.update` so the
                // audio buffer slice remains valid.  Passing `&audio_guard` directly
                // (instead of `.clone()`) avoids allocating ~2 KB every frame.
                let audio_guard = audio.buffer.lock().unwrap();
                let beat_threshold = match self.settings.beat_sensitivity {
                    BeatSensitivity::Low    => BEAT_THRESHOLD_HIGH,
                    BeatSensitivity::Medium => BEAT_THRESHOLD_MED,
                    BeatSensitivity::High   => BEAT_THRESHOLD_LOW,
                };
                gpu.update(&self.uniforms, &audio_guard, beat_threshold);
            } else {
                // No audio device selected — feed silence so shaders still animate.
                let silence = vec![0.0f32; crate::constants::SAMPLE_SIZE];
                gpu.update(&self.uniforms, &silence, BEAT_THRESHOLD_MED);
            }
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

        let beat_intensity = self.gpu.as_ref().map(|g| g.beat_intensity).unwrap_or(0.0);

        // Collect plugin groups before the closure to avoid borrowing self.gpu inside it.
        let plugin_groups = {
            let names: Vec<String> = self.gpu.as_ref()
                .map(|g| g.plugins.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            build_plugin_groups(&names)
        };

        let mut settings_copy = self.settings.clone();
        let mut show_info = self.show_info;
        let window_mode = self.window_mode;
        // Take accumulated input; set screen_rect if not already set
        let mut raw_input = std::mem::take(&mut self.egui_raw_input);
        if raw_input.screen_rect.is_none() {
            raw_input.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            ));
        }
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // ── Controls / Help panel (F1) ────────────────────────────────────
            egui::Window::new("ℹ Controls")
                .open(&mut show_info)
                .resizable(false)
                .collapsible(false)
                .default_width(340.0)
                .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
                .show(ctx, |ui| {
                    // ── Live status ───────────────────────────────────────────
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

                    // ── Grouped keyboard shortcuts ────────────────────────────
                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            let section = |ui: &mut egui::Ui, label: &str| {
                                ui.label(egui::RichText::new(label).strong().color(egui::Color32::from_rgb(180, 180, 90)));
                                ui.end_row();
                            };
                            let row = |ui: &mut egui::Ui, key: &str, desc: &str| {
                                ui.label(egui::RichText::new(key).monospace().strong());
                                ui.label(desc);
                                ui.end_row();
                            };

                            section(ui, "── Visualizations ──");
                            row(ui, "Space / M",    "Next visualization");
                            row(ui, "Shift+Space",  "Previous visualization");

                            section(ui, "── Window ──");
                            row(ui, "T",            "Cycle: Normal → Transparent → Overlay");
                            row(ui, "← / →",       "Decrease / increase opacity");
                            row(ui, "F11",          "Toggle fullscreen");

                            section(ui, "── Audio ──");
                            row(ui, "B",            "Cycle beat sensitivity: Low / Med / High");
                            row(ui, "↑ / ↓",       "Increase / decrease intensity");

                            section(ui, "── Interface ──");
                            row(ui, "F1",           "Toggle this help panel");
                            row(ui, "F2",           "Open settings");
                            row(ui, "F3",           "Select audio device");

                            section(ui, "── Application ──");
                            row(ui, "Esc",          "Exit (or exit fullscreen)");
                        });
                });

            // ── Settings panel (F2) ───────────────────────────────────────────
            egui::Window::new("⚙ Settings")
                .open(&mut settings_copy.show_settings)
                .resizable(true)
                .collapsible(false)
                .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
                .show(ctx, |ui| {
                    // ── Audio Processing ──────────────────────────────────
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

                    // ── Visual ────────────────────────────────────────────
                    ui.collapsing("🎨 Visual", |ui| {
                        ui.add(egui::Slider::new(&mut settings_copy.transparency, 0.1..=1.0).text("Opacity"));
                        ui.horizontal(|ui| {
                            ui.label("Color Scheme:");
                            ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Classic, "Classic");
                            ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Neon,    "Neon");
                            ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Pastel,  "Pastel");
                            ui.selectable_value(&mut settings_copy.color_scheme, ColorScheme::Fire,    "Fire");
                        });
                    });

                    // ── Playback ──────────────────────────────────────────
                    ui.collapsing("▶ Playback", |ui| {
                        ui.checkbox(&mut settings_copy.auto_switch_modes, "Auto-switch modes");
                        if settings_copy.auto_switch_modes {
                            ui.add(egui::Slider::new(&mut settings_copy.mode_switch_seconds, 5.0..=120.0).text("Switch interval (s)"));
                        }
                    });

                    // ── Effects ───────────────────────────────────────────
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
            let mut pending = self.pending_device_index;
            egui::Window::new("🎤 Audio Device")
                .open(&mut self.show_device_selection)
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::window(&ctx.style()).shadow(egui::epaint::Shadow::NONE))
                .show(ctx, |ui| {
                    ui.label("Choose an audio input device:");
                    ui.separator();
                    for (i, device) in self.devices.iter().enumerate() {
                        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                        let is_selected = self.settings.selected_device.as_ref() == Some(&name);
                        if ui.selectable_label(is_selected, &name).clicked() {
                            pending = Some(i);
                        }
                    }
                });
            self.pending_device_index = pending;

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
                        // Visualization name
                        ui.label(
                            egui::RichText::new(&plugin_name)
                                .monospace()
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        );
                        ui.label(egui::RichText::new("│").size(11.0).color(egui::Color32::DARK_GRAY));
                        // Window mode badge
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
                        // Beat intensity bar (8 segments)
                        let segments = 8usize;
                        // Clamp before rounding to guard against beat_intensity slightly
                        // exceeding 1.0 due to floating-point arithmetic in the detector.
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
                    // SAFETY: `hwnd` is a valid Win32 window handle obtained directly from
                    // winit's raw window handle for the current window.  We only modify
                    // the window's extended-style flags (WS_EX_LAYERED / WS_EX_TRANSPARENT)
                    // and the layered-window alpha, which are well-defined Win32 operations.
                    if self.window_mode.needs_layered() {
                        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        ex_style |= WS_EX_LAYERED.0 as isize;
                        if self.window_mode.needs_click_through() {
                            // Overlay mode: add WS_EX_TRANSPARENT so mouse events fall through.
                            ex_style |= WS_EX_TRANSPARENT.0 as isize;
                        } else {
                            // Regular transparency: ensure click-through flag is cleared.
                            ex_style &= !(WS_EX_TRANSPARENT.0 as isize);
                        }
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
                        let alpha = (self.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32)
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

    /// Set or clear the always-on-top (topmost) flag for the window on Windows.
    #[cfg(target_os = "windows")]
    fn set_topmost(&self, window: &Window, topmost: bool) {
        if let Ok(window_handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = window_handle.as_ref() {
                let hwnd = HWND(win32_handle.hwnd.get() as isize);
                let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
                unsafe {
                    // SAFETY: `hwnd` is a valid Win32 window handle from winit.
                    // SetWindowPos with SWP_NOMOVE | SWP_NOSIZE only changes the Z-order.
                    let _ = SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn adjust_transparency_level(&mut self, increase: bool) {
        let step = TRANSPARENCY_STEP as f32 / MAX_TRANSPARENCY_ALPHA as f32;
        if increase {
            self.settings.transparency = (self.settings.transparency + step).min(1.0);
        } else {
            self.settings.transparency = (self.settings.transparency - step).max(0.1);
        }
        self.transparency_level = (self.settings.transparency * MAX_TRANSPARENCY_ALPHA as f32) as u8;
        #[cfg(debug_assertions)]
        eprintln!("Opacity: {}%", (self.settings.transparency * 100.0) as u32);
        if self.window_mode.needs_layered() {
            if let Some(window) = self.window.as_ref().map(Arc::clone) {
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
        match physical_key {
            PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::KeyM) => {
                if let Some(gpu) = &self.gpu {
                    let num = gpu.plugins.len();
                    if num == 0 {
                        return; // No plugins loaded; nothing to switch to.
                    }
                    let step = if self.current_modifiers.shift_key() { num - 1 } else { 1 };
                    let mut next = (self.current_plugin_index + step) % num;
                    let mut found = false;
                    for _ in 0..num {
                        if !self.settings.disabled_plugins.contains(&gpu.plugins[next].name) {
                            found = true;
                            break;
                        }
                        next = (next + step) % num;
                    }
                    if found {
                        self.current_plugin_index = next;
                    }
                    #[cfg(debug_assertions)]
                    eprintln!("Switched to plugin: {}", gpu.plugins[self.current_plugin_index].name);
                    self.transition_active = true;
                    self.transition_time = 0.0;
                    self.last_mode_switch = Instant::now();
                }
            }
            PhysicalKey::Code(KeyCode::KeyT) => {
                // Advance window mode through the cycle: Normal → Transparent → Overlay → Normal.
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
            PhysicalKey::Code(KeyCode::KeyB) => {
                self.settings.beat_sensitivity = self.settings.beat_sensitivity.next();
                #[cfg(debug_assertions)]
                eprintln!("Beat sensitivity: {}", self.settings.beat_sensitivity.label());
            }
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(false);
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                #[cfg(target_os = "windows")]
                self.adjust_transparency_level(true);
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.uniforms.intensity = (self.uniforms.intensity + INTENSITY_STEP).min(10.0);
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.uniforms.intensity = (self.uniforms.intensity - INTENSITY_STEP).max(0.0);
            }
            PhysicalKey::Code(KeyCode::F11) => {
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
            PhysicalKey::Code(KeyCode::F3) => {
                self.show_device_selection = !self.show_device_selection;
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
        self.show_info = false;
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
    } else if n.contains("heatmap") {
        "🌡 Heatmap"
    } else if n.contains("spectrum") || n.contains("bars") || n.contains("gradient") || n.contains("circular") {
        "🎵 Spectrum"
    } else {
        "✨ Abstract"
    }
}

/// Build an ordered list of `(group_label, sorted_plugin_names)` from a flat name list.
fn build_plugin_groups(names: &[String]) -> Vec<(String, Vec<String>)> {
    const ORDER: &[&str] = &["🎵 Spectrum", "🌡 Heatmap", "🌊 Waveform", "🔮 3D Effects", "✨ Abstract"];
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
