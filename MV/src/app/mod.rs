//! Main application logic and event handling

pub mod lifecycle;
pub mod event_handler;
pub mod state;

use crate::input::audio::AudioHandler;
use crate::config::constants::*;
use crate::common::error::AppResult;
use crate::graphics::GpuResources;
use crate::config::settings::{AppSettings, BeatSensitivity, ColorScheme};
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

pub use state::{AppState, WindowMode};

pub struct App {
    pub(crate) state: AppState,
}

impl App {
    pub fn new(devices: Vec<cpal::Device>, settings: AppSettings) -> Self {
        Self {
            state: AppState::new(devices, settings),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::PhysicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // Set the icon after window creation
        let icon = {
            // Try to load PNG first
            match image::load_from_memory(include_bytes!("../../assets/logo.png")) {
                Ok(image) => {
                    // Resize to 32x32 for icon
                    let resized = image.resize(32, 32, image::imageops::FilterType::Lanczos3);
                    let rgba = resized.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    eprintln!("Loaded and resized PNG: {}x{}", width, height);
                    match Icon::from_rgba(rgba.into_raw(), width, height) {
                        Ok(icon) => Some(icon),
                        Err(e) => {
                            eprintln!("Failed to create icon from PNG: {:?}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load logo.png: {:?}, trying ICO", e);
                    // Fallback to ICO
                    match image::load_from_memory(include_bytes!("../../assets/logo.ico")) {
                        Ok(image) => {
                            // Resize to 32x32 for icon
                            let resized = image.resize(32, 32, image::imageops::FilterType::Lanczos3);
                            let rgba = resized.to_rgba8();
                            let (width, height) = rgba.dimensions();
                            eprintln!("Loaded and resized ICO: {}x{}", width, height);
                            match Icon::from_rgba(rgba.into_raw(), width, height) {
                                Ok(icon) => Some(icon),
                                Err(e) => {
                                    eprintln!("Failed to create icon from ICO: {:?}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to load logo.ico: {:?}", e);
                            None
                        }
                    }
                }
            }
        };
        if let Some(icon) = icon {
            window.set_window_icon(Some(icon));
        }

        self.state.window = Some(Arc::clone(&window));
        self.init_gpu(window);
        self.state.show_info = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.state.window {
            if window.id() != window_id { return; }
        } else {
            return;
        }

        self.forward_to_egui(&event);

        match event {
            WindowEvent::Resized(new_size) => self.resize(new_size),
            WindowEvent::ModifiersChanged(modifiers) => self.state.current_modifiers = modifiers.state(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Escape), state: ElementState::Pressed, .. },
                ..
            } => {
                if let Some(window) = &self.state.window {
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
                let old_show_info = self.state.show_info;
                self.state.show_info = !old_show_info;
                if self.state.show_info {
                    self.state.info_timer = Some(Instant::now());
                } else {
                    self.state.info_timer = None;
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::F2), state: ElementState::Pressed, .. },
                ..
            } => {
                self.state.settings.show_settings = !self.state.settings.show_settings;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::F4), state: ElementState::Pressed, .. },
                ..
            } => {
                self.state.show_shader_browser = !self.state.show_shader_browser;
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
                        crate::common::error::AppError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            self.resize(self.state.window.as_ref().unwrap().inner_size());
                        }
                        crate::common::error::AppError::Surface(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.state.window {
            window.request_redraw();
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Plugin grouping helpers
// ──────────────────────────────────────────────────────────────────────────────

fn plugin_group(name: &str) -> &'static str {
    crate::visualization::shader_info(name)
        .map(|info| info.category.label())
        .unwrap_or("✨ Abstract")
}

pub(crate) fn build_plugin_groups(names: &[String]) -> Vec<(String, Vec<String>)> {
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
