// Kaleidoscope – mirrored triangular pattern pulsing with the music.

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (coord.xy / uniforms.resolution - vec2<f32>(0.5)) * 2.0;
    let asp = uniforms.resolution.x / uniforms.resolution.y;

    var angle  = atan2(uv.y, uv.x * asp);
    let radius = length(vec2<f32>(uv.x * asp, uv.y));

    let N       = 6.0;
    let sector  = 3.14159265 * 2.0 / N;
    angle = fract(angle / sector) * sector;
    if angle > sector * 0.5 { angle = sector - angle; }

    let scale  = 1.0 + uniforms.bass_energy * 0.5 + uniforms.beat_intensity * 0.3;
    let r_mod  = fract(radius * scale + uniforms.time * 0.15);
    let a_mod  = angle / sector;

    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32(r_mod * f32(valid_len)), valid_len - 1u);
    let magnitude = data[freq_idx] * uniforms.intensity;

    let hue  = a_mod + r_mod * 0.4 + uniforms.time * 0.05;
    let sat  = 0.8 + magnitude * 0.2;
    let val  = clamp(magnitude * 1.5 + 0.2, 0.0, 1.0);
    let col  = hsv_to_rgb(hue, sat, val);

    let ring   = abs(fract(radius * scale * 2.0 + uniforms.time * 0.1) - 0.5);
    let spoke  = a_mod;
    let lines  = (smoothstep(0.04, 0.0, ring) + smoothstep(0.03, 0.0, spoke)) * 0.5;

    return vec4<f32>(col + lines * uniforms.color.rgb, 1.0);
}
