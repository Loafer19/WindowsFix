@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv     = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2    = vec2<f32>(uv.x * aspect, uv.y);

    let angle  = atan2(uv2.y, uv2.x);
    let radius = length(uv2);

    // Flight speed driven partly by bass energy
    let speed     = 2.2 + uniforms.bass_energy * 1.5;
    let tunnel_z  = uniforms.time * speed;

    // Depth coordinate: maps radius to "depth" (smaller radius = further in)
    let depth     = 1.0 / (radius + 0.04);

    // Audio lookup by angle
    let n         = arrayLength(&data);
    let valid     = n / 2u;
    let a_idx     = u32((angle / (2.0 * 3.14159265) + 0.5) * f32(valid)) % valid;
    let wall_audio = data[a_idx] * uniforms.intensity;

    // Tunnel wall shape: circular with audio bumps and sinusoidal ridges
    let ridges    = sin(angle * 6.0 + tunnel_z * 0.25) * 0.10
                  + sin(angle * 12.0 - tunnel_z * 0.15) * 0.05;
    let wall_r    = 1.0 + ridges + wall_audio * 0.35;
    let wall_dist = abs(depth - wall_r) / (wall_r * 0.5);

    // Colour – hue cycles with depth and time
    let hue  = angle / (2.0 * 3.14159265) + depth * 0.04 + tunnel_z * 0.08;
    let fog  = clamp(1.0 - exp(-wall_dist * 1.8), 0.0, 1.0);
    let val  = clamp((1.0 - fog) * (0.4 + wall_audio * 0.6), 0.0, 1.0);

    var col = hsv_to_rgb(hue, 0.85, val);

    // Bright ring where the wall_dist is very small (surface of tunnel)
    let ring_glow = exp(-wall_dist * wall_dist * 12.0) * 0.7 * (1.0 + uniforms.bass_energy * 0.5);
    col = clamp(col + ring_glow * vec3<f32>(1.0, 0.85, 0.65), vec3<f32>(0.0), vec3<f32>(1.0));

    // Centre glow – bass pulse
    let centre_glow = exp(-radius * radius * 3.5) * uniforms.bass_energy * 0.45;
    col += centre_glow * vec3<f32>(0.3, 0.55, 1.0);

    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
