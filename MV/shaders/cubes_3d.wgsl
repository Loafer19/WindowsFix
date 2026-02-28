// 3D Cubes field visualization

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

    // Create a grid of cubes
    let grid_size = 8.0;
    let cube_size = 1.0 / grid_size;

    let grid_x = floor(uv.x * grid_size);
    let grid_y = floor(uv.y * grid_size);

    let local_uv = (uv - vec2<f32>(grid_x, grid_y) / grid_size) * grid_size;

    // Audio data for this cube
    let n = arrayLength(&data);
    let audio_idx = u32((grid_x / grid_size + grid_y / grid_size * grid_size) * f32(n) / (grid_size * grid_size)) % n;
    let height = data[audio_idx] * uniforms.intensity * 0.8 + 0.2;

    // Check if we're inside the cube
    if (local_uv.x < cube_size && local_uv.y < cube_size && local_uv.y < height) {
        // 3D position within cube
        let cube_pos = vec3<f32>(local_uv.x / cube_size, local_uv.y / height, 0.0);

        // Simple lighting
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let normal = vec3<f32>(0.0, 0.0, 1.0); // Top face
        let diffuse = max(dot(normal, light_dir), 0.0) * 0.7 + 0.3;

        // Color based on position and audio
        let hue = grid_x / grid_size + grid_y / grid_size + uniforms.time * 0.1;
        let saturation = 0.6;
        let value = diffuse * height;

        let rgb = hsv_to_rgb(hue, saturation, value);
        return vec4<f32>(rgb, 1.0);
    }

    // Background
    let bg_hue = uv.x + uv.y + uniforms.time * 0.05;
    let bg_rgb = hsv_to_rgb(bg_hue, 0.3, 0.1);
    return vec4<f32>(bg_rgb, 1.0);
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
