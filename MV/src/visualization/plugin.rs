//! Plugin structure and loading

use crate::common::error::{AppError, AppResult};
use super::shader_info::{ShaderInfo, shader_info};

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
