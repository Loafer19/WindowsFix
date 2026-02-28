// 3D Tunnel visualization

struct Uniforms {
    color: vec4<f32>,
    intensity: f32,
    padding1: f32,
    resolution: vec2<f32>,
    mode: u32,
    padding3a: u32,
    padding3b: u32,
    padding3c: u32,
    padding2: vec3<u32>,
    time: f32,
    bass_energy: f32,
    smoothing_factor: f32,
    gain: f32,
    padding4: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(0) @binding(1) var<storage, read> data: array<f32>;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    return vec4<f32>(pos[idx], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2 = vec2<f32>(uv.x * aspect, uv.y);

    // Tunnel coordinates
    let angle = atan2(uv2.y, uv2.x);
    let radius = length(uv2);

    // Moving through tunnel
    let tunnel_z = uniforms.time * 2.0;
    let tunnel_radius = 1.0 / (radius + 0.1);

    // Audio-reactive tunnel walls
    let n = arrayLength(&data);
    let audio_idx = u32((angle / (2.0 * 3.14159) + 0.5) * f32(n)) % n;
    let wall_modulation = data[audio_idx] * uniforms.intensity * 0.5;

    let wall_distance = abs(tunnel_radius + sin(angle * 8.0 + tunnel_z) * 0.2 + wall_modulation - 1.0);

    // 3D depth effect
    let depth = 1.0 - wall_distance * 0.5;
    let fog = 1.0 - exp(-wall_distance * 2.0);

    // Color based on depth and audio
    let hue = angle / (2.0 * 3.14159) + tunnel_z * 0.1;
    let saturation = 0.8;
    let value = depth * (1.0 - fog * 0.8) + data[audio_idx] * uniforms.intensity * 0.3;

    let rgb = hsv_to_rgb(hue, saturation, value);

    // Add some glow
    let glow = exp(-wall_distance * wall_distance * 10.0);
    let glow_color = vec3<f32>(1.0, 0.8, 0.6) * glow * 0.5;

    return vec4<f32>(rgb + glow_color, 1.0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let hh = fract(h) * 6.0;
    let i = floor(hh);
    let f = hh - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let ii = u32(i) % 6u;
    if ii == 0u { return vec3<f32>(v, t, p); }
    if ii == 1u { return vec3<f32>(q, v, p); }
    if ii == 2u { return vec3<f32>(p, v, t); }
    if ii == 3u { return vec3<f32>(p, q, v); }
    if ii == 4u { return vec3<f32>(t, p, v); }
    return vec3<f32>(v, p, q);
}
