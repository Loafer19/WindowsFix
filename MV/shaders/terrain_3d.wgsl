// 3D Terrain visualization

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
    let uv = fragCoord.xy / uniforms.resolution;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2 = vec2<f32>(uv.x * aspect, uv.y);

    // Create terrain height using multiple octaves of noise
    let freq1 = 4.0;
    let freq2 = 8.0;
    let freq3 = 16.0;

    let height1 = sin(uv2.x * freq1 + uniforms.time) * cos(uv2.y * freq1 + uniforms.time * 0.7);
    let height2 = sin(uv2.x * freq2 + uniforms.time * 1.3) * cos(uv2.y * freq2 + uniforms.time * 0.9) * 0.5;
    let height3 = sin(uv2.x * freq3 + uniforms.time * 2.1) * cos(uv2.y * freq3 + uniforms.time * 1.5) * 0.25;

    let base_height = (height1 + height2 + height3) * 0.5 + 0.5;

    // Audio modulation
    let n = arrayLength(&data);
    let audio_x = u32(uv.x * f32(n)) % n;
    let audio_y = u32(uv.y * f32(n)) % n;
    let audio_mod = (data[audio_x] + data[audio_y]) * 0.5 * uniforms.intensity;

    let total_height = base_height + audio_mod * 0.3;

    // 3D lighting
    let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));

    // Calculate normal using finite differences
    let eps = 0.01;
    let h_dx = (sin((uv2.x + eps) * freq1) * cos(uv2.y * freq1) +
                sin((uv2.x + eps) * freq2) * cos(uv2.y * freq2) * 0.5 +
                sin((uv2.x + eps) * freq3) * cos(uv2.y * freq3) * 0.25) * 0.5 + 0.5 - base_height;
    let h_dy = (sin(uv2.x * freq1) * cos((uv2.y + eps) * freq1) +
                sin(uv2.x * freq2) * cos((uv2.y + eps) * freq2) * 0.5 +
                sin(uv2.x * freq3) * cos((uv2.y + eps) * freq3) * 0.25) * 0.5 + 0.5 - base_height;

    let normal = normalize(vec3<f32>(-h_dx / eps, -h_dy / eps, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.8 + 0.2;

    // Color based on height
    let height_color = mix(vec3<f32>(0.2, 0.4, 0.1), vec3<f32>(0.8, 0.6, 0.3), total_height);
    let final_color = height_color * diffuse;

    // Add some fog effect
    let fog = uv.y * 0.3;
    let fogged_color = mix(final_color, vec3<f32>(0.5, 0.7, 0.9), fog);

    return vec4<f32>(fogged_color, 1.0);
}
