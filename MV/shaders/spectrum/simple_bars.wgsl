// Simple Bars – minimal clean spectrum analyzer bars.

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let num_bars = 64.0;
    let slot_w   = uniforms.resolution.x / num_bars;
    let bar_idx  = floor(coord.x / slot_w);
    let local_x  = fract(coord.x / slot_w);

    if local_x > 0.9 {
        return vec4<f32>(0.02, 0.02, 0.03, 1.0);
    }

    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32((bar_idx / num_bars) * f32(valid_len)), valid_len - 1u);
    let magnitude = max(data[freq_idx] * uniforms.intensity, 0.0);

    let y_norm      = 1.0 - coord.y / uniforms.resolution.y;
    let bar_height  = magnitude;

    if y_norm < bar_height {
        let t     = y_norm / max(bar_height, 0.001);
        let col   = mix(uniforms.color.rgb * 0.7, vec3<f32>(1.0), t * 0.5);
        let flash = 1.0 + uniforms.beat_intensity * 1.2;
        return vec4<f32>(clamp(col * flash, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    }

    let cap_h = 2.0 / uniforms.resolution.y;
    if y_norm < bar_height + cap_h && bar_height > 0.01 {
        return vec4<f32>(1.0, 1.0, 1.0, 0.9);
    }

    let bg = uniforms.bass_energy * 0.04;
    return vec4<f32>(bg * 0.3, bg * 0.3, bg * 0.6, 1.0);
}
