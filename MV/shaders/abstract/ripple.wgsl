@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    // Aspect-corrected UV centred at 0
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv     = (coord.xy / uniforms.resolution - 0.5) * vec2<f32>(aspect, 1.0);
    let radius = length(uv);

    let valid_len = arrayLength(&data) / 2u;
    let num_rings = 16.0;

    // Map radius to a frequency band (inner = bass, outer = treble),
    // using the full screen so even corners are covered.
    let max_radius = 0.72 * aspect;   // covers the corner diagonal at standard aspect
    let norm_r     = clamp(radius / max_radius, 0.0, 1.0);
    let freq_idx   = min(u32(norm_r * f32(valid_len)), valid_len - 1u);
    let magnitude  = max(data[freq_idx] * uniforms.intensity, 0.0);

    // Outward-travelling ripple phase driven per ring by the ring's own magnitude
    let ring_phase  = fract(norm_r * num_rings - uniforms.time * (0.6 + magnitude * 0.8));
    let ring_width  = 0.18;
    let in_ring     = ring_phase < ring_width;
    let brightness  = clamp(magnitude * 2.2, 0.0, 1.0);

    if in_ring && brightness > 0.04 {
        let edge_t = 1.0 - ring_phase / ring_width;         // 1 at leading edge
        let hue    = norm_r + uniforms.time * 0.07 + magnitude * 0.25;
        let val    = brightness * (0.55 + edge_t * 0.45);
        let col    = hsv_to_rgb(hue, 0.88, val) * uniforms.color.rgb;
        return vec4<f32>(col, 1.0);
    }

    // Subtle dark background that reacts to bass
    let bg = uniforms.bass_energy * 0.06 * exp(-radius * 2.5);
    return vec4<f32>(bg * 0.3, bg * 0.3, bg * 0.6, 1.0);
}
