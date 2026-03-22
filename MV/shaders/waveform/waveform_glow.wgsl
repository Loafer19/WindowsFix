@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x      = coord.x / uniforms.resolution.x;
    let y_norm = 1.0 - coord.y / uniforms.resolution.y;

    let data_len = arrayLength(&data);
    let idx      = min(u32(x * f32(data_len)), data_len - 1u);
    let sample   = data[idx] * uniforms.intensity;

    // Centre at 0.5, scale to use ~45% of the height each way
    let wave_y = clamp(sample * 0.45 + 0.5, 0.01, 0.99);
    let dy     = abs(y_norm - wave_y);

    // Three concentric glow layers for the live waveform
    let w_core  = 0.003;
    let w_mid   = 0.012;
    let w_outer = 0.040;

    let core  = smoothstep(w_core,  0.0,     dy);
    let mid   = smoothstep(w_mid,   w_core,  dy) * 0.55;
    let outer = smoothstep(w_outer, w_mid,   dy) * 0.20;

    let brightness = core + mid + outer;

    // Hue: shifts along x, reacts to amplitude and bass
    let hue = x * 0.50 + sample * 0.25 + uniforms.bass_energy * 0.15 + uniforms.time * 0.05;
    let sat = 0.85 + abs(sample) * 0.15;
    let live_col = hsv_to_rgb(hue, sat, 1.0) * brightness;

    // History layout: SAMPLE_CNT samples per slot
    // NOTE: SAMPLE_CNT must match SAMPLE_SIZE in constants.rs
    let TRAIL_SLOTS: u32  = 16u;
    let SAMPLE_CNT:  u32  = 512u;
    let hist_len           = arrayLength(&history);
    let hist_data_idx      = min(u32(x * f32(SAMPLE_CNT)), SAMPLE_CNT - 1u);

    var trail_col = vec3<f32>(0.0);
    for (var i = 1u; i <= TRAIL_SLOTS; i++) {
        let age   = f32(i) / f32(TRAIL_SLOTS + 1u);
        let alpha = (1.0 - age) * 0.35;            // max 35% opacity for trails
        let y_off = age * 0.18;                     // drift upward

        let h_idx = i * SAMPLE_CNT + hist_data_idx;
        if h_idx >= hist_len { continue; }

        let h_sample  = history[h_idx];
        let h_wave_y  = clamp(h_sample * 0.45 + 0.5 + y_off, 0.0, 1.0);
        let h_dy      = abs(y_norm - h_wave_y);
        let h_b       = smoothstep(0.008, 0.0, h_dy) * alpha;

        if h_b > 0.001 {
            let h_hue = x * 0.50 + h_sample * 0.25 + uniforms.time * 0.04 + age * 0.3;
            trail_col += h_b * hsv_to_rgb(h_hue, 0.70, 1.0);
        }
    }

    // Background: very faint symmetric centre gradient
    let bg = exp(-abs(y_norm - 0.5) * 30.0) * 0.025 * (1.0 + uniforms.bass_energy);
    let bg_col = hsv_to_rgb(uniforms.time * 0.06, 0.70, bg);

    return vec4<f32>(live_col + trail_col + bg_col, 1.0);
}
