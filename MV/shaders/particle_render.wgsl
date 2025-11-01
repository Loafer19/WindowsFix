struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    color: vec4<f32>,
    intensity: f32,
    resolution: vec2<f32>,
    mode: u32,
    padding: vec4<u32>,
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
