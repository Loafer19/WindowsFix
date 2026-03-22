//! Application state management

use crate::input::audio::AudioHandler;
use crate::config::constants::*;
use crate::config::settings::AppSettings;
use crate::common::types::VisUniforms;
use cpal::traits::DeviceTrait;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

// ──────────────────────────────────────────────────────────────────────────────
// Window mode state machine
// ──────────────────────────────────────────────────────────────────────────────

/// The three distinct compositing modes the window can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Normal,
    Transparent,
    Overlay,
}

impl WindowMode {
    pub fn next(self) -> Self {
        match self {
            Self::Normal       => Self::Transparent,
            Self::Transparent  => Self::Overlay,
            Self::Overlay      => Self::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal      => "Normal",
            Self::Transparent => "Transparent",
            Self::Overlay     => "Overlay",
        }
    }

    pub fn needs_layered(self) -> bool {
        matches!(self, Self::Transparent | Self::Overlay)
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn needs_click_through(self) -> bool {
        matches!(self, Self::Overlay)
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn needs_topmost(self) -> bool {
        matches!(self, Self::Overlay)
    }
}

/// Application state structure
pub struct AppState {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) gpu: Option<crate::graphics::GpuResources>,
    pub(crate) audio: Option<AudioHandler>,
    pub(crate) devices: Vec<cpal::Device>,
    pub(crate) uniforms: VisUniforms,
    pub(crate) current_plugin_index: usize,
    pub(crate) window_mode: WindowMode,
    pub(crate) transparency_level: u8,
    pub(crate) show_info: bool,
    pub(crate) info_timer: Option<Instant>,
    pub(crate) show_device_selection: bool,
    pub(crate) pending_device_index: Option<usize>,
    pub(crate) settings: AppSettings,
    pub(crate) egui_ctx: egui::Context,
    pub(crate) egui_raw_input: egui::RawInput,
    pub(crate) egui_pointer_pos: egui::Pos2,
    pub(crate) current_modifiers: winit::keyboard::ModifiersState,
    pub(crate) transition_time: f32,
    pub(crate) transition_active: bool,
    pub(crate) last_mode_switch: Instant,
    pub(crate) last_frame_time: Instant,
    pub(crate) enabled_plugin_cache: Vec<usize>,
    pub(crate) show_shader_browser: bool,
}

impl AppState {
    pub fn new(devices: Vec<cpal::Device>, settings: AppSettings) -> Self {
        let mut audio = None;
        let mut show_device_selection = false;

        if let Some(selected_name) = &settings.selected_device {
            if let Some(index) = devices.iter().position(|d| d.name().ok().as_ref() == Some(selected_name)) {
                if let Ok(audio_handler) = AudioHandler::new(devices[index].clone()) {
                    audio = Some(audio_handler);
                } else {
                    show_device_selection = true;
                }
            } else {
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
            enabled_plugin_cache: Vec::new(),
            show_shader_browser: false,
        }
    }
}
