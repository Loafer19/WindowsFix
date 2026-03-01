//! Plugin system for visualization shaders

use crate::error::{AppError, AppResult};

/// Represents a loaded visualization plugin
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub is_spectrum: bool,
    pub render_pipeline: wgpu::RenderPipeline,
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

        // Plugins that use raw waveform samples (not FFT spectrum).
        // NOTE: if you add a new waveform-based shader to load_plugins(), also
        // add its name to this list, otherwise is_spectrum will be wrongly true.
        let is_waveform = matches!(name, "waveform" | "waveform_glow" | "waveform_history" | "oscilloscope");

        Ok(Self {
            name: name.to_string(),
            is_spectrum: !is_waveform,
            render_pipeline,
        })
    }
}

/// Load all visualization plugins from embedded shader sources.
///
/// Shaders are compiled into the binary via `include_str!` so the app works
/// even when the `shaders/` directory is not present alongside the executable
/// (e.g. in GitHub release builds).
pub fn load_plugins(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> AppResult<Vec<Plugin>> {
    let shader_sources: &[(&str, &str)] = &[
        ("bars_3d",           include_str!("../shaders/bars_3d.wgsl")),
        ("circular_spectrum", include_str!("../shaders/circular_spectrum.wgsl")),
        ("cubes_3d",          include_str!("../shaders/cubes_3d.wgsl")),
        ("depth_wave_3d",     include_str!("../shaders/depth_wave_3d.wgsl")),
        ("gradient_bars",     include_str!("../shaders/gradient_bars.wgsl")),
        ("mandala",           include_str!("../shaders/mandala.wgsl")),
        ("oscilloscope",      include_str!("../shaders/oscilloscope.wgsl")),
        ("plasma_sphere_3d",  include_str!("../shaders/plasma_sphere_3d.wgsl")),
        ("ripple",            include_str!("../shaders/ripple.wgsl")),
        ("spectrum",          include_str!("../shaders/spectrum.wgsl")),
        ("sphere_3d",         include_str!("../shaders/sphere_3d.wgsl")),
        ("terrain_3d",        include_str!("../shaders/terrain_3d.wgsl")),
        ("tunnel_3d",         include_str!("../shaders/tunnel_3d.wgsl")),
        ("wave_3d",           include_str!("../shaders/wave_3d.wgsl")),
        ("waveform",          include_str!("../shaders/waveform.wgsl")),
        ("waveform_glow",     include_str!("../shaders/waveform_glow.wgsl")),
        ("waveform_history",  include_str!("../shaders/waveform_history.wgsl")),
    ];

    let mut plugins = Vec::with_capacity(shader_sources.len());
    for (name, source) in shader_sources {
        let plugin = Plugin::load_from_source(device, pipeline_layout, name, source, format)
            .map_err(|e| AppError::Plugin(format!("Failed to load shader '{}': {}", name, e)))?;
        plugins.push(plugin);
    }

    if plugins.is_empty() {
        return Err(AppError::Plugin("No visualization plugins loaded".to_string()));
    }

    Ok(plugins)
}
