// Heatmap – frequency spectrum rendered as a cold-to-hot color gradient.
// Each frequency column is independently colored based on its magnitude:
// silent = deep blue → cyan → green → yellow → red → white (max energy).
// The color_scheme uniform tints the output so the user's palette is reflected.

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
    beat_intensity: f32,
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

// Classic "plasma" heatmap gradient: cold → hot
fn heatmap_color(t: f32) -> vec3<f32> {
    let v = clamp(t, 0.0, 1.0);
    var col: vec3<f32>;
    if v < 0.2 {
        col = mix(vec3<f32>(0.0, 0.0, 0.08), vec3<f32>(0.0, 0.1, 0.9), v / 0.2);
    } else if v < 0.4 {
        col = mix(vec3<f32>(0.0, 0.1, 0.9), vec3<f32>(0.0, 0.85, 0.85), (v - 0.2) / 0.2);
    } else if v < 0.6 {
        col = mix(vec3<f32>(0.0, 0.85, 0.85), vec3<f32>(0.0, 0.9, 0.0), (v - 0.4) / 0.2);
    } else if v < 0.8 {
        col = mix(vec3<f32>(0.0, 0.9, 0.0), vec3<f32>(1.0, 0.88, 0.0), (v - 0.6) / 0.2);
    } else if v < 0.95 {
        col = mix(vec3<f32>(1.0, 0.88, 0.0), vec3<f32>(1.0, 0.08, 0.0), (v - 0.8) / 0.15);
    } else {
        col = mix(vec3<f32>(1.0, 0.08, 0.0), vec3<f32>(1.0, 1.0, 1.0), (v - 0.95) / 0.05);
    }
    // Blend with user color scheme (50% tint — preserves the heatmap gradient)
    return mix(col, col * uniforms.color.rgb * 1.6, 0.35);
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x = coord.x / uniforms.resolution.x;
    let y = 1.0 - coord.y / uniforms.resolution.y;  // 0=bottom, 1=top

    let valid_len = arrayLength(&data) / 2u;
    let freq_idx  = min(u32(x * f32(valid_len)), valid_len - 1u);
    let magnitude = max(data[freq_idx] * uniforms.intensity, 0.0);

    // Scale heat so the heatmap looks active even at moderate volumes
    let heat = clamp(magnitude * 2.2, 0.0, 1.0);

    // Beat flash: temporarily boost all cells on a hard beat
    let beat_boost = uniforms.beat_intensity * 0.25;

    // Vertical gradient: full heat at the magnitude level, cooler above
    let heat_at_y = clamp(heat - max(y - heat, 0.0) * 2.8, 0.0, 1.0);

    // Hotspot: a bright glow just above the peak frequency height
    let dist_to_peak = abs(y - heat);
    let hotspot = exp(-dist_to_peak * dist_to_peak * 60.0) * heat * 1.4;

    let final_heat = clamp(heat_at_y + hotspot + beat_boost, 0.0, 1.0);

    if final_heat < 0.015 {
        // Near-black background with very faint cool tint
        return vec4<f32>(heatmap_color(0.0) * 0.5 * (0.01 + uniforms.bass_energy * 0.03), 1.0);
    }

    return vec4<f32>(heatmap_color(final_heat), 1.0);
}
