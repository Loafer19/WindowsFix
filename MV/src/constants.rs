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

/// Minimum alpha value for the transparent-mode window (10% opaque).
/// Prevents the window from becoming completely invisible.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const MIN_TRANSPARENCY_ALPHA: u8 = 25;

/// Maximum alpha value – fully opaque (255/255 = 100%).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const MAX_TRANSPARENCY_ALPHA: u8 = 255;

/// Number of waveform history snapshots to keep.
pub const WAVEFORM_HISTORY_SIZE: usize = 64;

/// Update the waveform history every N rendered frames (~15 fps of history at 60 fps).
pub const HISTORY_UPDATE_INTERVAL: u32 = 4;

// ─── Beat Detection ───────────────────────────────────────────────────────────

/// Number of frames kept in the rolling energy history used for beat detection.
/// At ~60 fps this covers approximately 0.7 seconds.
pub const BEAT_HISTORY_SIZE: usize = 43;

/// Energy ratio thresholds above the rolling mean that trigger a beat.
/// Low  = more sensitive (fires on softer transients).
/// High = less sensitive (only fires on strong transients).
pub const BEAT_THRESHOLD_LOW: f32 = 1.3;
pub const BEAT_THRESHOLD_MED: f32 = 1.5;
pub const BEAT_THRESHOLD_HIGH: f32 = 1.8;

/// How fast `beat_intensity` decays toward 0 each frame (multiplicative).
pub const BEAT_DECAY: f32 = 0.85;
