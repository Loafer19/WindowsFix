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
    var pos = vec2<f32>(0.0, 0.0);
    if (idx == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (idx == 1u) {
        pos = vec2<f32>(3.0, -1.0);
    } else if (idx == 2u) {
        pos = vec2<f32>(-1.0, 3.0);
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let num_bars = 64.0;
    let bar_width = uniforms.resolution.x / num_bars;
    let bar_index = floor(coord.x / bar_width);
    // FFT fills only the first half of the buffer; map bars to valid range
    let valid_len = arrayLength(&data) / 2u;
    let freq_idx = min(u32((bar_index / num_bars) * f32(valid_len)), valid_len - 1u);

    let magnitude = data[freq_idx] * uniforms.intensity;
    let bar_height = magnitude * uniforms.resolution.y;

    if (coord.y < bar_height) {
        let normalized_height = coord.y / bar_height;
        let normalized_freq = bar_index / num_bars;

        // Create gradient based on frequency and height
        let hue1 = normalized_freq; // Low to high frequency
        let hue2 = normalized_freq + 0.3; // Offset for gradient
        let saturation = 0.9;
        let value1 = normalized_height;
        let value2 = normalized_height * 0.7;

        let color1 = hsv_to_rgb(hue1, saturation, value1);
        let color2 = hsv_to_rgb(hue2, saturation, value2);

        // Vertical gradient within each bar
        let gradient_factor = coord.y / bar_height;
        let final_color = mix(color1, color2, gradient_factor);

        return vec4<f32>(final_color, 1.0);
    }

    // Subtle background pattern
    let pattern = sin(coord.x * 0.01) * sin(coord.y * 0.01) * 0.02 + 0.02;
    return vec4<f32>(pattern, pattern, pattern, 1.0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs((h * 6.0) % 2.0 - 1.0));
    let m = v - c;

    var rgb = vec3<f32>(0.0);
    if (h < 1.0/6.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 2.0/6.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 3.0/6.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 4.0/6.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 5.0/6.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}
