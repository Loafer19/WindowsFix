struct Uniforms {
    color: vec4<f32>,
    intensity: f32,
    resolution: vec2<f32>,
    mode: u32,
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
    let max_radius = min(uniforms.resolution.x, uniforms.resolution.y) * 0.4;

    if (radius > max_radius) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Map angle to frequency index (0 to 2π -> 0 to arrayLength)
    let normalized_angle = (angle + 3.14159) / (2.0 * 3.14159);
    let freq_idx = u32(normalized_angle * f32(arrayLength(&data)));

    // Map radius to magnitude threshold
    let magnitude = data[freq_idx] * uniforms.intensity;
    let threshold = radius / max_radius;

    if (magnitude > threshold) {
        let hue = normalized_angle;
        let saturation = 1.0;
        let value = magnitude;
        return vec4<f32>(hsv_to_rgb(hue, saturation, value), 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
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
