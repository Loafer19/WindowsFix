// 3D Cubes field – full-cell cubes with 3D shading, audio-driven height

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

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let grid  = 8.0;
    let gap   = 0.10;            // fraction of each cell used as gap (both axes)
    let uv    = fragCoord.xy / uniforms.resolution;

    let cell_x = floor(uv.x * grid);
    let cell_y = floor(uv.y * grid);
    // Position inside this cell, normalised 0..1
    let lx = fract(uv.x * grid);
    let ly = fract(uv.y * grid);

    // Map cell to FFT bin
    let n       = arrayLength(&data);
    let valid   = n / 2u;
    let flat    = cell_x + cell_y * grid;   // 0..63
    let a_idx   = min(u32(flat / (grid * grid) * f32(valid)), valid - 1u);
    let height  = clamp(data[a_idx] * uniforms.intensity * 0.85 + 0.12, 0.12, 1.0);

    // Gaps between cubes
    let margin = gap * 0.5;
    if lx < margin || lx > 1.0 - margin || ly < margin || ly > 1.0 - margin {
        let bg = 0.05 * (sin(uniforms.time * 0.3 + flat * 0.2) * 0.5 + 0.5);
        return vec4<f32>(bg * 0.3, bg * 0.3, bg * 0.5, 1.0);
    }

    // Inner position inside the bar area (0..1)
    let ix = (lx - margin) / (1.0 - gap);
    let iy = (ly - margin) / (1.0 - gap);  // 0=top 1=bottom in screen coords

    // Bars grow from BOTTOM; top of bar is at screen-y = cell_top + (1-height)*cell_h
    // In normalised cell space, bar occupies iy > 1-height
    let bar_top_norm = 1.0 - height;
    if iy < bar_top_norm {
        // Above bar – empty
        let bg = 0.04 * (sin(uniforms.time * 0.3 + flat * 0.17) * 0.5 + 0.5);
        return vec4<f32>(bg * 0.3, bg * 0.3, bg * 0.5, 1.0);
    }

    let height_t = (iy - bar_top_norm) / height;  // 0=top of bar, 1=bottom

    // 3D shading: top face is brighter, right face lighter than left
    let top_cap  = height_t < 0.04;
    let right_side = ix > 0.92;

    let hue = (cell_x / grid + cell_y / grid * 0.3 + uniforms.time * 0.08);
    var shade: f32;
    if top_cap {
        shade = 0.90;
    } else if right_side {
        shade = 0.30 + height_t * 0.25;
    } else {
        shade = 0.50 + height_t * 0.35;
    }

    let rgb = hsv_to_rgb(hue, 0.78, clamp(shade, 0.0, 1.0));
    return vec4<f32>(rgb, 1.0);
}
