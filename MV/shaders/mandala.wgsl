// Mandala – full-screen radial symmetry pattern driven by audio

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
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv     = (coord.xy / uniforms.resolution - 0.5) * vec2<f32>(aspect, 1.0);
    let radius = length(uv);

    // Slowly rotating base angle
    let rot    = uniforms.time * 0.08;
    let angle  = atan2(uv.y, uv.x) + rot;

    // Spiral coordinate: combine angle and log-radius so the pattern tiles inward
    let spiral_angle = angle + log(max(radius, 0.001)) * 1.8;
    let norm_angle   = fract(spiral_angle / (2.0 * 3.14159265));  // 0..1

    // FFT magnitude for this angular slice
    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32(norm_angle * f32(valid_len)), valid_len - 1u);
    let magnitude = max(data[freq_idx] * uniforms.intensity, 0.0);

    // Three petal layers at different frequencies
    let layer1 = sin(spiral_angle *  8.0 + uniforms.time * 0.5)  * 0.5 + 0.5;
    let layer2 = sin(spiral_angle * 12.0 - uniforms.time * 0.35) * 0.5 + 0.5;
    let layer3 = sin(spiral_angle * 18.0 + uniforms.time * 0.7)  * 0.5 + 0.5;

    // Bass pulse makes patterns expand slightly
    let pulse   = 1.0 + uniforms.bass_energy * 0.12;
    let pattern = (layer1 * layer2 + layer2 * layer3) * 0.5 * pulse;
    let threshold = 0.52 + magnitude * 0.38;

    if pattern > threshold {
        let hue = norm_angle + magnitude * 0.18 + uniforms.time * 0.04;
        let sat = 0.80 + uniforms.bass_energy * 0.20;
        let val = clamp(magnitude * pattern * 1.4, 0.0, 1.0);
        return vec4<f32>(hsv_to_rgb(hue, sat, val), 1.0);
    }

    // Subtle background – dark spiral hints
    let bg_hint = sin(spiral_angle * 6.0) * sin(radius * 8.0) * 0.015 + 0.015;
    let bg_hue  = norm_angle + uniforms.time * 0.03;
    return vec4<f32>(hsv_to_rgb(bg_hue, 0.70, bg_hint * (1.0 + uniforms.bass_energy)), 1.0);
}
