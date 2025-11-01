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
    let x = coord.x / uniforms.resolution.x;
    let idx = u32(x * f32(arrayLength(&data)));
    let sample = data[idx] * uniforms.intensity;
    let waveform_y = (sample + 1.0) * 0.5 * uniforms.resolution.y;

    let distance = abs(coord.y - waveform_y);
    let glow_width = 3.0;

    if (distance < glow_width) {
        let alpha = 1.0 - (distance / glow_width);
        let glow_intensity = alpha * alpha; // Quadratic falloff

        // Create a glowing effect with color variation
        let hue = sample * 0.5 + 0.5; // Map sample to hue
        let rgb = hsv_to_rgb(hue, 0.8, glow_intensity);

        return vec4<f32>(rgb, glow_intensity);
    }

    // Subtle background glow
    let bg_distance = min(
        abs(coord.y - uniforms.resolution.y * 0.5),
        min(coord.x, uniforms.resolution.x - coord.x)
    );
    let bg_glow = exp(-bg_distance * 0.01) * 0.05;
    return vec4<f32>(bg_glow, bg_glow, bg_glow, 1.0);
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
