// Gradient bars – bars grow from the bottom, full width, gradient + reflection

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
    let num_bars  = 64.0;
    let gap_frac  = 0.08;
    let slot_w    = uniforms.resolution.x / num_bars;
    let bar_index = floor(coord.x / slot_w);
    let local_x   = fract(coord.x / slot_w);

    let valid_len  = arrayLength(&data) / 2u;
    let freq_idx   = min(u32((bar_index / num_bars) * f32(valid_len)), valid_len - 1u);
    let magnitude  = max(data[freq_idx] * uniforms.intensity, 0.0);

    let from_bottom = uniforms.resolution.y - coord.y;
    let bar_h_px    = magnitude * uniforms.resolution.y * 0.92;

    // Gap
    if local_x > 1.0 - gap_frac {
        let bg = from_bottom / uniforms.resolution.y * 0.03;
        return vec4<f32>(bg, bg * 0.6, bg * 1.2, 1.0);
    }

    let norm_freq = bar_index / num_bars;
    let hue_base  = norm_freq + uniforms.time * 0.03;

    if from_bottom <= bar_h_px && bar_h_px > 0.5 {
        let height_norm = from_bottom / bar_h_px;   // 0=base 1=top

        // Two-tone vertical gradient: warm at base → cool at top
        let col_bot = hsv_to_rgb(hue_base + 0.08, 0.95, 0.35 + height_norm * 0.3);
        let col_top = hsv_to_rgb(hue_base - 0.08, 0.80, 0.70 + height_norm * 0.25);
        let mixed   = mix(col_bot, col_top, height_norm);

        // Bright top-cap line
        let cap_norm = (bar_h_px - from_bottom) / bar_h_px;
        let cap_glow = exp(-cap_norm * cap_norm * 400.0) * 0.8;
        let final_col = mixed + vec3<f32>(cap_glow);
        return vec4<f32>(clamp(final_col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    }

    // Reflection below
    let reflect_dist = -(from_bottom - bar_h_px);
    if reflect_dist > 0.0 && reflect_dist < bar_h_px * 0.30 && bar_h_px > 0.5 {
        let fade = (1.0 - reflect_dist / (bar_h_px * 0.30)) * 0.18;
        let rc   = hsv_to_rgb(hue_base, 0.80, fade);
        return vec4<f32>(rc, 1.0);
    }

    // Background pattern
    let bg = from_bottom / uniforms.resolution.y * 0.04;
    return vec4<f32>(bg, bg * 0.6, bg * 1.2, 1.0);
}
