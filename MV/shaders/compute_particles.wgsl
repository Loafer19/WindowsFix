struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    lifetime: f32,
    padding: vec3<f32>,
    color: vec4<f32>,
};

struct Uniforms {
    color: vec4<f32>,
    intensity: f32,
    resolution: vec2<f32>,
    mode: u32,
    padding: vec4<u32>,
};

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<storage, read> fft_data: array<f32>;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&particles)) {
        return;
    }

    var p = particles[idx];

    let len = arrayLength(&fft_data);
    var scale: f32;
    var num_bins: u32;
    if (uniforms.mode == 0u) { // spectrum
        scale = f32(len) / 2.0;
        num_bins = 10u;
    } else {
        scale = 1.0;
        num_bins = len / 10u; // fewer bins for waveform
    }

    var bass = 0.0;
    for (var i = 0u; i < num_bins; i = i + 1u) {
        if (i < len) {
            bass += abs(fft_data[i]);
        }
    }
    bass /= f32(num_bins) * scale;
    bass *= uniforms.intensity * 5.0; // amplify

    p.velocity.y += bass * 0.5;
    p.velocity.y -= 0.01; // gravity
    p.velocity *= 0.98; // damping
    p.position += p.velocity;
    p.lifetime -= 0.01;

    // Bounce off bottom
    if (p.position.y < -1.0) {
        p.position.y = -1.0;
        p.velocity.y = -p.velocity.y * 0.8;
    }
    // Bounce off sides
    if (p.position.x > 1.0) {
        p.position.x = 1.0;
        p.velocity.x = -p.velocity.x * 0.8;
    } else if (p.position.x < -1.0) {
        p.position.x = -1.0;
        p.velocity.x = -p.velocity.x * 0.8;
    }

    if (p.lifetime <= 0.0 || p.position.y < -1.0) {
        p.position = vec2<f32>((f32(idx) / f32(arrayLength(&particles)) - 0.5) * 2.0, -1.0);
        p.velocity = vec2<f32>(0.0, bass * 2.0);
        p.lifetime = 2.0 + bass;
        p.color = uniforms.color;
    }

    particles[idx] = p;
}
