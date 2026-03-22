//! Application settings

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorScheme {
    Classic,
    Neon,
    Pastel,
    Fire,
}

/// Controls how aggressively the beat detector fires.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Selected audio input device name.
    pub selected_device: Option<String>,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            show_settings: false,
            transparency: crate::config::constants::DEFAULT_TRANSPARENCY as f32 / 255.0,
            auto_switch_modes: false,
            mode_switch_seconds: 30.0,
            smoothing_factor: 0.1,
            gain: 1.5,
            color_scheme: ColorScheme::Classic,
            bass_boost: 1.0,
            disabled_plugins: HashSet::new(),
            beat_sensitivity: BeatSensitivity::Medium,
            selected_device: None,
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write("settings.json", json)?;
        Ok(())
    }

    pub fn load() -> Self {
        if Path::new("settings.json").exists() {
            if let Ok(json) = fs::read_to_string("settings.json") {
                match serde_json::from_str::<Self>(&json) {
                    Ok(mut s) => {
                        // Clamp all numeric fields to valid ranges so a corrupted
                        // or hand-edited settings file cannot cause panics or
                        // unexpected behaviour at runtime.
                        s.transparency = s.transparency.clamp(0.1, 1.0);
                        s.gain = s.gain.clamp(0.5, 5.0);
                        s.smoothing_factor = s.smoothing_factor.clamp(0.01, 0.3);
                        s.bass_boost = s.bass_boost.clamp(0.0, 2.0);
                        s.mode_switch_seconds = s.mode_switch_seconds.clamp(5.0, 120.0);
                        return s;
                    }
                    Err(e) => eprintln!("Warning: failed to parse settings.json: {e}. Using defaults."),
                }
            }
        }
        Self::new()
    }
}

impl Default for AppSettings {
    fn default() -> Self { Self::new() }
}
