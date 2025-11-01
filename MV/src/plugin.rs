//! Plugin system for visualization shaders

use crate::constants::{SHADER_DIR, COMPUTE_PARTICLES_SHADER, PARTICLE_RENDER_SHADER};
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::Path;

/// Represents a loaded visualization plugin
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub is_spectrum: bool,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Plugin {
    /// Load a plugin from a WGSL shader file
    pub fn load_from_file(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        name: &str,
        path: &str,
        format: wgpu::TextureFormat,
    ) -> AppResult<Self> {
        let source = fs::read_to_string(path)
            .map_err(|e| AppError::Plugin(format!("Failed to read shader file {}: {}", path, e)))?;

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
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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

        Ok(Self {
            name: name.to_string(),
            is_spectrum: !name.contains("waveform"),
            render_pipeline,
        })
    }
}

/// Load all visualization plugins from the shaders directory (excluding internal shaders)
pub fn load_plugins(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> AppResult<Vec<Plugin>> {
    let shader_dir = Path::new(SHADER_DIR);
    if !shader_dir.exists() {
        return Err(AppError::Plugin(format!("Shader directory '{}' does not exist", SHADER_DIR)));
    }

    let mut plugins = Vec::new();

    // Internal shaders that should not be loaded as visualization plugins
    let internal_shaders = [COMPUTE_PARTICLES_SHADER, PARTICLE_RENDER_SHADER];

    for entry in fs::read_dir(shader_dir)
        .map_err(|e| AppError::Plugin(format!("Failed to read shader directory: {}", e)))?
    {
        let entry = entry.map_err(|e| AppError::Plugin(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            if extension == "wgsl" {
                let filename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| AppError::Plugin("Invalid shader filename".to_string()))?;

                // Skip internal shaders (compute and particle rendering)
                if internal_shaders.contains(&filename) {
                    continue;
                }

                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| AppError::Plugin("Invalid shader filename".to_string()))?
                    .to_string();

                let path_str = path.to_string_lossy().to_string();
                let plugin = Plugin::load_from_file(device, pipeline_layout, &name, &path_str, format)?;
                plugins.push(plugin);
            }
        }
    }

    if plugins.is_empty() {
        return Err(AppError::Plugin("No visualization plugins found in shader directory".to_string()));
    }

    Ok(plugins)
}
