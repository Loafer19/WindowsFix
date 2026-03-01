// Waveform History – scrolling waveform trail that fades as it ages.
//
// Each frame the Rust side pushes the newest raw-audio snapshot into slot 0
// of the history buffer and shifts older snapshots toward higher indices.
// This shader draws all 64 snapshots at once: the newest (slot 0) renders
// at full brightness in the centre of the screen, while older ones drift
// upward and fade out so they "disappear at the edge".

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
@group(0) @binding(1) var<storage, read> data: array<f32>;      // current frame (unused here)
@group(0) @binding(2) var<storage, read> history: array<f32>;   // [HISTORY_SIZE * SAMPLE_SIZE]

// Full-screen triangle
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
    // y_norm: 0 = bottom of screen, 1 = top
    let y_norm = 1.0 - coord.y / uniforms.resolution.y;

    // History layout: 64 slots × 512 samples
    // NOTE: these values must match WAVEFORM_HISTORY_SIZE and SAMPLE_SIZE in constants.rs
    let N_HISTORY:  u32 = 64u;
    let SAMPLE_CNT: u32 = 512u;

    let hist_len = arrayLength(&history);
    let data_idx = min(u32(x * f32(SAMPLE_CNT)), SAMPLE_CNT - 1u);

    var accumulated = vec3<f32>(0.0);

    for (var i = 0u; i < N_HISTORY; i++) {
        // age: 0.0 = newest (i=0), 1.0 = oldest (i=N_HISTORY-1)
        let age = f32(i) / f32(N_HISTORY - 1u);

        // Quadratic fade: newest is fully visible, oldest invisible
        let alpha = (1.0 - age) * (1.0 - age) * uniforms.intensity;
        if alpha < 0.005 {
            continue;
        }

        // Drift upward as the snapshot ages (oldest near the top)
        let y_drift = age * 0.38;

        let hist_idx = i * SAMPLE_CNT + data_idx;
        if hist_idx >= hist_len {
            continue;
        }
        let sample  = history[hist_idx];
        // Amplitude decreases slightly with age so old lines look "quieter"
        let amp_scale = 1.0 - age * 0.45;
        let wave_y  = clamp(sample * 0.40 * amp_scale + 0.5 + y_drift, 0.0, 1.0);
        let dy      = abs(y_norm - wave_y);

        // Core line + outer glow
        let core = smoothstep(0.004, 0.0,    dy) * alpha;
        let glow = smoothstep(0.038, 0.004,  dy) * 0.35 * alpha;
        let brightness = core + glow;

        if brightness > 0.001 {
            // Hue: newer lines cooler (blue), older lines warmer (red/orange)
            let hue = 0.65 - age * 0.65 + x * 0.1 + uniforms.time * 0.03;
            accumulated += brightness * hsv_to_rgb(hue, 0.80 + age * 0.20, 1.0);
        }
    }

    // Faint background glow at screen centre
    let bg = exp(-abs(y_norm - 0.5) * 25.0) * 0.018 * (1.0 + uniforms.bass_energy * 0.5);
    let bg_hue = uniforms.time * 0.05;
    let bg_col = hsv_to_rgb(bg_hue, 0.60, bg);

    return vec4<f32>(accumulated + bg_col, 1.0);
}
