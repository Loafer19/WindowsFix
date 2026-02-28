//! Constants used throughout the application

/// Default window size
pub const DEFAULT_WINDOW_WIDTH: u32 = 800;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 600;

/// Audio buffer size
pub const SAMPLE_SIZE: usize = 512;

/// Number of particles for particle system
pub const NUM_PARTICLES: u32 = 1000;

/// Workgroup size for compute shader
pub const COMPUTE_WORKGROUP_SIZE: u32 = 64;

/// Shader directory name
pub const SHADER_DIR: &str = "shaders";

/// Particle render shader filename
pub const PARTICLE_RENDER_SHADER: &str = "particle_render.wgsl";

/// Compute particles shader filename
pub const COMPUTE_PARTICLES_SHADER: &str = "compute_particles.wgsl";

/// Window title
pub const WINDOW_TITLE: &str = "Music Visualizer";

/// Default color values
pub const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const DEFAULT_INTENSITY: f32 = 1.0;

/// Intensity adjustment step
pub const INTENSITY_STEP: f32 = 0.1;

/// Number of FFT bins considered "bass" for energy computation.
///
/// With a 512-sample FFT at 44 100 Hz the bin resolution is ~86 Hz/bin,
/// so 6 bins cover roughly 0–516 Hz — a generous low-frequency range that
/// gives a perceptually useful "punch" value across common sample rates
/// (44 100 / 48 000 Hz).  Increase this value for a broader low-end window.
pub const BASS_BIN_COUNT: usize = 6;

/// Default transparency level (150/255 ≈ 59%)
pub const DEFAULT_TRANSPARENCY: u8 = 150;

/// Transparency adjustment step (~10%)
pub const TRANSPARENCY_STEP: u8 = 25;
