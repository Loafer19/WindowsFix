//! GPU update functions

use crate::common::types::VisUniforms;
use crate::config::constants::*;

use super::GpuResources;

impl GpuResources {
    pub fn update(&mut self, uniforms: &VisUniforms, audio_data: &[f32], beat_threshold: f32) {
        let time = self.start_time.elapsed().as_secs_f32();
        let smoothing = uniforms.smoothing_factor.clamp(0.01, 0.3);
        let gain = uniforms.gain.clamp(0.5, 5.0);

        let mode = uniforms.mode as usize;
        let is_spectrum = mode < self.plugins.len() && self.plugins[mode].is_spectrum;

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
            // Compute bass energy from the raw waveform RMS for non-spectrum modes.
            let rms = (waveform.iter().map(|x| x * x).sum::<f32>() / waveform.len().max(1) as f32).sqrt();
            self.bass_energy = (rms * 5.0).min(1.0);
            data_to_write = waveform;
        }

        // ── Beat detection ─────────────────────────────────────────────────────
        // `self.bass_energy` was already computed above (either from FFT or RMS);
        // push it into the rolling history (newest at index 0).
        self.energy_history.copy_within(0..BEAT_HISTORY_SIZE - 1, 1);
        self.energy_history[0] = self.bass_energy;

        let avg_energy = self.energy_history.iter().sum::<f32>() / BEAT_HISTORY_SIZE as f32;
        if avg_energy > 0.001 && self.bass_energy > beat_threshold * avg_energy {
            // Beat detected – set peak intensity proportional to the excess energy.
            let excess = (self.bass_energy / (avg_energy * beat_threshold)).min(2.0);
            self.beat_intensity = excess.min(1.0);
        } else {
            // Decay toward zero so the pulse fades over several frames.
            self.beat_intensity *= BEAT_DECAY;
            if self.beat_intensity < 0.01 {
                self.beat_intensity = 0.0;
            }
        }

        // Maintain waveform history for the waveform_history shader.
        // Always updated from raw audio so the history reflects the actual signal.
        self.history_frame_counter += 1;
        if self.history_frame_counter >= HISTORY_UPDATE_INTERVAL {
            self.history_frame_counter = 0;
            let n = WAVEFORM_HISTORY_SIZE;
            let ss = SAMPLE_SIZE;
            // Shift: slot 0 = newest, slot N-1 = oldest
            self.waveform_history.copy_within(0..(n - 1) * ss, ss);
            // Write the new waveform (gain-scaled raw audio) at slot 0
            let len = ss.min(audio_data.len());
            for i in 0..len {
                self.waveform_history[i] = audio_data[i] * gain;
            }
            for i in len..ss {
                self.waveform_history[i] = 0.0;
            }
            self.queue.write_buffer(&self.buffers.history_buffer, 0, bytemuck::cast_slice(&self.waveform_history));
        }

        let mut updated = *uniforms;
        updated.time = time;
        updated.bass_energy = self.bass_energy;
        updated.beat_intensity = self.beat_intensity;

        self.queue.write_buffer(&self.buffers.uniform_buffer, 0, bytemuck::cast_slice(&[updated]));
        self.queue.write_buffer(&self.buffers.fft_buffer, 0, bytemuck::cast_slice(&data_to_write));
    }

    /// Compute the magnitude spectrum of `audio_data` using the pre-allocated
    /// complex buffer and cached FFT planner, avoiding per-call heap allocations.
    fn compute_fft(&mut self, audio_data: &[f32]) -> Vec<f32> {
        let n = audio_data.len();
        // Resize the reusable buffer only when the input length changes (rare).
        if self.fft_complex_buf.len() != n {
            self.fft_complex_buf.resize(n, rustfft::num_complex::Complex::new(0.0, 0.0));
        }
        for (dst, &src) in self.fft_complex_buf.iter_mut().zip(audio_data.iter()) {
            *dst = rustfft::num_complex::Complex::new(src, 0.0);
        }
        let fft = self.fft_planner.plan_fft_forward(n);
        fft.process(&mut self.fft_complex_buf);
        self.fft_complex_buf[0..n / 2].iter().map(|c| c.norm()).collect()
    }
}
