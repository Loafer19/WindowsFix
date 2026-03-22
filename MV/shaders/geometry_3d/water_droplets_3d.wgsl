@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    // UV: centre at (0,0), correct for aspect
    let uv = (coord.xy / uniforms.resolution - 0.5) * vec2<f32>(aspect, 1.0);

    let valid_len = arrayLength(&data) / 2u;

    // --- Droplet grid ---
    // Divide the screen into a grid of cells; each cell hosts one droplet.
    let grid_scale = 4.0;                      // number of cells per unit
    let cell  = uv * grid_scale;
    let cell_id = floor(cell);                 // integer cell index
    let cell_uv = fract(cell) - 0.5;          // -0.5…0.5 within the cell

    // Per-cell random values
    let r1 = hash21(cell_id);
    let r2 = hash21(cell_id + vec2<f32>(3.7, 11.3));
    let r3 = hash21(cell_id + vec2<f32>(7.3,  5.1));

    // Map cell to a frequency band so different parts of the screen react
    // to different frequencies.
    let norm_x  = (cell_id.x / (grid_scale * aspect * 2.0) + 0.5);
    let norm_y  = (cell_id.y / (grid_scale          * 2.0) + 0.5);
    let freq_n  = clamp((norm_x + norm_y) * 0.5, 0.0, 1.0);
    let freq_i  = min(u32(freq_n * f32(valid_len)), valid_len - 1u);
    let freq_v  = max(data[freq_i] * uniforms.intensity, 0.0);

    // Phase: the droplet in this cell fires periodically, offset by r1.
    // Bass energy and frequency magnitude accelerate the repetition.
    let rate   = 0.6 + r2 * 0.8 + freq_v * 0.9 + uniforms.bass_energy * 0.4;
    let phase  = fract((uniforms.time * rate + r1));  // 0=impact, 1=fade-out

    // --- Ripple rings (multiple concentric) ---
    let d = length(cell_uv);
    var ripple_total = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let ring_phase = clamp(phase - f32(k) * 0.15, 0.0, 1.0);
        let amplitude  = freq_v * (1.0 + uniforms.bass_energy * 0.5) * (0.8 - f32(k) * 0.15);
        ripple_total  += ripple_ring(d, ring_phase, 0.04, amplitude);
    }

    // Beat-triggered single bright flash ring
    let beat_ring = ripple_ring(d, fract(uniforms.time * 0.3 + r3), 0.06, uniforms.beat_intensity * 1.2);
    ripple_total += beat_ring;

    // --- Surface shading ---
    let normal   = water_normal(uv, uniforms.time, uniforms.bass_energy);
    let light_dir = normalize(vec3<f32>(0.4, 0.6, 1.0));
    let view_dir  = vec3<f32>(0.0, 0.0, 1.0);
    let half_dir  = normalize(light_dir + view_dir);

    let diffuse  = max(dot(normal, light_dir), 0.0);
    let specular = pow(max(dot(normal, half_dir), 0.0), 48.0) * 0.6;

    // Base water colour: deep teal, tinted by color_scheme
    let water_base = mix(vec3<f32>(0.02, 0.07, 0.14), uniforms.color.rgb * 0.4, 0.45);
    let lit_water  = water_base * (0.35 + diffuse * 0.5) + vec3<f32>(specular);

    // Ripple colour: brighter, slightly desaturated, driven by freq energy
    let ripple_col  = mix(lit_water, uniforms.color.rgb + vec3<f32>(0.25), 0.55);
    let surface_col = lit_water + ripple_col * clamp(ripple_total * 1.4, 0.0, 1.2);

    // Subtle glow from bass energy at the screen centre
    let glow = uniforms.bass_energy * 0.08 * exp(-length(uv) * 1.8);
    let final_col = surface_col + uniforms.color.rgb * glow;

    return vec4<f32>(clamp(final_col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
