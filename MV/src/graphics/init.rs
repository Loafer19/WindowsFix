//! GPU initialization functions

use crate::common::error::AppResult;
use crate::common::types::{Particle, VisUniforms};
use crate::config::constants::*;
use std::mem;
use wgpu::util::DeviceExt;

/// Buffer manager for GPU resources
pub struct BufferManager {
    pub uniform_buffer: wgpu::Buffer,
    pub fft_buffer: wgpu::Buffer,
    pub history_buffer: wgpu::Buffer,
    pub particle_buffer: wgpu::Buffer,
    pub quad_buffer: wgpu::Buffer,
}

impl BufferManager {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let uniform_buffer = Self::create_uniform_buffer(device, width, height);
        let fft_buffer = Self::create_fft_buffer(device);
        let history_buffer = Self::create_history_buffer(device);
        let (particle_buffer, quad_buffer) = Self::create_particle_buffers(device);

        Self {
            uniform_buffer,
            fft_buffer,
            history_buffer,
            particle_buffer,
            quad_buffer,
        }
    }

    pub fn create_uniform_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
        let uniforms = VisUniforms {
            color: DEFAULT_COLOR,
            intensity: DEFAULT_INTENSITY,
            padding1: 0.0,
            resolution: [width as f32, height as f32],
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
        };
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn create_fft_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let fft_data = vec![0.0f32; SAMPLE_SIZE];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FFT Buffer"),
            contents: bytemuck::cast_slice(&fft_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn create_history_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let history_data = vec![0.0f32; WAVEFORM_HISTORY_SIZE * SAMPLE_SIZE];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("History Buffer"),
            contents: bytemuck::cast_slice(&history_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn create_particle_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let initial_particles: Vec<Particle> = (0..NUM_PARTICLES)
            .map(|_| Particle {
                position: [0.0, -1.0],
                velocity: [0.0, 0.0],
                lifetime: 0.0,
                padding: [0.0; 3],
                color: DEFAULT_COLOR,
            })
            .collect();

        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer"),
            contents: bytemuck::cast_slice(&initial_particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let quad_data: [[f32; 2]; 4] = [
            [-0.01, -0.01], [0.01, -0.01], [0.01, 0.01], [-0.01, 0.01],
        ];
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(&quad_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        (particle_buffer, quad_buffer)
    }
}

pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("bind_group_layout"),
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    fft_buffer: &wgpu::Buffer,
    history_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: fft_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: history_buffer.as_entire_binding() },
        ],
        label: Some("bind_group"),
    })
}

pub fn create_particle_system(
    device: &wgpu::Device,
    fft_buffer: &wgpu::Buffer,
    uniform_buffer: &wgpu::Buffer,
    particle_buffer: &wgpu::Buffer,
    quad_buffer: &wgpu::Buffer,
    render_pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> AppResult<(wgpu::BindGroup, wgpu::ComputePipeline, wgpu::RenderPipeline)> {

    let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
        ],
        label: Some("compute_bind_group_layout"),
    });

    let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: particle_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: fft_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() },
        ],
        label: Some("particle_bind_group"),
    });

    let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/compute_particles.wgsl").into()),
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
        entry_point: "main",
    });

    let particle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Particle Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/particle_render.wgsl").into()),
    });

    let particle_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Particle Render Pipeline"),
        layout: Some(render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &particle_shader,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Particle>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                        wgpu::VertexAttribute { offset: 32, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                    ],
                },
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &particle_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
        multiview: None,
    });

    Ok((particle_bind_group, compute_pipeline, particle_render_pipeline))
}
