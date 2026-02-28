struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

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

@vertex
fn vs_main(
    @location(0) quad_pos: vec2<f32>,
    @location(1) instance_pos: vec2<f32>,
    @location(2) instance_color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let particle_size = 0.02;
    let size = vec2<f32>(particle_size * aspect, particle_size);
    let pos = instance_pos + quad_pos * size;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.color = instance_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
