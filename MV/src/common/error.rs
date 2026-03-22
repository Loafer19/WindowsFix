//! Error types for the music visualizer application

use std::io;

/// Custom error type for the application
#[derive(Debug)]
pub enum AppError {
    /// GPU-related errors
    Gpu(wgpu::Error),
    /// Surface errors
    Surface(wgpu::SurfaceError),
    /// I/O errors
    Io(io::Error),
    /// Audio errors
    Audio(String),
    /// Plugin loading errors
    Plugin(String),
    /// Configuration errors
    Config(String),
    /// Surface creation errors
    SurfaceCreate(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Gpu(e) => write!(f, "GPU error: {}", e),
            AppError::Surface(e) => write!(f, "Surface error: {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Audio(msg) => write!(f, "Audio error: {}", msg),
            AppError::Plugin(msg) => write!(f, "Plugin error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::SurfaceCreate(msg) => write!(f, "Surface creation error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<wgpu::Error> for AppError {
    fn from(error: wgpu::Error) -> Self {
        AppError::Gpu(error)
    }
}

impl From<wgpu::SurfaceError> for AppError {
    fn from(error: wgpu::SurfaceError) -> Self {
        AppError::Surface(error)
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::Io(error)
    }
}

impl From<cpal::BuildStreamError> for AppError {
    fn from(error: cpal::BuildStreamError) -> Self {
        AppError::Audio(format!("Build stream error: {}", error))
    }
}

impl From<cpal::PlayStreamError> for AppError {
    fn from(error: cpal::PlayStreamError) -> Self {
        AppError::Audio(format!("Play stream error: {}", error))
    }
}

impl From<cpal::DefaultStreamConfigError> for AppError {
    fn from(error: cpal::DefaultStreamConfigError) -> Self {
        AppError::Audio(format!("Default stream config error: {}", error))
    }
}

impl From<cpal::SupportedStreamConfigsError> for AppError {
    fn from(error: cpal::SupportedStreamConfigsError) -> Self {
        AppError::Audio(format!("Supported stream configs error: {}", error))
    }
}

impl From<wgpu::CreateSurfaceError> for AppError {
    fn from(error: wgpu::CreateSurfaceError) -> Self {
        AppError::SurfaceCreate(format!("Create surface error: {}", error))
    }
}

impl From<wgpu::RequestDeviceError> for AppError {
    fn from(error: wgpu::RequestDeviceError) -> Self {
        AppError::Config(format!("Request device error: {}", error))
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
