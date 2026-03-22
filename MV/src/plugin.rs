//! Plugin system for visualization shaders

use crate::error::{AppError, AppResult};

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

/// Represents a loaded visualization plugin
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub is_spectrum: bool,
    pub render_pipeline: wgpu::RenderPipeline,
    pub info: Option<&'static ShaderInfo>,
}

impl Plugin {
    /// Create a plugin from an embedded WGSL shader source string
    pub fn load_from_source(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        name: &str,
        source: &str,
        format: wgpu::TextureFormat,
    ) -> AppResult<Self> {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(name),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let info = shader_info(name);
        let is_waveform = info.map_or(false, |i| i.is_waveform);

        Ok(Self {
            name: name.to_string(),
            is_spectrum: !is_waveform,
            render_pipeline,
            info,
        })
    }
}

/// Load all visualization plugins from embedded shader sources.
pub fn load_plugins(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> AppResult<Vec<Plugin>> {
    let common      = include_str!("../shaders/common.wgsl");
    let common_hist = include_str!("../shaders/common_history.wgsl");

    // (name, preamble, specific_shader_source)
    let raw: &[(&str, &str, &str)] = &[
        ("bars_3d",           common,      include_str!("../shaders/spectrum/bars_3d.wgsl")),
        ("circular_spectrum", common,      include_str!("../shaders/spectrum/circular_spectrum.wgsl")),
        ("energy_field",      common,      include_str!("../shaders/abstract/energy_field.wgsl")),
        ("gradient_bars",     common,      include_str!("../shaders/spectrum/gradient_bars.wgsl")),
        ("heatmap",           common,      include_str!("../shaders/spectrum/heatmap.wgsl")),
        ("kaleidoscope",      common,      include_str!("../shaders/spectrum/kaleidoscope.wgsl")),
        ("mandala",           common,      include_str!("../shaders/abstract/mandala.wgsl")),
        ("neon_pulse",        common,      include_str!("../shaders/waveform/neon_pulse.wgsl")),
        ("oscilloscope",      common,      include_str!("../shaders/waveform/oscilloscope.wgsl")),
        ("ripple",            common,      include_str!("../shaders/abstract/ripple.wgsl")),
        ("simple_bars",       common,      include_str!("../shaders/spectrum/simple_bars.wgsl")),
        ("spectrum",          common,      include_str!("../shaders/spectrum/spectrum.wgsl")),
        ("wave_3d",           common,      include_str!("../shaders/abstract/wave_3d.wgsl")),
        ("waveform",          common,      include_str!("../shaders/waveform/waveform.wgsl")),
        ("cubes_3d",          common,      include_str!("../shaders/geometry_3d/cubes_3d.wgsl")),
        ("depth_wave_3d",     common,      include_str!("../shaders/abstract/depth_wave_3d.wgsl")),
        ("plasma_sphere_3d",  common,      include_str!("../shaders/abstract/plasma_sphere_3d.wgsl")),
        ("sphere_3d",         common,      include_str!("../shaders/geometry_3d/sphere_3d.wgsl")),
        ("terrain_3d",        common,      include_str!("../shaders/geometry_3d/terrain_3d.wgsl")),
        ("tunnel_3d",         common,      include_str!("../shaders/abstract/tunnel_3d.wgsl")),
        ("water_droplets_3d", common,      include_str!("../shaders/geometry_3d/water_droplets_3d.wgsl")),
        ("waveform_glow",     common_hist, include_str!("../shaders/waveform/waveform_glow.wgsl")),
        ("waveform_history",  common_hist, include_str!("../shaders/waveform/waveform_history.wgsl")),
    ];

    let mut plugins = Vec::with_capacity(raw.len());
    for &(name, preamble, specific) in raw {
        let src = format!("{}\n{}", preamble, specific);
        let plugin = Plugin::load_from_source(device, pipeline_layout, name, &src, format)
            .map_err(|e| AppError::Plugin(format!("Failed to load shader '{}': {}", name, e)))?;
        plugins.push(plugin);
    }

    if plugins.is_empty() {
        return Err(AppError::Plugin("No visualization plugins loaded".to_string()));
    }

    Ok(plugins)
}
