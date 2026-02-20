To run the 'MV' Rust project (a music visualizer application), follow these steps:

1. Open a terminal and navigate to the `MV` directory: `cd MV`
2. Run the application: `cargo run`

The application will:
- List available audio input devices
- Prompt you to select one by number
- Launch a window with audio visualizations once a device is selected

Ensure you have Rust installed (via [rustup](https://rustup.rs/)) and an audio input device available. The app uses GPU acceleration via wgpu and audio processing via cpal.


The MV music visualizer supports the following keyboard controls while running:

- **Space** or **P**: Cycle through visualization modes (switches between different shaders: bars_3d, circular_spectrum, gradient_bars, mandala, spectrum, waveform_glow, waveform)
- **F**: Toggle fullscreen mode
- **T**: Toggle window transparency (semi-transparent overlay on Windows)
- **Up Arrow**: Increase visualization intensity
- **Down Arrow**: Decrease visualization intensity
- **Escape**: Exit the application (or exit fullscreen if in fullscreen mode)

The app visualizes audio input in real-time using GPU shaders. Each mode provides a different visual representation of the audio spectrum or waveform.
