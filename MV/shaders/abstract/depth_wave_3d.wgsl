const PI:     f32 = 3.14159265358979;
const N_ROWS: i32 = 24;   // depth layers

// Wave height for a given row.
// • Each row shifts the FFT read-head by a per-row offset so every line
//   samples a slightly different frequency region.
// • An independent sine oscillation gives each row its own motion.
fn row_height(x_norm: f32, row_i: i32, depth: f32) -> f32 {
    let n     = arrayLength(&data);
    let valid = n / 2u;

    // Frequency-shifted x so each row reads a different spectral slice
    let shift = f32(row_i) * 0.042;
    let fx    = fract(x_norm + shift);
    let idx   = min(u32(fx * f32(valid)), valid - 1u);
    let audio = max(data[idx] * uniforms.intensity, 0.0);

    // Per-row oscillation – different frequency and phase per row
    let osc_freq  = 6.0 + f32(row_i) * 0.4;
    let osc_speed = 1.0 - depth * 0.35;
    let osc_phase = f32(row_i) * 0.78;
    let osc = sin(x_norm * osc_freq * PI + uniforms.time * osc_speed + osc_phase)
              * 0.065 * clamp(uniforms.bass_energy + 0.20, 0.0, 1.0);

    return clamp((audio + osc), 0.0, 0.92);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv     = (fragCoord.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;

    // ── Perspective camera above and behind the grid ──────────────────────
    // Camera sits at y=1.3, z=-2.8.  Looking toward +z along the flat grid.
    let cam_y     = 1.30;
    let cam_z     = -2.80;
    let fov_scale = 0.58;

    // Simple pinhole ray (no matrix needed – camera looks along +z)
    let rdx = uv.x * aspect * fov_scale;
    let rdy = uv.y * fov_scale;
    let rdz = 1.0;
    let rd_len = sqrt(rdx * rdx + rdy * rdy + rdz * rdz);
    let rx = rdx / rd_len;
    let ry = rdy / rd_len;
    let rz = rdz / rd_len;

    let z_near = 0.40;
    let z_far  = 6.50;
    let half_w = 2.30;   // half-width of the wave grid in world units

    var out_col = vec3<f32>(0.0);
    var hit     = false;

    // ── Iterate rows front→back (nearest first for correct occlusion) ─────
    for (var i: i32 = 0; i < N_ROWS; i++) {
        let depth = f32(i) / f32(N_ROWS - 1);   // 0 = front, 1 = back
        let z_row = z_near + depth * (z_far - z_near);

        // Intersect the camera ray with the horizontal plane z = z_row
        if rz < 0.001 { break; }
        let t     = (z_row - cam_z) / rz;
        let x_3d  = 0.0 + rx * t;               // cam_x = 0
        let y_ray = cam_y + ry * t;             // world-space height of ray at z_row

        // Ray is well below the grid – nothing further can be visible
        if y_ray < -0.20 { break; }

        // Clip to wave-grid width
        if x_3d < -half_w || x_3d > half_w { continue; }
        let x_norm = (x_3d + half_w) / (2.0 * half_w);

        // World-unit wave height at this column and row
        let h      = row_height(x_norm, i, depth);
        let wave_y = h * 1.55;          // scale from [0,1] to world units

        // Line thickness in world units (thinner with distance, thicker at peaks)
        let thickness = (0.022 + h * 0.009) * (1.0 - depth * 0.28);

        let dy = abs(y_ray - wave_y);

        if dy < thickness {
            // ── On the wave-surface line ──────────────────────────────────
            let edge_t = 1.0 - dy / thickness;
            // Hue: position along x + depth tint + slow time drift
            let hue = x_norm * 0.68 + depth * 0.26 + uniforms.time * 0.045;
            let val = (0.50 + edge_t * 0.50) * (1.0 - depth * 0.40);
            var col = hsv_to_rgb(hue, 0.80, clamp(val, 0.0, 1.0));
            // Bright specular crest highlight
            let crest = edge_t * edge_t * 0.55 * (1.0 - depth * 0.5);
            col = clamp(col + crest, vec3<f32>(0.0), vec3<f32>(1.0));
            out_col = col;
            hit = true;
            break;

        } else if y_ray < wave_y && y_ray >= 0.0 {
            // ── Inside the wave body – face fill (occludes everything behind) ──
            let height_t = y_ray / max(wave_y, 0.001);
            let hue      = x_norm * 0.68 + depth * 0.26 + uniforms.time * 0.045;
            let val      = height_t * (1.0 - depth * 0.62) * 0.28;
            out_col = hsv_to_rgb(hue, 0.92, clamp(val, 0.0, 1.0));
            hit = true;
            break;

        }
        // y_ray >= wave_y: ray passes above this row → check next (farther) row
    }

    if !hit {
        // ── Sky / horizon background ──────────────────────────────────────
        let sky_t   = uv.y * 0.5 + 0.5;          // 0 = bottom, 1 = top
        let horizon = exp(-sky_t * sky_t * 5.5) * uniforms.bass_energy * 0.28;
        let sky_bot = vec3<f32>(0.010, 0.010, 0.050);
        let sky_top = vec3<f32>(0.000, 0.012, 0.090);
        out_col = mix(sky_bot, sky_top, sky_t) + horizon * vec3<f32>(0.12, 0.30, 0.80);
    }

    return vec4<f32>(out_col, 1.0);
}
