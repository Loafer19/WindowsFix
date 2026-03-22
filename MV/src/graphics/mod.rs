//! Graphics and GPU management

pub mod init;
pub mod resources;
pub mod update;
pub mod render;

// Re-export the main GPU resources
pub use resources::GpuResources;
