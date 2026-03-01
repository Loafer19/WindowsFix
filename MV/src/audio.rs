//! Audio input handling using cpal

use crate::constants::SAMPLE_SIZE;
use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Audio handler for capturing input from a selected device
pub struct AudioHandler {
    pub buffer: Arc<Mutex<Vec<f32>>>,
    _input_stream: cpal::Stream,
}

impl AudioHandler {
    /// Create a new audio handler with specified input device
    pub fn new(device: cpal::Device) -> AppResult<Self> {

        // Try to find a supported input config
        let supported_configs = device
            .supported_input_configs()
            .map_err(|e| AppError::Audio(format!("Failed to get supported configs: {}", e)))?;

        let config = supported_configs
            .filter(|config| config.channels() == 1 || config.channels() == 2)
            .find_map(|config| {
                // Prefer 44100 Hz, but accept others
                let sample_rate = if config.min_sample_rate().0 <= 44100 && config.max_sample_rate().0 >= 44100 {
                    cpal::SampleRate(44100)
                } else {
                    config.min_sample_rate()
                };
                Some(config.with_sample_rate(sample_rate))
            })
            .ok_or_else(|| AppError::Audio("No suitable input config found for device".to_string()))?;

        let channels = config.channels();
        let sample_format = config.sample_format();
        let stream_config = config.into();
        let buffer = Arc::new(Mutex::new(vec![0.0; SAMPLE_SIZE]));

        let err_fn = |err| eprintln!("Audio error: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        mix_to_mono(data, &mut buf, channels);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U8 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u8], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 128.0) / 128.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(AppError::Audio(format!("Unsupported sample format: {:?}", sample_format))),
        }?;

        stream.play().map_err(|e| AppError::Audio(format!("Failed to start input stream: {}", e)))?;

        Ok(Self {
            buffer,
            _input_stream: stream,
        })
    }
}

/// Mix multi-channel interleaved samples down to mono and write into `out`.
/// `channels` is the number of channels per frame.
fn mix_to_mono(data: &[f32], out: &mut Vec<f32>, channels: u16) {
    let ch = channels as usize;
    if ch == 0 {
        for x in out.iter_mut() { *x = 0.0; }
        return;
    }
    let frames = data.len() / ch;
    let len = out.len().min(frames);
    for i in 0..len {
        let mut sum = 0.0f32;
        for c in 0..ch {
            sum += data[i * ch + c];
        }
        out[i] = sum / ch as f32;
    }
    for x in &mut out[len..] { *x = 0.0; }
}
