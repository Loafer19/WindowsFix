//! Shader information and registry

/// Visual category of a shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderCategory {
    Spectrum,
    Waveform,
    Geometry3D,
    Abstract,
    Heatmap,
}

impl ShaderCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Spectrum   => "🎵 Spectrum",
            Self::Waveform   => "🌊 Waveform",
            Self::Geometry3D => "🔮 3D Effects",
            Self::Abstract   => "✨ Abstract",
            Self::Heatmap    => "🌡 Heatmap",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTier {
    Light,
    Medium,
    Heavy,
}

impl PerformanceTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Light  => "⚡ Light",
            Self::Medium => "⚖ Medium",
            Self::Heavy  => "🔥 Heavy",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderInfo {
    pub id:          &'static str,
    pub description: &'static str,
    pub category:    ShaderCategory,
    pub performance: PerformanceTier,
    pub is_waveform: bool,
}

pub const SHADER_REGISTRY: &[ShaderInfo] = &[
    ShaderInfo { id: "bars_3d",           description: "3-D spectrum bars rising from the bottom",                category: ShaderCategory::Spectrum,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "circular_spectrum", description: "Radial bars arranged in a circle, mirrored",             category: ShaderCategory::Spectrum,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "cubes_3d",          description: "Perspective grid of cubes that grow with the beat",      category: ShaderCategory::Geometry3D, performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "depth_wave_3d",     description: "Rippling wave surface in perspective space",             category: ShaderCategory::Abstract,   performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "energy_field",      description: "Electric plasma field that pulses with the beat",        category: ShaderCategory::Abstract,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "gradient_bars",     description: "Spectrum bars with a smooth colour gradient",            category: ShaderCategory::Spectrum,   performance: PerformanceTier::Light,  is_waveform: false },
    ShaderInfo { id: "heatmap",           description: "Cold-to-hot thermal colour gradient per frequency",      category: ShaderCategory::Heatmap,    performance: PerformanceTier::Light,  is_waveform: false },
    ShaderInfo { id: "kaleidoscope",      description: "Mirrored kaleidoscope pattern driven by bass energy",    category: ShaderCategory::Spectrum,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "mandala",           description: "Rotating mandala pattern with beat highlights",          category: ShaderCategory::Abstract,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "neon_pulse",        description: "Neon-glow waveform that pulses bright on every beat",    category: ShaderCategory::Waveform,   performance: PerformanceTier::Light,  is_waveform: true  },
    ShaderInfo { id: "oscilloscope",      description: "Classic X-Y oscilloscope waveform trace",               category: ShaderCategory::Waveform,   performance: PerformanceTier::Light,  is_waveform: true  },
    ShaderInfo { id: "plasma_sphere_3d",  description: "Animated plasma sphere with frequency-driven colours",  category: ShaderCategory::Geometry3D, performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "ripple",            description: "Concentric ripples that expand on each beat",           category: ShaderCategory::Abstract,   performance: PerformanceTier::Light,  is_waveform: false },
    ShaderInfo { id: "simple_bars",       description: "Clean minimal spectrum bars, fast and clear",           category: ShaderCategory::Spectrum,   performance: PerformanceTier::Light,  is_waveform: false },
    ShaderInfo { id: "spectrum",          description: "Simple full-width frequency spectrum bars",             category: ShaderCategory::Spectrum,   performance: PerformanceTier::Light,  is_waveform: false },
    ShaderInfo { id: "sphere_3d",         description: "Rotating sphere with surface deformed by audio",        category: ShaderCategory::Geometry3D, performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "terrain_3d",        description: "Procedural terrain that rises with the bass",           category: ShaderCategory::Geometry3D, performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "tunnel_3d",         description: "Infinite tunnel with walls pulsing to the beat",        category: ShaderCategory::Abstract,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "water_droplets_3d", description: "Droplets rippling across a water surface",             category: ShaderCategory::Geometry3D, performance: PerformanceTier::Heavy,  is_waveform: false },
    ShaderInfo { id: "wave_3d",           description: "Undulating wave mesh driven by audio frequencies",      category: ShaderCategory::Abstract,   performance: PerformanceTier::Medium, is_waveform: false },
    ShaderInfo { id: "waveform",          description: "Anti-aliased waveform line with subtle glow",           category: ShaderCategory::Waveform,   performance: PerformanceTier::Light,  is_waveform: true  },
    ShaderInfo { id: "waveform_glow",     description: "Multi-layer neon glow waveform with trailing history",  category: ShaderCategory::Waveform,   performance: PerformanceTier::Medium, is_waveform: true  },
    ShaderInfo { id: "waveform_history",  description: "Scrolling waveform history showing the last few seconds", category: ShaderCategory::Waveform, performance: PerformanceTier::Medium, is_waveform: true  },
];

pub fn shader_info(id: &str) -> Option<&'static ShaderInfo> {
    SHADER_REGISTRY.iter().find(|s| s.id == id)
}
