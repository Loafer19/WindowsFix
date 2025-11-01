//! Audio input handling using cpal

use crate::constants::{SAMPLE_RATE, SAMPLE_SIZE, TEST_FREQUENCY};
use crate::error::{AppError, AppResult};
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Audio handler for capturing input and streaming output
pub struct AudioHandler {
    pub buffer: Arc<Mutex<Vec<f32>>>,
    channels: u16,
    _input_stream: cpal::Stream,
    _output_stream: Option<cpal::Stream>,
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
                        let mut buffer = buffer_clone.lock().unwrap();
                        if channels == 1 {
                            let len = buffer.len().min(data.len());
                            buffer[..len].copy_from_slice(&data[..len]);
                            for x in &mut buffer[len..] { *x = 0.0; }
                        } else {
                            let frame_len = data.len() / channels as usize;
                            let len = buffer.len().min(frame_len);
                            for i in 0..len {
                                let mut sum = 0.0;
                                for ch in 0..channels {
                                    sum += data[i * channels as usize + ch as usize];
                                }
                                buffer[i] = sum / channels as f32;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        }
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
                        let mut buffer = buffer_clone.lock().unwrap();
                        if channels == 1 {
                            let len = buffer.len().min(data.len());
                            for (i, x) in buffer[..len].iter_mut().enumerate() {
                                *x = data[i] as f32 / 32768.0;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        } else {
                            let frame_len = data.len() / channels as usize;
                            let len = buffer.len().min(frame_len);
                            for i in 0..len {
                                let mut sum = 0.0;
                                for ch in 0..channels {
                                    sum += data[i * channels as usize + ch as usize] as f32 / 32768.0;
                                }
                                buffer[i] = sum / channels as f32;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        }
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
                        let mut buffer = buffer_clone.lock().unwrap();
                        if channels == 1 {
                            let len = buffer.len().min(data.len());
                            for (i, x) in buffer[..len].iter_mut().enumerate() {
                                *x = (data[i] as f32 - 32768.0) / 32768.0;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        } else {
                            let frame_len = data.len() / channels as usize;
                            let len = buffer.len().min(frame_len);
                            for i in 0..len {
                                let mut sum = 0.0;
                                for ch in 0..channels {
                                    sum += (data[i * channels as usize + ch as usize] as f32 - 32768.0) / 32768.0;
                                }
                                buffer[i] = sum / channels as f32;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        }
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
                        let mut buffer = buffer_clone.lock().unwrap();
                        if channels == 1 {
                            let len = buffer.len().min(data.len());
                            for (i, x) in buffer[..len].iter_mut().enumerate() {
                                *x = (data[i] as f32 - 128.0) / 128.0;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        } else {
                            let frame_len = data.len() / channels as usize;
                            let len = buffer.len().min(frame_len);
                            for i in 0..len {
                                let mut sum = 0.0;
                                for ch in 0..channels {
                                    sum += (data[i * channels as usize + ch as usize] as f32 - 128.0) / 128.0;
                                }
                                buffer[i] = sum / channels as f32;
                            }
                            for x in &mut buffer[len..] { *x = 0.0; }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(AppError::Audio(format!("Unsupported sample format: {:?}", sample_format))),
        }?;

        // Create output stream to stream to the device (optional)
        let _output_stream = if let Ok(output_config) = device.default_output_config() {
            let output_sample_format = output_config.sample_format();
            let output_stream_config = output_config.into();

            let output_stream = match output_sample_format {
                cpal::SampleFormat::F32 => {
                    let buffer_clone = Arc::clone(&buffer);
                    device.build_output_stream(
                        &output_stream_config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let buffer = buffer_clone.lock().unwrap();
                            let len = data.len().min(buffer.len());
                            data[..len].copy_from_slice(&buffer[..len]);
                            for sample in &mut data[len..] {
                                *sample = 0.0;
                            }
                        },
                        err_fn,
                        None,
                    )?
                }
                cpal::SampleFormat::I16 => {
                    let buffer_clone = Arc::clone(&buffer);
                    device.build_output_stream(
                        &output_stream_config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            let buffer = buffer_clone.lock().unwrap();
                            let len = data.len().min(buffer.len());
                            for (i, sample) in data[..len].iter_mut().enumerate() {
                                *sample = (buffer[i] * 32767.0) as i16;
                            }
                            for sample in &mut data[len..] {
                                *sample = 0;
                            }
                        },
                        err_fn,
                        None,
                    )?
                }
                cpal::SampleFormat::U16 => {
                    let buffer_clone = Arc::clone(&buffer);
                    device.build_output_stream(
                        &output_stream_config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            let buffer = buffer_clone.lock().unwrap();
                            let len = data.len().min(buffer.len());
                            for (i, sample) in data[..len].iter_mut().enumerate() {
                                *sample = ((buffer[i] * 32767.0) + 32768.0) as u16;
                            }
                            for sample in &mut data[len..] {
                                *sample = 32768;
                            }
                        },
                        err_fn,
                        None,
                    )?
                }
                _ => return Err(AppError::Audio(format!("Unsupported output sample format: {:?}", output_sample_format))),
            };

            output_stream.play().map_err(|e| AppError::Audio(format!("Failed to start output stream: {}", e)))?;

            Some(output_stream)
        } else {
            None
        };

        stream.play().map_err(|e| AppError::Audio(format!("Failed to start input stream: {}", e)))?;

        Ok(Self {
            buffer,
            channels,
            _input_stream: stream,
            _output_stream,
        })
    }

    /// Generate test sine wave data if no audio input
    pub fn generate_test_wave(&self) -> Vec<f32> {
        (0..SAMPLE_SIZE)
            .map(|i| {
                (i as f32 * TEST_FREQUENCY * 2.0 * std::f32::consts::PI / SAMPLE_RATE).sin() * 0.5
            })
            .collect()
    }

    /// Check if audio buffer contains meaningful data
    pub fn has_audio_data(&self) -> bool {
        let buffer = self.buffer.lock().unwrap();
        !buffer.iter().all(|&x| x.abs() < 0.01)
    }
}
