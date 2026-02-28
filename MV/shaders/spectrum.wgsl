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
    var pos = vec2<f32>(0.0, 0.0);
    if (idx == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (idx == 1u) {
        pos = vec2<f32>(3.0, -1.0);
    } else if (idx == 2u) {
        pos = vec2<f32>(-1.0, 3.0);
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x = coord.x / uniforms.resolution.x;
    let y = 1.0 - coord.y / uniforms.resolution.y;
    // FFT fills only the first half of the buffer; map full width to valid range
    let valid_len = arrayLength(&data) / 2u;
    let idx = min(u32(x * f32(valid_len)), valid_len - 1u);
    let magnitude = data[idx] * uniforms.intensity;
    if (y < magnitude) {
        return uniforms.color;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
