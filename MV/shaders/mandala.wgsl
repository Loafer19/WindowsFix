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
    let center = uniforms.resolution * 0.5;
    let pos = coord.xy - center;
    let angle = atan2(pos.y, pos.x);
    let radius = length(pos);
    let max_radius = min(uniforms.resolution.x, uniforms.resolution.y) * 0.45;

    if (radius > max_radius || radius < 20.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Create spiral pattern
    let spiral_angle = angle + log(radius) * 2.0;
    let normalized_angle = (spiral_angle / (2.0 * 3.14159)) % 1.0;

    // Map to frequency data
    let freq_idx = u32(normalized_angle * f32(arrayLength(&data)));
    let magnitude = data[freq_idx] * uniforms.intensity;

    // Create mandala effect with multiple layers
    let layer1 = sin(spiral_angle * 8.0) * 0.5 + 0.5;
    let layer2 = sin(spiral_angle * 12.0 + 3.14159 * 0.5) * 0.5 + 0.5;
    let layer3 = sin(spiral_angle * 16.0 + 3.14159) * 0.5 + 0.5;

    let combined_pattern = (layer1 + layer2 + layer3) / 3.0;
    let threshold = 0.6 + magnitude * 0.4;

    if (combined_pattern > threshold) {
        // Color based on angle and magnitude
        let hue = normalized_angle + magnitude * 0.2;
        let saturation = 0.8;
        let value = magnitude * combined_pattern;

        return vec4<f32>(hsv_to_rgb(hue, saturation, value), 1.0);
    }

    // Subtle background mandala
    let bg_pattern = sin(angle * 6.0) * sin(radius * 0.05) * 0.03 + 0.03;
    return vec4<f32>(bg_pattern, bg_pattern * 0.5, bg_pattern * 0.8, 1.0);
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
