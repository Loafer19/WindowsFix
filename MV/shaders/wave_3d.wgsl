// 3D Wave – animated surface wave with audio modulation and depth shading

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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let hh = fract(h) * 6.0;
    let i  = floor(hh);
    let f  = hh - i;
    let p  = v * (1.0 - s);
    let q  = v * (1.0 - s * f);
    let t  = v * (1.0 - s * (1.0 - f));
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
    let uv     = (fragCoord.xy / uniforms.resolution) * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let p      = vec2<f32>(uv.x * aspect, uv.y);

    // Three-octave wave height
    let wf = 7.0;
    let h1 = sin(p.x * wf         + uniforms.time * 1.8) * cos(p.y * wf         + uniforms.time * 1.2);
    let h2 = sin(p.x * wf * 2.1   - uniforms.time * 1.4) * cos(p.y * wf * 1.9   + uniforms.time * 0.9) * 0.55;
    let h3 = sin(p.x * wf * 3.7   + uniforms.time * 2.5) * cos(p.y * wf * 3.3   - uniforms.time * 1.7) * 0.28;
    let wave_h = (h1 + h2 + h3) * 0.33;

    // Audio modulation mapped to x position
    let n      = arrayLength(&data);
    let valid  = n / 2u;
    let a_idx  = min(u32((p.x * 0.5 + 0.5) * f32(valid)), valid - 1u);
    let audio  = data[a_idx] * uniforms.intensity;
    let total  = wave_h + audio * 0.35;

    // Surface normal via analytic derivatives
    let dh_dx = cos(p.x * wf + uniforms.time * 1.8) * wf
              + cos(p.x * wf * 2.1 - uniforms.time * 1.4) * wf * 2.1 * 0.55
              + cos(p.x * wf * 3.7 + uniforms.time * 2.5) * wf * 3.7 * 0.28;
    let dh_dy = cos(p.y * wf + uniforms.time * 1.2) * (-wf)
              + cos(p.y * wf * 1.9 + uniforms.time * 0.9) * (-wf * 1.9) * 0.55
              + cos(p.y * wf * 3.3 - uniforms.time * 1.7) * (-wf * 3.3) * 0.28;
    let surf_norm = normalize(vec3<f32>(-dh_dx * 0.08, -dh_dy * 0.08, 1.0));

    // Two lights
    let l1    = normalize(vec3<f32>( sin(uniforms.time * 0.5), cos(uniforms.time * 0.4),  1.8));
    let l2    = normalize(vec3<f32>(-cos(uniforms.time * 0.6), sin(uniforms.time * 0.3), -1.0));
    let diff1 = max(dot(surf_norm, l1), 0.0);
    let diff2 = max(dot(surf_norm, l2), 0.0) * 0.40;

    // Depth cue: use |p| to simulate distance from centre (farther = darker + cooler)
    let dist_cue = length(p) * 0.25;
    let depth_cool = dist_cue * 0.18;

    let hue = (total + 1.0) * 0.35 + uniforms.time * 0.06 - depth_cool;
    let sat = 0.80 + audio * 0.20;
    let val = clamp((diff1 + diff2) * 0.65 + 0.20 + audio * 0.15 - dist_cue * 0.3, 0.0, 1.0);

    let rgb = hsv_to_rgb(hue, sat, val);

    // Specular flash on peaks
    let view = vec3<f32>(0.0, 0.0, 1.0);
    let h_   = normalize(view + l1);
    let spec = pow(max(dot(surf_norm, h_), 0.0), 38.0) * 0.50;

    return vec4<f32>(clamp(rgb + spec, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
