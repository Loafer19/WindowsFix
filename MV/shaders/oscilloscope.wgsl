// Radial oscilloscope visualization

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
