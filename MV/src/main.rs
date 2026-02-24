//! Music Visualizer - Main entry point

mod app;
mod audio;
mod constants;
mod error;
mod gpu;
mod plugin;
mod types;

use crate::audio::AudioHandler;
use crate::app::App;
use crate::error::{AppError, AppResult};
use winit::event_loop::EventLoop;
use cpal::traits::{DeviceTrait, HostTrait};
use std::io::{self, Write};

fn main() -> AppResult<()> {
    // Select audio input device
    let host = cpal::default_host();
    let all_devices: Vec<_> = host.input_devices().map_err(|e| AppError::Audio(format!("Failed to get input devices: {}", e)))?.collect();

    // Filter devices that support input configs
    let devices: Vec<_> = all_devices.into_iter()
        .filter(|device| device.supported_input_configs().map_or(false, |mut iter| iter.next().is_some()))
        .collect();

    if devices.is_empty() {
        return Err(AppError::Audio("No input devices available".to_string()));
    }

    println!("Available audio input devices:");
    for (i, device) in devices.iter().enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        println!("{}: {}", i + 1, name);
    }

    print!("Select device (1-{}): ", devices.len());
    io::stdout().flush().map_err(|e| AppError::Audio(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| AppError::Audio(format!("Failed to read input: {}", e)))?;
    let selection: usize = input.trim().parse().map_err(|_| AppError::Audio("Invalid input, expected a number".to_string()))?;

    if selection < 1 || selection > devices.len() {
        return Err(AppError::Audio(format!("Invalid selection, must be between 1 and {}", devices.len())));
    }

    let selected_device = devices[selection - 1].clone();

    // Initialize audio handler
    let audio_handler = AudioHandler::new(selected_device)?;

    // Create event loop and app
    let event_loop = EventLoop::builder().build().map_err(|e| AppError::Config(format!("Failed to create event loop: {:?}", e)))?;
    let mut app = App::new(audio_handler);

    // Run the application
    event_loop.run_app(&mut app).map_err(|e| AppError::Config(format!("Failed to run app: {:?}", e)))?;

    Ok(())
}
