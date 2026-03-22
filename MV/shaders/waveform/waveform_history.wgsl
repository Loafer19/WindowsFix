@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x      = coord.x / uniforms.resolution.x;
    // y_norm: 0 = bottom of screen, 1 = top
    let y_norm = 1.0 - coord.y / uniforms.resolution.y;

    // History layout: 64 slots × 512 samples
    // NOTE: these values must match WAVEFORM_HISTORY_SIZE and SAMPLE_SIZE in constants.rs
    let N_HISTORY:  u32 = 64u;
    let SAMPLE_CNT: u32 = 512u;

    let hist_len = arrayLength(&history);
    let data_idx = min(u32(x * f32(SAMPLE_CNT)), SAMPLE_CNT - 1u);

    var accumulated = vec3<f32>(0.0);

    for (var i = 0u; i < N_HISTORY; i++) {
        // age: 0.0 = newest (i=0), 1.0 = oldest (i=N_HISTORY-1)
        let age = f32(i) / f32(N_HISTORY - 1u);

        // Quadratic fade: newest is fully visible, oldest invisible
        let alpha = (1.0 - age) * (1.0 - age) * uniforms.intensity;
        if alpha < 0.005 {
            continue;
        }

        // Drift upward as the snapshot ages (oldest near the top)
        let y_drift = age * 0.38;

        let hist_idx = i * SAMPLE_CNT + data_idx;
        if hist_idx >= hist_len {
            continue;
        }
        let sample  = history[hist_idx];
        // Amplitude decreases slightly with age so old lines look "quieter"
        let amp_scale = 1.0 - age * 0.45;
        let wave_y  = clamp(sample * 0.40 * amp_scale + 0.5 + y_drift, 0.0, 1.0);
        let dy      = abs(y_norm - wave_y);

        // Core line + outer glow
        let core = smoothstep(0.004, 0.0,    dy) * alpha;
        let glow = smoothstep(0.038, 0.004,  dy) * 0.35 * alpha;
        let brightness = core + glow;

        if brightness > 0.001 {
            // Hue: newer lines cooler (blue), older lines warmer (red/orange)
            let hue = 0.65 - age * 0.65 + x * 0.1 + uniforms.time * 0.03;
            accumulated += brightness * hsv_to_rgb(hue, 0.80 + age * 0.20, 1.0);
        }
    }

    // Faint background glow at screen centre
    let bg = exp(-abs(y_norm - 0.5) * 25.0) * 0.018 * (1.0 + uniforms.bass_energy * 0.5);
    let bg_hue = uniforms.time * 0.05;
    let bg_col = hsv_to_rgb(bg_hue, 0.60, bg);

    return vec4<f32>(accumulated + bg_col, 1.0);
}
