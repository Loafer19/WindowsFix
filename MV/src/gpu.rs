//! GPU resource management and initialization

use crate::constants::*;
use crate::error::{AppError, AppResult};
use crate::plugin::Plugin;
use rustfft::{FftPlanner, num_complex::Complex};
use std::mem;
use wgpu::util::DeviceExt;

/// GPU resources and state
#[derive(Debug)]
pub struct GpuResources {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub uniform_buffer: wgpu::Buffer,
    pub fft_buffer: wgpu::Buffer,
    pub particle_buffer: wgpu::Buffer,
    pub quad_buffer: wgpu::Buffer,
    pub particle_bind_group: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub particle_render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub plugins: Vec<Plugin>,
}

impl GpuResources {
    /// Initialize GPU resources asynchronously
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> AppResult<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| AppError::Config("No suitable adapter found".to_string()))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let uniform_buffer = Self::create_uniform_buffer(&device, size.width, size.height);
        let fft_buffer = Self::create_fft_buffer(&device);
        let bind_group_layout = Self::create_bind_group_layout(&device);
        let bind_group = Self::create_bind_group(&device, &bind_group_layout, &uniform_buffer, &fft_buffer);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let plugins = crate::plugin::load_plugins(&device, &render_pipeline_layout, config.format)?;

        let (particle_buffer, quad_buffer, particle_bind_group, compute_pipeline, particle_render_pipeline) =
            Self::create_particle_system(&device, &fft_buffer, &uniform_buffer, &render_pipeline_layout, config.format)?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            uniform_buffer,
            fft_buffer,
            particle_buffer,
            quad_buffer,
            particle_bind_group,
            compute_pipeline,
            particle_render_pipeline,
            bind_group,
            plugins,
        })
    }

    /// Create uniform buffer with initial values
    fn create_uniform_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
        let uniforms = VisUniforms {
            color: DEFAULT_COLOR,
            intensity: DEFAULT_INTENSITY,
            padding1: 0.0,
            resolution: [width as f32, height as f32],
            mode: MODE_SPECTRUM,
            padding2: [0; 3],
            padding3: [0; 4],
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// Create FFT buffer for audio data
    fn create_fft_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let fft_data = vec![0.0f32; SAMPLE_SIZE];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FFT Buffer"),
            contents: bytemuck::cast_slice(&fft_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// Create bind group layout for shaders
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
            ],
            label: Some("bind_group_layout"),
        })
    }

    /// Create bind group for shaders
    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
        fft_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fft_buffer.as_entire_binding(),
                },
            ],
            label: Some("bind_group"),
        })
    }

    /// Create particle system resources
    fn create_particle_system(
        device: &wgpu::Device,
        fft_buffer: &wgpu::Buffer,
        uniform_buffer: &wgpu::Buffer,
        render_pipeline_layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
    ) -> AppResult<(wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup, wgpu::ComputePipeline, wgpu::RenderPipeline)> {
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
            [-0.01, -0.01],
            [0.01, -0.01],
            [0.01, 0.01],
            [-0.01, 0.01],
        ];
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(&quad_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("compute_bind_group_layout"),
        });

        let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fft_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("particle_bind_group"),
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/compute_particles.wgsl").into()),
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
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let particle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/particle_render.wgsl").into()),
        });

        let particle_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &particle_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &particle_shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok((particle_buffer, quad_buffer, particle_bind_group, compute_pipeline, particle_render_pipeline))
    }

    /// Resize the surface
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Update uniforms and process audio data
    pub fn update(&mut self, uniforms: &VisUniforms, audio_data: &[f32]) {
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

        let data_to_write: Vec<f32>;
        if self.plugins[uniforms.mode as usize].is_spectrum {
            let mut magnitudes = self.compute_fft(audio_data);
            let len = magnitudes.len() as f32;
            for m in &mut magnitudes {
                *m /= len;
                *m = (*m * 50.0 * uniforms.intensity).min(1.0);
            }
            data_to_write = magnitudes;
        } else {
            data_to_write = audio_data.to_vec();
        }
        self.queue.write_buffer(&self.fft_buffer, 0, bytemuck::cast_slice(&data_to_write));
    }

    /// Compute FFT magnitudes from audio data
    fn compute_fft(&self, audio_data: &[f32]) -> Vec<f32> {
        let mut buffer: Vec<Complex<f32>> = audio_data.iter().map(|&x| Complex::new(x, 0.0)).collect();
        let fft = FftPlanner::new().plan_fft_forward(audio_data.len());
        fft.process(&mut buffer);
        buffer[0..audio_data.len() / 2]
            .iter()
            .map(|c| c.norm())
            .collect()
    }

    /// Render a frame
    pub fn render(&mut self, plugin_index: usize) -> AppResult<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Compute particles
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.particle_bind_group, &[]);
            cpass.dispatch_workgroups(NUM_PARTICLES / COMPUTE_WORKGROUP_SIZE + 1, 1, 1);
        }

        // Render
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw visualization using current plugin
            let plugin = &self.plugins[plugin_index];
            rpass.set_pipeline(&plugin.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..3, 0..1); // Simple triangle for full-screen quad

                // Draw particles (overlay)
                rpass.set_pipeline(&self.particle_render_pipeline);
                rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.particle_buffer.slice(..));
                rpass.draw(0..4, 0..NUM_PARTICLES);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

/// Uniforms for visualization shaders
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VisUniforms {
    pub color: [f32; 4],
    pub intensity: f32,
    pub padding1: f32,
    pub resolution: [f32; 2],
    pub mode: u32,
    pub padding2: [u32; 3],
    pub padding3: [u32; 4],
}

/// Particle structure for GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub lifetime: f32,
    pub padding: [f32; 3],
    pub color: [f32; 4],
}
