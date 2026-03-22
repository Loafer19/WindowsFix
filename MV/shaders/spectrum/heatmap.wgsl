@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x = coord.x / uniforms.resolution.x;
    let y = 1.0 - coord.y / uniforms.resolution.y;  // 0=bottom, 1=top

    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32(x * f32(valid_len)), valid_len - 1u);
    let magnitude = max(data[freq_idx] * uniforms.intensity, 0.0);

    // Scale heat so the heatmap looks active even at moderate volumes
    let heat = clamp(magnitude * 2.2, 0.0, 1.0);

    // Beat flash: temporarily boost all cells on a hard beat
    let beat_boost = uniforms.beat_intensity * 0.25;

    // Vertical gradient: full heat at the magnitude level, cooler above
    let heat_at_y = clamp(heat - max(y - heat, 0.0) * 2.8, 0.0, 1.0);

    // Hotspot: a bright glow just above the peak frequency height
    let dist_to_peak = abs(y - heat);
    let hotspot = exp(-dist_to_peak * dist_to_peak * 60.0) * heat * 1.4;

    let final_heat = clamp(heat_at_y + hotspot + beat_boost, 0.0, 1.0);

    if final_heat < 0.015 {
        // Near-black background with very faint cool tint
        return vec4<f32>(heatmap_color(0.0) * 0.5 * (0.01 + uniforms.bass_energy * 0.03), 1.0);
    }

    return vec4<f32>(heatmap_color(final_heat), 1.0);
}
