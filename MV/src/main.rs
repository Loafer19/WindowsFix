//! Music Visualizer - Main entry point

mod app;
mod audio;
mod constants;
mod error;
mod gpu;
mod plugin;
mod settings;
mod types;

use crate::app::App;
use crate::error::{AppError, AppResult};
use crate::settings::AppSettings;
use winit::event_loop::EventLoop;
use cpal::traits::{DeviceTrait, HostTrait};

fn main() -> AppResult<()> {
    // Load settings
    let settings = AppSettings::load();

    // Get audio input devices
    let host = cpal::default_host();
    let all_devices: Vec<_> = host.input_devices().map_err(|e| AppError::Audio(format!("Failed to get input devices: {}", e)))?.collect();

    // Filter devices that support input configs
    let devices: Vec<_> = all_devices.into_iter()
        .filter(|device| device.supported_input_configs().map_or(false, |mut iter| iter.next().is_some()))
        .collect();

    if devices.is_empty() {
        return Err(AppError::Audio("No input devices available".to_string()));
    }

    // Create event loop and app
    let event_loop = EventLoop::builder().build().map_err(|e| AppError::Config(format!("Failed to create event loop: {:?}", e)))?;
    let mut app = App::new(devices, settings);

    // Run the application
    event_loop.run_app(&mut app).map_err(|e| AppError::Config(format!("Failed to run app: {:?}", e)))?;

    Ok(())
}
