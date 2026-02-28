// Terrain 3D – perspective-projected heightfield with horizon, fog and audio

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

// Multi-octave height at world position (wx, wz).
fn terrain_height(wx: f32, wz: f32) -> f32 {
    let h1 = sin(wx * 2.5 + uniforms.time * 0.5) * cos(wz * 2.5 + uniforms.time * 0.4);
    let h2 = sin(wx * 5.0 - uniforms.time * 0.7) * cos(wz * 4.8 + uniforms.time * 0.6) * 0.50;
    let h3 = sin(wx * 10.0 + uniforms.time * 1.1) * cos(wz * 9.5 + uniforms.time * 0.9) * 0.25;
    let base = (h1 + h2 + h3) * 0.25 + 0.35;   // 0..~0.7

    // Audio modulation: map wx to an FFT bin
    let n     = arrayLength(&data);
    let valid = n / 2u;
    let nx    = fract(wx * 0.15 + 0.5);
    let idx   = min(u32(nx * f32(valid)), valid - 1u);
    let audio = data[idx] * uniforms.intensity * 0.30;

    return base + audio;
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv     = fragCoord.xy / uniforms.resolution;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;

    // Screen y: 0 = bottom, 1 = top
    let sy = 1.0 - uv.y;

    // Horizon sits at ~55 % up the screen
    let horizon = 0.55;

    if sy > horizon {
        // Sky: gradient from horizon colour to deep blue at top
        let t      = (sy - horizon) / (1.0 - horizon);
        let sky_lo = vec3<f32>(0.10, 0.14, 0.25);
        let sky_hi = vec3<f32>(0.02, 0.04, 0.12);
        let sky    = mix(sky_lo, sky_hi, t);
        // Horizon sun-glow amplified by bass
        let glow   = exp(-t * t * 18.0) * uniforms.bass_energy * 0.40;
        return vec4<f32>(sky + glow * vec3<f32>(0.4, 0.6, 1.0), 1.0);
    }

    // Ground plane – use perspective projection to find world (wx, wz)
    // Camera sits at y = 1.2, looking along +z at a shallow angle
    let cam_y  = 1.20;
    let cam_fz = 0.80;   // focal length on z (controls pitch angle)
    let fov_x  = 1.40;   // half-width field of view factor

    // Inverse-project: pixel below horizon maps to ground plane
    let gy_frac = (horizon - sy) / horizon;     // 0 at horizon, 1 at bottom
    let t_ground = cam_y / (gy_frac * cam_fz + 0.0001);

    let sx      = (uv.x - 0.5) * 2.0 * aspect;
    let wx      = sx * fov_x * t_ground + uniforms.time * 0.60;   // scroll forward
    let wz      = t_ground;

    let height  = terrain_height(wx, wz);

    // Finite-difference normal
    let eps = 0.05;
    let dh_dx = terrain_height(wx + eps, wz) - terrain_height(wx - eps, wz);
    let dh_dz = terrain_height(wx, wz + eps) - terrain_height(wx, wz - eps);
    let tnorm  = normalize(vec3<f32>(-dh_dx / (2.0 * eps), 1.0, -dh_dz / (2.0 * eps)));

    // Lighting
    let light = normalize(vec3<f32>(0.4, 0.9, 0.5));
    let diff  = max(dot(tnorm, light), 0.0) * 0.75 + 0.25;

    // Height-based colour (valley = deep blue-green, peak = bright amber)
    let h_norm   = clamp(height * 1.2, 0.0, 1.0);
    let col_low  = vec3<f32>(0.05, 0.20, 0.30);
    let col_high = vec3<f32>(0.80, 0.55, 0.15);
    var ground_col = mix(col_low, col_high, h_norm) * diff;

    // Atmospheric depth fog
    let fog_t    = clamp(t_ground / 18.0, 0.0, 1.0);
    let fog_col  = vec3<f32>(0.10, 0.14, 0.25);
    ground_col   = mix(ground_col, fog_col, fog_t * fog_t);

    return vec4<f32>(ground_col, 1.0);
}
