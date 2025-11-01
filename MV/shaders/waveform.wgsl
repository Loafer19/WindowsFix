struct Uniforms {
    color: vec4<f32>,
    intensity: f32,
    resolution: vec2<f32>,
    mode: u32,
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
    let y_coord = 1.0 - coord.y / uniforms.resolution.y;
    let idx = u32(x * f32(arrayLength(&data)));
    let sample = data[idx] * uniforms.intensity;
    let y = (sample + 1.0) * 0.5;
    if (abs(y_coord - y) < 0.01) {
        return uniforms.color;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
