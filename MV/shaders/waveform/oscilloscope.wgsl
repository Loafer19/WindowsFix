@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (fragCoord.xy / uniforms.resolution) * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2 = vec2<f32>(uv.x * aspect, uv.y);

    let pi = 3.14159265358979;
    let angle = atan2(uv2.y, uv2.x);          // -pi..pi
    let dist  = length(uv2);

    let normalized = (angle + pi) / (2.0 * pi); // 0..1
    let n = arrayLength(&data);
    let idx = u32(normalized * f32(n)) % n;
    let sample = data[idx] * uniforms.intensity;

    let base_radius  = 0.35;
    let waveform_r   = base_radius + sample * 0.3;
    let ring_dist    = abs(dist - waveform_r);
    let thickness    = 0.008 + uniforms.bass_energy * 0.02;

    if ring_dist < thickness {
        let t   = 1.0 - ring_dist / thickness;
        let hue = normalized + uniforms.time * 0.05 + uniforms.bass_energy * 0.3;
        let rgb = hsv_to_rgb(hue, 0.8 + uniforms.bass_energy * 0.2, 1.0);
        return vec4<f32>(rgb * t * uniforms.intensity, t);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
