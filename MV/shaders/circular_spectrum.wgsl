// Circular spectrum – mirrored radial bars filling the full window

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
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let center = uniforms.resolution * 0.5;
    let pos    = coord.xy - center;
    let angle  = atan2(pos.y, pos.x);              // -π .. π
    let radius = length(pos);

    // Use the shortest axis so the circle always fits on screen
    let max_radius = min(uniforms.resolution.x, uniforms.resolution.y) * 0.46;
    let inner_r    = max_radius * 0.18;            // hollow centre

    // Map angle to FFT bin – only valid (first-half) bins
    let valid_len = arrayLength(&data) / 2u;
    let norm_angle = (angle + 3.14159265) / (2.0 * 3.14159265);  // 0..1
    let freq_idx   = min(u32(norm_angle * f32(valid_len)), valid_len - 1u);
    let magnitude  = max(data[freq_idx] * uniforms.intensity, 0.0);

    let bar_end_r  = inner_r + magnitude * (max_radius - inner_r);
    let hue        = norm_angle + uniforms.time * 0.06;

    // Draw bar between inner_r and bar_end_r
    if radius >= inner_r && radius <= bar_end_r {
        let t   = (radius - inner_r) / max(bar_end_r - inner_r, 0.001);
        let val = 0.45 + t * 0.55;
        return vec4<f32>(hsv_to_rgb(hue, 0.90, val), 1.0);
    }

    // Thin inner ring highlight
    if radius < inner_r && radius > inner_r - 2.0 {
        let glow = uniforms.bass_energy;
        return vec4<f32>(hsv_to_rgb(uniforms.time * 0.1, 0.8, glow), 1.0);
    }

    // Outside max circle: dark background with subtle radial glow from bass
    let bg_glow = uniforms.bass_energy * 0.12 * exp(-pow((radius / max_radius - 1.1) * 3.0, 2.0));
    return vec4<f32>(bg_glow * 0.4, bg_glow * 0.2, bg_glow, 1.0);
}
