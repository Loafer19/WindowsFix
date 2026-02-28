// 3D Bars visualization – bars grow from the bottom, full screen width

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
    let num_bars  = 32.0;
    let gap_frac  = 0.10;          // fraction of slot width used as gap
    let slot_w    = uniforms.resolution.x / num_bars;
    let bar_index = floor(coord.x / slot_w);
    let local_x   = fract(coord.x / slot_w); // 0..1 inside the slot

    // Valid FFT range
    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32((bar_index / num_bars) * f32(valid_len)), valid_len - 1u);
    let magnitude = max(data[freq_idx] * uniforms.intensity, 0.0);

    // Bars grow from the BOTTOM: pixel is in bar when distance from bottom < bar height
    let from_bottom = uniforms.resolution.y - coord.y;
    let bar_h_px    = magnitude * uniforms.resolution.y * 0.95;

    // Gap between bars
    if local_x > 1.0 - gap_frac {
        let bg = from_bottom / uniforms.resolution.y * 0.04;
        return vec4<f32>(bg * 0.5, bg * 0.5, bg, 1.0);
    }

    let bar_x = local_x / (1.0 - gap_frac);  // 0..1 inside bar body
    let hue   = bar_index / num_bars * 0.72 + uniforms.time * 0.04;

    if from_bottom <= bar_h_px && bar_h_px > 0.5 {
        let height_norm = from_bottom / bar_h_px;  // 0=bottom 1=top
        let cap_h       = max(bar_h_px * 0.035, 2.0);
        let in_cap      = from_bottom > bar_h_px - cap_h;

        var shade: f32;
        if in_cap {
            // Bright top cap face
            shade = 0.95 - bar_x * 0.15;
        } else {
            // Front face: darker at base, lighter toward cap; right edge highlight
            shade = 0.30 + height_norm * 0.45;
            if bar_x > 0.88 {
                shade = shade * 1.35;
            }
        }
        shade = clamp(shade, 0.0, 1.0);
        return vec4<f32>(hsv_to_rgb(hue, 0.85, shade), 1.0);
    }

    // Subtle floor reflection
    let reflect_dist = -(from_bottom - bar_h_px);
    if reflect_dist < bar_h_px * 0.25 && bar_h_px > 0.5 {
        let fade = 1.0 - reflect_dist / (bar_h_px * 0.25);
        return vec4<f32>(hsv_to_rgb(hue, 0.85, 0.10 * fade * fade), 1.0);
    }

    // Background
    let bg = from_bottom / uniforms.resolution.y * 0.05;
    return vec4<f32>(bg * 0.5, bg * 0.5, bg, 1.0);
}
