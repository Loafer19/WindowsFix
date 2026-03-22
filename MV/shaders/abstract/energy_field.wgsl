// Energy Field – electric plasma that pulses with audio energy.

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv  = coord.xy / uniforms.resolution;
    let t   = uniforms.time;
    let be  = uniforms.bass_energy;
    let bi  = uniforms.beat_intensity;

    let valid_len = arrayLength(&data) / 2u;
    let freq_x    = min(u32(uv.x * f32(valid_len)), valid_len - 1u);
    let freq_y    = min(u32(uv.y * f32(valid_len)), valid_len - 1u);
    let mag_x     = data[freq_x] * uniforms.intensity;
    let mag_y     = data[freq_y] * uniforms.intensity;

    let field1 = sin((uv.x + mag_x) * 12.0 + t * 1.2) * cos((uv.y + mag_y) * 8.0 + t * 0.9);
    let field2 = sin((uv.x - uv.y) * 10.0 + t * 1.5 + be * 3.0) * sin(length(uv - 0.5) * 15.0 - t * 2.0);
    let field3 = cos((uv.x * uv.y) * 20.0 + t * 0.7 + bi * 5.0);

    let combined = (field1 + field2 + field3) / 3.0;
    let plasma   = combined * 0.5 + 0.5;

    let hue = plasma + t * 0.06 + be * 0.3;
    let sat = 0.85 + bi * 0.15;
    let val = clamp(plasma * 0.8 + be * 0.4 + bi * 0.4, 0.0, 1.0);

    let col = mix(hsv_to_rgb(hue, sat, val), uniforms.color.rgb, 0.2);

    let bolt = exp(-abs(combined) * 20.0) * (0.5 + bi);
    return vec4<f32>(clamp(col + bolt * 0.8, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
