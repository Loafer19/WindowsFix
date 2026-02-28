// 3D Wave visualization

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

// Fullscreen triangle: vertices outside [-1,1] clip to full viewport coverage
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
    let uv = (fragCoord.xy / uniforms.resolution) * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2 = vec2<f32>(uv.x * aspect, uv.y);

    // Create wave pattern
    let wave_freq = 8.0;
    let wave_x = sin(uv2.x * wave_freq + uniforms.time * 2.0) * 0.5;
    let wave_y = cos(uv2.y * wave_freq + uniforms.time * 1.5) * 0.5;
    let wave_height = (wave_x + wave_y) * 0.5;

    // Modulate with audio data
    let n = arrayLength(&data);
    let idx = u32((uv2.x * 0.5 + 0.5) * f32(n)) % n;
    let audio_mod = data[idx] * uniforms.intensity;

    let total_height = wave_height + audio_mod * 0.3;

    // 3D lighting effect
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(vec3<f32>(
        -cos(uv2.x * wave_freq + uniforms.time * 2.0) * wave_freq * 0.5,
        -sin(uv2.y * wave_freq + uniforms.time * 1.5) * wave_freq * 0.5,
        1.0
    ));
    let diffuse = max(dot(normal, light_dir), 0.0);

    // Color based on height and audio
    let hue = (total_height + 1.0) * 0.5 + uniforms.time * 0.1;
    let saturation = 0.8 + audio_mod * 0.2;
    let value = 0.5 + diffuse * 0.5;

    let rgb = hsv_to_rgb(hue, saturation, value);
    return vec4<f32>(rgb, 1.0);
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
