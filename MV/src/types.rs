//! Shared data types for GPU buffers

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
    pub time: f32,
    pub bass_energy: f32,
    pub smoothing_factor: f32,
    pub gain: f32,
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
