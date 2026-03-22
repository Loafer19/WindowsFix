// Neon Pulse – bright neon waveform that flashes hard on every beat.

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x      = coord.x / uniforms.resolution.x;
    let y_norm = 1.0 - coord.y / uniforms.resolution.y;

    let data_len = arrayLength(&data);
    let idx      = min(u32(x * f32(data_len)), data_len - 1u);
    let sample   = data[idx] * uniforms.intensity;

    let wave_y = clamp(sample * 0.45 + 0.5, 0.01, 0.99);
    let dy     = abs(y_norm - wave_y);

    let w_core  = 0.002;
    let w_hot   = 0.007;
    let w_glow  = 0.030;
    let w_halo  = 0.070;

    let core  = smoothstep(w_core, 0.0,    dy);
    let hot   = smoothstep(w_hot,  w_core, dy) * 0.70;
    let glow  = smoothstep(w_glow, w_hot,  dy) * 0.30;
    let halo  = smoothstep(w_halo, w_glow, dy) * 0.08;

    let brightness = core + hot + glow + halo;

    let beat_flash = uniforms.beat_intensity;
    let hue  = x * 0.6 + uniforms.time * 0.07 + abs(sample) * 0.3;
    let base = hsv_to_rgb(hue, 1.0 - beat_flash * 0.5, 1.0);
    let col  = mix(base, vec3<f32>(1.0), beat_flash * 0.6) * brightness;

    let cline = exp(-abs(y_norm - 0.5) * 80.0) * 0.03 * (1.0 + uniforms.bass_energy);

    return vec4<f32>(col + cline * uniforms.color.rgb, 1.0);
}
