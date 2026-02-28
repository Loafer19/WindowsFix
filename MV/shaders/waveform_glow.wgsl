// Waveform Glow – multi-layer neon glow waveform, full screen width

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
    let x      = coord.x / uniforms.resolution.x;
    // y_norm: 0 = bottom, 1 = top (correct for waveform)
    let y_norm = 1.0 - coord.y / uniforms.resolution.y;

    let data_len = arrayLength(&data);
    let idx      = min(u32(x * f32(data_len)), data_len - 1u);
    let sample   = data[idx] * uniforms.intensity;

    // Centre at 0.5, scale to use ~45 % of the height each way
    let wave_y = clamp(sample * 0.45 + 0.5, 0.01, 0.99);
    let dy     = abs(y_norm - wave_y);

    // Three concentric glow layers
    let w_core  = 0.003;
    let w_mid   = 0.012;
    let w_outer = 0.040;

    let core  = smoothstep(w_core,  0.0,     dy);
    let mid   = smoothstep(w_mid,   w_core,  dy) * 0.55;
    let outer = smoothstep(w_outer, w_mid,   dy) * 0.20;

    let brightness = core + mid + outer;

    // Hue: shifts along x, reacts to amplitude and bass
    let hue = x * 0.50 + sample * 0.25 + uniforms.bass_energy * 0.15 + uniforms.time * 0.05;
    let sat = 0.85 + abs(sample) * 0.15;
    let col = hsv_to_rgb(hue, sat, 1.0);

    // Background: very faint symmetric centre gradient
    let bg = exp(-abs(y_norm - 0.5) * 30.0) * 0.025 * (1.0 + uniforms.bass_energy);
    let bg_col = hsv_to_rgb(uniforms.time * 0.06, 0.70, bg);

    return vec4<f32>(col * brightness + bg_col, 1.0);
}
