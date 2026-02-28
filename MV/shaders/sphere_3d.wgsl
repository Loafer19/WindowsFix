// 3D Sphere visualization

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
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv2 = vec2<f32>(uv.x * aspect, uv.y);

    // Sphere coordinates
    let radius = length(uv2);
    if (radius > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let z = sqrt(1.0 - radius * radius);
    let normal = normalize(vec3<f32>(uv2.x, uv2.y, z));

    // Audio-reactive deformation
    let n = arrayLength(&data);
    let theta = atan2(uv2.y, uv2.x);
    let phi = acos(z);

    let audio_idx = u32((theta / (2.0 * 3.14159) + 0.5) * f32(n)) % n;
    let deformation = data[audio_idx] * uniforms.intensity * 0.3;

    let deformed_normal = normalize(normal + vec3<f32>(
        sin(theta * 4.0 + uniforms.time) * deformation,
        cos(phi * 3.0 + uniforms.time * 1.5) * deformation,
        sin(theta * 2.0 + phi * 2.0 + uniforms.time * 0.8) * deformation
    ));

    // 3D lighting
    let light_dir = normalize(vec3<f32>(sin(uniforms.time), cos(uniforms.time), 1.0));
    let diffuse = max(dot(deformed_normal, light_dir), 0.0);

    let rim_light = pow(1.0 - dot(deformed_normal, vec3<f32>(0.0, 0.0, 1.0)), 3.0);

    // Color based on position and audio
    let hue = theta / (2.0 * 3.14159) + phi * 0.5 + uniforms.time * 0.2;
    let saturation = 0.7 + deformation * 0.3;
    let value = diffuse * 0.8 + rim_light * 0.4 + data[audio_idx] * uniforms.intensity * 0.2;

    let rgb = hsv_to_rgb(hue, saturation, value);

    return vec4<f32>(rgb, 1.0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let hh = fract(h) * 6.0;
    let i = floor(hh);
    let f = hh - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let ii = u32(i) % 6u;
    if ii == 0u { return vec3<f32>(v, t, p); }
    if ii == 1u { return vec3<f32>(q, v, p); }
    if ii == 2u { return vec3<f32>(p, v, t); }
    if ii == 3u { return vec3<f32>(p, q, v); }
    if ii == 4u { return vec3<f32>(t, p, v); }
    return vec3<f32>(v, p, q);
}
