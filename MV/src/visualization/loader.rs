//! Plugin loading functionality

use crate::common::error::{AppError, AppResult};
use super::plugin::Plugin;

/// Load all visualization plugins from embedded shader sources.
pub fn load_plugins(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> AppResult<Vec<Plugin>> {
    let common      = include_str!("../../shaders/common.wgsl");
    let common_hist = include_str!("../../shaders/common_history.wgsl");

    // (name, preamble, specific_shader_source)
    let raw: &[(&str, &str, &str)] = &[
        ("bars_3d",           common,      include_str!("../../shaders/spectrum/bars_3d.wgsl")),
        ("circular_spectrum", common,      include_str!("../../shaders/spectrum/circular_spectrum.wgsl")),
        ("energy_field",      common,      include_str!("../../shaders/abstract/energy_field.wgsl")),
        ("gradient_bars",     common,      include_str!("../../shaders/spectrum/gradient_bars.wgsl")),
        ("heatmap",           common,      include_str!("../../shaders/spectrum/heatmap.wgsl")),
        ("kaleidoscope",      common,      include_str!("../../shaders/spectrum/kaleidoscope.wgsl")),
        ("mandala",           common,      include_str!("../../shaders/abstract/mandala.wgsl")),
        ("neon_pulse",        common,      include_str!("../../shaders/waveform/neon_pulse.wgsl")),
        ("oscilloscope",      common,      include_str!("../../shaders/waveform/oscilloscope.wgsl")),
        ("ripple",            common,      include_str!("../../shaders/abstract/ripple.wgsl")),
        ("simple_bars",       common,      include_str!("../../shaders/spectrum/simple_bars.wgsl")),
        ("spectrum",          common,      include_str!("../../shaders/spectrum/spectrum.wgsl")),
        ("wave_3d",           common,      include_str!("../../shaders/abstract/wave_3d.wgsl")),
        ("waveform",          common,      include_str!("../../shaders/waveform/waveform.wgsl")),
        ("cubes_3d",          common,      include_str!("../../shaders/geometry_3d/cubes_3d.wgsl")),
        ("depth_wave_3d",     common,      include_str!("../../shaders/abstract/depth_wave_3d.wgsl")),
        ("plasma_sphere_3d",  common,      include_str!("../../shaders/abstract/plasma_sphere_3d.wgsl")),
        ("sphere_3d",         common,      include_str!("../../shaders/geometry_3d/sphere_3d.wgsl")),
        ("terrain_3d",        common,      include_str!("../../shaders/geometry_3d/terrain_3d.wgsl")),
        ("tunnel_3d",         common,      include_str!("../../shaders/abstract/tunnel_3d.wgsl")),
        ("water_droplets_3d", common,      include_str!("../../shaders/geometry_3d/water_droplets_3d.wgsl")),
        ("waveform_glow",     common_hist, include_str!("../../shaders/waveform/waveform_glow.wgsl")),
        ("waveform_history",  common_hist, include_str!("../../shaders/waveform/waveform_history.wgsl")),
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
