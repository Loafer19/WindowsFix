//! Audio analysis and FFT processing

use crate::config::constants::*;
use rustfft::{FftPlanner, num_complex::Complex};
use std::time::Instant;

/// Audio analysis state and processing
pub struct AudioAnalysis {
    pub smoothed_fft: Vec<f32>,
    pub bass_energy: f32,
    pub waveform_history: Vec<f32>,
    pub history_frame_counter: u32,
    pub energy_history: Vec<f32>,
    pub beat_intensity: f32,
    fft_complex_buf: Vec<Complex<f32>>,
    fft_planner: FftPlanner<f32>,
    start_time: Instant,
}

impl AudioAnalysis {
    pub fn new() -> Self {
        Self {
            smoothed_fft: vec![0.0f32; SAMPLE_SIZE / 2],
            bass_energy: 0.0,
            waveform_history: vec![0.0f32; WAVEFORM_HISTORY_SIZE * SAMPLE_SIZE],
            history_frame_counter: 0,
            energy_history: vec![0.0f32; BEAT_HISTORY_SIZE],
            beat_intensity: 0.0,
            fft_complex_buf: vec![Complex::new(0.0, 0.0); SAMPLE_SIZE],
            fft_planner: FftPlanner::new(),
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self, uniforms: &crate::common::types::VisUniforms, audio_data: &[f32], beat_threshold: f32) {
        let time = self.start_time.elapsed().as_secs_f32();
        let smoothing = uniforms.smoothing_factor.clamp(0.01, 0.3);
        let gain = uniforms.gain.clamp(0.5, 5.0);

        let mode = uniforms.mode as usize;
        let is_spectrum = mode < 20 && mode != 8 && mode != 9 && mode != 10 && mode != 11 && mode != 12 && mode != 13 && mode != 14 && mode != 15 && mode != 16 && mode != 17 && mode != 18 && mode != 19; // Approximation

        let data_to_write: Vec<f32>;
        if is_spectrum {
            let mut magnitudes = self.compute_fft(audio_data);
            let len = magnitudes.len() as f32;
            for m in &mut magnitudes {
                *m /= len;
                *m = (*m * 50.0 * uniforms.intensity * gain).min(1.0);
            }
            let n = magnitudes.len().min(self.smoothed_fft.len());
            for i in 0..n {
                self.smoothed_fft[i] = self.smoothed_fft[i] * (1.0 - smoothing) + magnitudes[i] * smoothing;
            }
            let bass_bins = BASS_BIN_COUNT.min(self.smoothed_fft.len());
            let raw_bass = self.smoothed_fft[..bass_bins].iter().sum::<f32>() / bass_bins as f32;
            self.bass_energy = (raw_bass * 10.0).min(1.0);
            data_to_write = self.smoothed_fft.clone();
        } else {
            let mut waveform = audio_data.to_vec();
            for s in &mut waveform {
                *s *= gain;
            }
            let rms = (waveform.iter().map(|x| x * x).sum::<f32>() / waveform.len().max(1) as f32).sqrt();
            self.bass_energy = (rms * 5.0).min(1.0);
            data_to_write = waveform;
        }

        // Beat detection
        self.energy_history.copy_within(0..BEAT_HISTORY_SIZE - 1, 1);
        self.energy_history[0] = self.bass_energy;

        let avg_energy = self.energy_history.iter().sum::<f32>() / BEAT_HISTORY_SIZE as f32;
        if avg_energy > 0.001 && self.bass_energy > beat_threshold * avg_energy {
            let excess = (self.bass_energy / (avg_energy * beat_threshold)).min(2.0);
            self.beat_intensity = excess.min(1.0);
        } else {
            self.beat_intensity *= BEAT_DECAY;
            if self.beat_intensity < 0.01 {
                self.beat_intensity = 0.0;
            }
        }

        // Maintain waveform history
        self.history_frame_counter += 1;
        if self.history_frame_counter >= HISTORY_UPDATE_INTERVAL {
            self.history_frame_counter = 0;
            let n = WAVEFORM_HISTORY_SIZE;
            let ss = SAMPLE_SIZE;
            self.waveform_history.copy_within(0..(n - 1) * ss, ss);
            let len = ss.min(audio_data.len());
            for i in 0..len {
                self.waveform_history[i] = audio_data[i] * gain;
            }
            for i in len..ss {
                self.waveform_history[i] = 0.0;
            }
        }
    }

    fn compute_fft(&mut self, audio_data: &[f32]) -> Vec<f32> {
        let n = audio_data.len();
        if self.fft_complex_buf.len() != n {
            self.fft_complex_buf.resize(n, Complex::new(0.0, 0.0));
        }
        for (dst, &src) in self.fft_complex_buf.iter_mut().zip(audio_data.iter()) {
            *dst = Complex::new(src, 0.0);
        }
        let fft = self.fft_planner.plan_fft_forward(n);
        fft.process(&mut self.fft_complex_buf);
        self.fft_complex_buf[0..n / 2].iter().map(|c| c.norm()).collect()
    }
}
