// 3D Sphere – audio-reactive deformed sphere with nebula background

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
    let uv     = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2    = vec2<f32>(uv.x * aspect, uv.y);

    let radius = length(uv2);

    if radius > 1.0 {
        // Nebula background – radial glow + star field
        let bg_hue  = uniforms.time * 0.04;
        let bg_glow = uniforms.bass_energy * 0.18 / (1.0 + (radius - 1.0) * 4.0);
        let nebula  = hsv_to_rgb(bg_hue, 0.80, bg_glow) * 0.35;
        // Pseudo-random stars based on screen position
        let seed    = fract(sin(dot(uv, vec2<f32>(127.1, 311.7))) * 43758.5);
        let star    = step(0.993, seed) * 0.70;
        return vec4<f32>(nebula + star, 1.0);
    }

    let z      = sqrt(max(1.0 - radius * radius, 0.0));
    let normal = normalize(vec3<f32>(uv2.x, uv2.y, z));

    // Audio-reactive deformation
    let n        = arrayLength(&data);
    let valid    = n / 2u;
    let theta    = atan2(uv2.y, uv2.x);
    let phi      = acos(clamp(z, -1.0, 1.0));
    let a_idx    = u32((theta / (2.0 * 3.14159265) + 0.5) * f32(valid)) % valid;
    let deform   = data[a_idx] * uniforms.intensity * 0.28;

    let def_norm = normalize(normal + vec3<f32>(
        sin(theta * 4.0 + uniforms.time * 1.2)  * deform,
        cos(phi   * 3.0 + uniforms.time * 1.5)  * deform,
        sin(theta * 2.0 + phi * 2.0 + uniforms.time * 0.8) * deform,
    ));

    // Two animated lights
    let l1     = normalize(vec3<f32>( sin(uniforms.time * 0.8), cos(uniforms.time * 0.6),  1.2));
    let l2     = normalize(vec3<f32>(-cos(uniforms.time * 0.7), sin(uniforms.time * 0.5), -0.5));
    let diff1  = max(dot(def_norm, l1), 0.0);
    let diff2  = max(dot(def_norm, l2), 0.0) * 0.45;

    // Specular
    let view  = vec3<f32>(0.0, 0.0, 1.0);
    let h1    = normalize(view + l1);
    let spec  = pow(max(dot(def_norm, h1), 0.0), 42.0) * 0.60;

    // Rim
    let rim   = pow(1.0 - max(dot(view, def_norm), 0.0), 3.5) * 0.40;

    // Audio-based hue
    let hue = theta / (2.0 * 3.14159265) + phi * 0.45 + uniforms.time * 0.18;
    let sat = 0.72 + deform * 0.28;
    let val = clamp(diff1 * 0.75 + diff2 * 0.25 + rim * 0.38, 0.0, 1.0);

    let base_col = hsv_to_rgb(hue, sat, val);
    let spec_col = vec3<f32>(1.0, 0.95, 0.85) * spec;
    let rim_col  = hsv_to_rgb(hue + 0.30, 0.90, 1.0) * rim;

    return vec4<f32>(clamp(base_col + spec_col + rim_col, vec3<f32>(0.0), vec3<f32>(1.4)), 1.0);
}
