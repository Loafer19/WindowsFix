//! GPU buffer management

use crate::config::constants::*;
use crate::common::types::{Particle, VisUniforms};
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
