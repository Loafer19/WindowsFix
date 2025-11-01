//! Main application logic and event handling

use crate::audio::AudioHandler;
use crate::constants::*;
use crate::error::AppResult;
use crate::gpu::{GpuResources, VisUniforms};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window};
#[cfg(target_os = "windows")]
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{COLORREF, HWND};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LAYERED_WINDOW_ATTRIBUTES_FLAGS};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED};

/// Main application state
pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuResources>,
    audio: AudioHandler,
    uniforms: VisUniforms,
    current_plugin_index: usize,
    transparent: bool,
    debug_info: bool,
}

impl App {
    /// Create a new application instance
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
                padding2: [0; 3],
                padding3: [0; 4],
            },
            current_plugin_index: 0,
            transparent: false,
            debug_info: false,
        }
    }

    /// Initialize GPU resources
    pub fn init_gpu(&mut self, window: Arc<Window>) {
        let gpu = pollster::block_on(GpuResources::new(window)).expect("Failed to initialize GPU");
        self.gpu = Some(gpu);
    }

    /// Update application state
    pub fn update(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            // Set mode based on current plugin index (each plugin has its own mode)
            self.uniforms.mode = self.current_plugin_index as u32;

            // Get audio data
            let audio_data = self.audio.buffer.lock().unwrap().clone();

            // Update GPU with new data
            gpu.update(&self.uniforms, &audio_data);
        }
    }

    /// Render a frame
    pub fn render(&mut self) -> AppResult<()> {
        if let Some(gpu) = &mut self.gpu {
            gpu.render(self.current_plugin_index)?;
        }
        Ok(())
    }

    /// Handle window resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu) = &mut self.gpu {
            gpu.resize(new_size);
            self.uniforms.resolution = [new_size.width as f32, new_size.height as f32];
        }
    }

    /// Handle keyboard input
    pub fn handle_key_press(&mut self, physical_key: PhysicalKey) {
        match physical_key {
            PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::KeyP) => {
                if let Some(gpu) = &self.gpu {
                    self.current_plugin_index = (self.current_plugin_index + 1) % gpu.plugins.len();
                    println!("Switched to plugin: {}", gpu.plugins[self.current_plugin_index].name);
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
                    {
                        if let Ok(RawWindowHandle::Win32(win32_handle)) = window.raw_window_handle() {
                            let hwnd = HWND(win32_handle.hwnd.get() as isize);
                            unsafe {
                                if self.transparent {
                                    // Enable layered window
                                    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as isize);
                                    // Set semi-transparent (alpha 150 out of 255)
                                    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 150, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
                                } else {
                                    // Disable layered window
                                    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !(WS_EX_LAYERED.0 as isize));
                                }
                            }
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = window.set_transparent(self.transparent);
                    }
                }
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
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() != window_id {
                return;
            }
        } else {
            return;
        }

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
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state: ElementState::Pressed, .. }, .. } => {
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
