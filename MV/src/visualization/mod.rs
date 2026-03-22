//! Visualization plugins and shaders

pub mod plugin;
pub mod loader;
pub mod shader_info;

// Re-export types and functions
pub use plugin::Plugin;
pub use loader::load_plugins;
pub use shader_info::{ShaderCategory, PerformanceTier, ShaderInfo, SHADER_REGISTRY, shader_info};
