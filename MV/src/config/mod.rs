//! Configuration management

pub mod settings;
pub mod constants;
pub mod colors;
pub mod persistence;

// Re-export main config functionality
pub use settings::*;
pub use constants::*;
