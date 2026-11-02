//! Audio input handling using cpal

use crate::config::constants::SAMPLE_SIZE;
use crate::common::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Audio handler for capturing input from a selected device
pub struct AudioHandler {
    pub buffer: Arc<Mutex<Vec<f32>>>,
    /// Set by the stream error callback (e.g. device unplugged); the app then
    /// drops this handler and asks the user to pick a device again
    failed: Arc<AtomicBool>,
    _input_stream: cpal::Stream,
}

/// Lock the shared sample buffer, recovering from poisoning instead of
/// panicking (a panic inside the cpal callback would kill the audio thread)
pub fn lock_buffer(buffer: &Mutex<Vec<f32>>) -> MutexGuard<'_, Vec<f32>> {
    buffer.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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

        let failed = Arc::new(AtomicBool::new(false));
        let make_err_fn = || {
            let failed = Arc::clone(&failed);
            move |err: cpal::StreamError| {
                eprintln!("Audio stream error: {}", err);
                failed.store(true, Ordering::Relaxed);
            }
        };

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = lock_buffer(&buffer_clone);
                        mix_to_mono(data, &mut buf, channels);
                    },
                    make_err_fn(),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buf = lock_buffer(&buffer_clone);
                        let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    make_err_fn(),
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut buf = lock_buffer(&buffer_clone);
                        let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    make_err_fn(),
                    None,
                )
            }
            cpal::SampleFormat::U8 => {
                let buffer_clone = Arc::clone(&buffer);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u8], _: &cpal::InputCallbackInfo| {
                        let mut buf = lock_buffer(&buffer_clone);
                        let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 128.0) / 128.0).collect();
                        mix_to_mono(&f32_data, &mut buf, channels);
                    },
                    make_err_fn(),
                    None,
                )
            }
            _ => return Err(AppError::Audio(format!("Unsupported sample format: {:?}", sample_format))),
        }?;

        stream.play().map_err(|e| AppError::Audio(format!("Failed to start input stream: {}", e)))?;

        Ok(Self {
            buffer,
            failed,
            _input_stream: stream,
        })
    }

    /// True once the stream reported an error (device lost, driver reset, ...)
    pub fn is_failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
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
