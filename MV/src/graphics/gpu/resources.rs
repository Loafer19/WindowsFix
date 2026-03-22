//! GPU resources and state

use crate::common::error::{AppError, AppResult};
use crate::graphics::buffers::BufferManager;
use crate::visualization::Plugin;
use std::time::Instant;

/// GPU resources and state
pub struct GpuResources {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub buffers: BufferManager,
    pub particle_bind_group: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub particle_render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub plugins: Vec<Plugin>,
    pub egui_renderer: egui_wgpu::Renderer,
    pub(crate) start_time: Instant,
    pub(crate) smoothed_fft: Vec<f32>,
    pub bass_energy: f32,
    pub(crate) waveform_history: Vec<f32>,
    pub(crate) history_frame_counter: u32,
    /// Rolling energy history for beat detection (ring-buffer, newest at index 0).
    pub(crate) energy_history: Vec<f32>,
    /// Instantaneous beat intensity that peaks on beat and decays each frame.
    pub beat_intensity: f32,
    /// Pre-allocated complex buffer reused every FFT call to avoid per-frame heap allocation.
    pub(crate) fft_complex_buf: Vec<rustfft::num_complex::Complex<f32>>,
    /// Cached FFT planner; plans are memoized internally so reusing the planner
    /// avoids redundant algorithm selection on every call.
    pub(crate) fft_planner: rustfft::FftPlanner<f32>,
}

impl GpuResources {
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

        let buffers = BufferManager::new(&device, size.width, size.height);
        let bind_group_layout = super::init::create_bind_group_layout(&device);
        let bind_group = super::init::create_bind_group(&device, &bind_group_layout, &buffers.uniform_buffer, &buffers.fft_buffer, &buffers.history_buffer);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let plugins = crate::visualization::load_plugins(&device, &render_pipeline_layout, config.format)?;

        let (particle_bind_group, compute_pipeline, particle_render_pipeline) =
            super::init::create_particle_system(&device, &buffers.fft_buffer, &buffers.uniform_buffer, &buffers.particle_buffer, &buffers.quad_buffer, &render_pipeline_layout, config.format)?;

        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            buffers,
            particle_bind_group,
            compute_pipeline,
            particle_render_pipeline,
            bind_group,
            plugins,
            egui_renderer,
            start_time: Instant::now(),
            smoothed_fft: vec![0.0f32; crate::config::constants::SAMPLE_SIZE / 2],
            bass_energy: 0.0,
            waveform_history: vec![0.0f32; crate::config::constants::WAVEFORM_HISTORY_SIZE * crate::config::constants::SAMPLE_SIZE],
            history_frame_counter: 0,
            energy_history: vec![0.0f32; crate::config::constants::BEAT_HISTORY_SIZE],
            beat_intensity: 0.0,
            fft_complex_buf: vec![rustfft::num_complex::Complex::new(0.0, 0.0); crate::config::constants::SAMPLE_SIZE],
            fft_planner: rustfft::FftPlanner::new(),
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
