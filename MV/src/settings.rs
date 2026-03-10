//! Application settings

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    Classic,
    Neon,
    Pastel,
    Fire,
}

/// Controls how aggressively the beat detector fires.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeatSensitivity {
    /// Fires only on strong transients (threshold ×1.8 above rolling mean).
    Low,
    /// Balanced default (threshold ×1.5 above rolling mean).
    Medium,
    /// Fires on softer transients as well (threshold ×1.3 above rolling mean).
    High,
}

impl BeatSensitivity {
    /// Cycle to the next level: Low → Medium → High → Low.
    pub fn next(self) -> Self {
        match self {
            Self::Low    => Self::Medium,
            Self::Medium => Self::High,
            Self::High   => Self::Low,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low    => "Low",
            Self::Medium => "Medium",
            Self::High   => "High",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub show_settings: bool,
    /// Window opacity used by the transparency slider (0.1 = nearly transparent, 1.0 = opaque).
    pub transparency: f32,
    pub auto_switch_modes: bool,
    pub mode_switch_seconds: f32,
    pub smoothing_factor: f32,
    pub gain: f32,
    pub color_scheme: ColorScheme,
    pub bass_boost: f32,
    /// Names of visualization plugins that the user has disabled.
    pub disabled_plugins: HashSet<String>,
    /// Beat detection sensitivity level.
    pub beat_sensitivity: BeatSensitivity,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            show_settings: false,
            transparency: crate::constants::DEFAULT_TRANSPARENCY as f32 / 255.0,
            auto_switch_modes: false,
            mode_switch_seconds: 30.0,
            smoothing_factor: 0.1,
            gain: 1.5,
            color_scheme: ColorScheme::Classic,
            bass_boost: 1.0,
            disabled_plugins: HashSet::new(),
            beat_sensitivity: BeatSensitivity::Medium,
        }
    }

    pub fn scheme_color(&self) -> [f32; 4] {
        match self.color_scheme {
            ColorScheme::Classic => [1.0, 1.0, 1.0, 1.0],
            ColorScheme::Neon    => [0.0, 1.0, 0.8, 1.0],
            ColorScheme::Pastel  => [0.8, 0.7, 1.0, 1.0],
            ColorScheme::Fire    => [1.0, 0.4, 0.0, 1.0],
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self { Self::new() }
}
