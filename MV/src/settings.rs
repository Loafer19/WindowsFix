//! Application settings

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    Classic,
    Neon,
    Pastel,
    Fire,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub show_settings: bool,
    pub transparency: f32,
    pub auto_switch_modes: bool,
    pub mode_switch_seconds: f32,
    pub smoothing_factor: f32,
    pub gain: f32,
    pub color_scheme: ColorScheme,
    pub bass_boost: f32,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            show_settings: false,
            transparency: 1.0,
            auto_switch_modes: false,
            mode_switch_seconds: 30.0,
            smoothing_factor: 0.1,
            gain: 1.5,
            color_scheme: ColorScheme::Classic,
            bass_boost: 1.0,
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
