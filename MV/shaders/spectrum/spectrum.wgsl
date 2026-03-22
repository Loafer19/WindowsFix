@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x = coord.x / uniforms.resolution.x;
    let y = 1.0 - coord.y / uniforms.resolution.y;
    // FFT fills only the first half of the buffer; map full width to valid range
    let valid_len = arrayLength(&data) / 2u;
    let idx = min(u32(x * f32(valid_len)), valid_len - 1u);
    let magnitude = data[idx] * uniforms.intensity;
    if (y < magnitude) {
        // Beat flash: brighten the bar color on strong beats
        let flash = 1.0 + uniforms.beat_intensity * 1.5;
        return vec4<f32>(
            clamp(uniforms.color.rgb * flash, vec3<f32>(0.0), vec3<f32>(1.0)),
            1.0,
        );
    }
    // Beat flash background pulse
    let bg_flash = uniforms.beat_intensity * 0.08;
    return vec4<f32>(bg_flash, bg_flash * 0.5, bg_flash * 1.5, 1.0);
}
