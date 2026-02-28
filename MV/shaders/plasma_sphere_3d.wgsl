// Plasma Sphere 3D – raymarched audio-reactive sphere with proper depth,
// animated surface deformation, multiple lights, specular and rim lighting.

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

// Signed distance to the audio-reactive deformed sphere.
fn sphere_sdf(p: vec3<f32>) -> f32 {
    let r   = length(p);
    let safe_r = max(r, 0.0001);
    let phi   = asin(clamp(p.z / safe_r, -1.0, 1.0));
    let theta = atan2(p.y, p.x);

    let n     = arrayLength(&data);
    let valid = n / 2u;
    let a_idx = u32((theta / (2.0 * 3.14159265) + 0.5) * f32(valid)) % valid;
    let e_idx = u32((phi   / 3.14159265         + 0.5) * f32(valid)) % valid;
    let audio = (data[a_idx] + data[e_idx]) * 0.5 * uniforms.intensity;

    // Multi-frequency surface waves – different frequencies per axis
    let w1 = sin(theta * 7.0  + uniforms.time * 1.60) * cos(phi * 5.0 + uniforms.time * 1.10) * 0.09;
    let w2 = sin(theta * 11.0 - uniforms.time * 2.10) * sin(phi * 8.0 + uniforms.time * 0.80) * 0.045;
    let w3 = cos(theta * 4.0  + phi * 3.0 + uniforms.time * 0.70) * 0.035;

    let deform = clamp((w1 + w2 + w3 + audio * 0.14) * uniforms.intensity, -0.32, 0.52);
    return r - (0.76 + deform);
}

// Central-differences normal (6 SDF samples)
fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0018;
    return normalize(vec3<f32>(
        sphere_sdf(p + vec3<f32>(e, 0.0, 0.0)) - sphere_sdf(p - vec3<f32>(e, 0.0, 0.0)),
        sphere_sdf(p + vec3<f32>(0.0, e, 0.0)) - sphere_sdf(p - vec3<f32>(0.0, e, 0.0)),
        sphere_sdf(p + vec3<f32>(0.0, 0.0, e)) - sphere_sdf(p - vec3<f32>(0.0, 0.0, e)),
    ));
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv     = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;

    // Perspective camera looking at the origin
    let ro = vec3<f32>(0.0, 0.0, 2.6);
    let rd = normalize(vec3<f32>(uv.x * aspect * 0.48, uv.y * 0.48, -1.0));

    // ── Sphere-bound early-exit: skip march if ray misses bounding sphere ──
    let oc     = ro;   // sphere at origin
    let b      = dot(oc, rd);
    let c      = dot(oc, oc) - 1.4 * 1.4;  // bounding radius 1.4
    let discr  = b * b - c;

    if discr < 0.0 {
        // Ray misses the bounding sphere → nebula background
        let bg_hue  = uniforms.time * 0.04;
        let bg_glow = uniforms.bass_energy * 0.18
                      / (1.0 + length(uv * vec2<f32>(aspect, 1.0)) * 1.8);
        let bg_col  = hsv_to_rgb(bg_hue, 0.80, bg_glow);
        // Subtle star-field sparkle
        let star_seed = fract(sin(dot(uv, vec2<f32>(127.1, 311.7))) * 43758.5);
        let star      = step(0.994, star_seed) * 0.6;
        return vec4<f32>(bg_col * 0.25 + star, 1.0);
    }

    // ── Raymarching ───────────────────────────────────────────────────────
    var t      = max(-b - sqrt(discr) - 0.05, 0.3);  // start just inside bounding sphere
    var hit_t  = -1.0;

    for (var i: i32 = 0; i < 72; i++) {
        let p = ro + rd * t;
        let d = sphere_sdf(p);
        if d < 0.0015 {
            hit_t = t;
            break;
        }
        t += max(d * 0.72, 0.005);
        if t > 5.0 { break; }
    }

    if hit_t < 0.0 {
        // Missed the sphere surface → draw nebula background
        let bg_hue  = uniforms.time * 0.04;
        let bg_glow = uniforms.bass_energy * 0.15
                      / (1.0 + length(uv * vec2<f32>(aspect, 1.0)) * 2.2);
        let bg_col  = hsv_to_rgb(bg_hue, 0.80, bg_glow);
        let star_seed = fract(sin(dot(uv, vec2<f32>(127.1, 311.7))) * 43758.5);
        let star      = step(0.994, star_seed) * 0.6;
        return vec4<f32>(bg_col * 0.25 + star, 1.0);
    }

    // ── Shading ───────────────────────────────────────────────────────────
    let hit_pos = ro + rd * hit_t;
    let normal  = calc_normal(hit_pos);
    let view    = normalize(ro - hit_pos);

    // Two animated lights
    let l1 = normalize(vec3<f32>( sin(uniforms.time * 0.7),  cos(uniforms.time * 0.5),  1.6));
    let l2 = normalize(vec3<f32>(-cos(uniforms.time * 0.6),  sin(uniforms.time * 0.4), -0.8));

    let diff1 = max(dot(normal, l1), 0.0);
    let diff2 = max(dot(normal, l2), 0.0) * 0.55;

    // Specular (Blinn-Phong)
    let h1   = normalize(view + l1);
    let spec = pow(max(dot(normal, h1), 0.0), 56.0) * 0.75;

    // Rim light
    let rim = pow(1.0 - max(dot(view, normal), 0.0), 4.0) * 0.45;

    // Audio-reactive colour
    let n     = arrayLength(&data);
    let valid = n / 2u;
    let angle = atan2(hit_pos.y, hit_pos.x);
    let a_idx = u32((angle / (2.0 * 3.14159265) + 0.5) * f32(valid)) % valid;
    let audio = data[a_idx] * uniforms.intensity;

    let hue = angle / (2.0 * 3.14159265) + uniforms.time * 0.07 + uniforms.bass_energy * 0.22;
    let sat = 0.72 + audio * 0.28;
    let val = clamp(diff1 * 0.72 + diff2 * 0.28 + rim * 0.38, 0.0, 1.0);

    let base_col = hsv_to_rgb(hue, sat, val);
    let spec_col = vec3<f32>(1.0, 0.93, 0.82) * spec;
    let rim_col  = hsv_to_rgb(hue + 0.32, 0.90, 1.0) * rim;

    let final_col = clamp(base_col + spec_col + rim_col, vec3<f32>(0.0), vec3<f32>(1.5));
    return vec4<f32>(final_col, 1.0);
}
