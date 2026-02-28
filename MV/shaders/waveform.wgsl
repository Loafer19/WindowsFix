// Waveform – anti-aliased line with subtle glow, full screen width

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

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let x       = coord.x / uniforms.resolution.x;
    // y_norm: 0 = bottom of screen, 1 = top
    let y_norm  = 1.0 - coord.y / uniforms.resolution.y;

    let data_len = arrayLength(&data);
    let idx      = min(u32(x * f32(data_len)), data_len - 1u);
    let sample   = data[idx] * uniforms.intensity;
    // Map waveform sample (-1..1) to screen y (0..1), centred at 0.5
    let wave_y   = clamp(sample * 0.45 + 0.5, 0.01, 0.99);

    let dy       = abs(y_norm - wave_y);

    // Core line (sharp) – scales with resolution so it looks 1–2 px wide
    let core_w   = 0.004;
    // Outer glow
    let glow_w   = 0.030;

    let core = smoothstep(core_w, 0.0, dy);
    let glow = smoothstep(glow_w, core_w, dy) * 0.40;

    let brightness = core + glow;
    if brightness < 0.005 {
        // Background: very dark with faint centre-line
        let center_line = exp(-abs(y_norm - 0.5) * 60.0) * 0.04;
        return vec4<f32>(center_line * uniforms.color.rgb, 1.0);
    }

    // Tint: hue shifts slightly along x and with sample amplitude
    let hue = x * 0.3 + abs(sample) * 0.2 + uniforms.time * 0.04;
    let hh  = fract(hue) * 6.0;
    let i   = floor(hh);
    let f   = hh - i;
    let p   = 1.0 - f;
    let q   = f;
    let t_  = 1.0 - f;
    var rgb = uniforms.color.rgb;  // fallback to scheme colour
    // Simple hue rotation tint blended with scheme colour
    let tint_str = 0.45;
    let ii = u32(i) % 6u;
    var tint = vec3<f32>(0.0);
    if ii == 0u { tint = vec3<f32>(1.0, q,   0.0); }
    else if ii == 1u { tint = vec3<f32>(t_,  1.0, 0.0); }
    else if ii == 2u { tint = vec3<f32>(0.0, 1.0, q  ); }
    else if ii == 3u { tint = vec3<f32>(0.0, t_,  1.0); }
    else if ii == 4u { tint = vec3<f32>(q,   0.0, 1.0); }
    else             { tint = vec3<f32>(1.0, 0.0, t_ ); }
    rgb = mix(uniforms.color.rgb, tint, tint_str);

    return vec4<f32>(rgb * brightness, 1.0);
}
