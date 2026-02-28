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
    let radius = length(pos);
    let max_radius = min(uniforms.resolution.x, uniforms.resolution.y) * 0.5;
    let normalized_radius = radius / max_radius;

    if (normalized_radius > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Map radius to a frequency band (inner = bass, outer = treble)
    let valid_len = arrayLength(&data) / 2u;
    let freq_idx = min(u32(normalized_radius * f32(valid_len)), valid_len - 1u);
    let magnitude = data[freq_idx] * uniforms.intensity;

    // Number of ripple rings
    let num_rings = 12.0;
    // Phase of the ripple based on audio magnitude at each frequency
    let ring_phase = fract(normalized_radius * num_rings);
    let ring_thickness = 0.15;
    let in_ring = ring_phase < ring_thickness;

    // Brightness based on audio magnitude at this radial band
    let brightness = clamp(magnitude * 2.0, 0.0, 1.0);

    if (in_ring && brightness > 0.05) {
        // Color: hue rotates with radius, brightness from audio
        let hue = normalized_radius + magnitude * 0.3;
        let color = hsv_to_rgb(hue, 0.9, brightness);
        return vec4<f32>(color * uniforms.color.rgb, uniforms.color.a);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs(fract(h * 6.0) * 2.0 - 1.0));
    let m = v - c;

    var rgb = vec3<f32>(0.0);
    let sector = u32(h * 6.0) % 6u;
    if (sector == 0u) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (sector == 1u) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (sector == 2u) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (sector == 3u) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (sector == 4u) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}
