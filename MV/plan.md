# MV Project Restructuring Plan

## Current Structure Analysis
The current `MV/src/` has a flat structure with modules like `app/`, `input/`, and top-level files for `audio.rs`, `gpu.rs`, `plugin.rs`, etc. The goal is to reorganize into a more modular, layered architecture.

## Proposed New Structure

### MV/src/
```
MV/src/
├── app/
│   ├── mod.rs                 (App main struct - refactored from current mod.rs)
│   ├── event_handler.rs       (Keyboard/mouse events - moved from current)
│   ├── lifecycle.rs           (Init/update/render - moved from current)
│   └── state.rs               (AppState struct - extracted from mod.rs)
├── ui/
│   ├── mod.rs                 (UI manager - extracted from lifecycle.rs)
│   ├── theme.rs               (Design system - new, for egui theming)
│   ├── notifications.rs       (Toast system - new, for notifications)
│   ├── components/
│   │   ├── mod.rs             (Component exports)
│   │   ├── button.rs          (Custom button component - new)
│   │   ├── card.rs            (Custom card component - new)
│   │   ├── stat.rs            (Stat display component - new)
│   │   └── slider.rs          (Custom slider component - new)
│   └── panels/
│       ├── mod.rs             (Panel exports)
│       ├── controls.rs        (Controls panel - extracted from lifecycle.rs)
│       ├── settings.rs        (Settings panel - extracted from lifecycle.rs)
│       ├── dashboard.rs       (Dashboard panel - new)
│       ├── shader_inspector.rs (Shader inspector - new)
│       ├── shader_browser.rs  (Shader browser - extracted from lifecycle.rs)
│       ├── shader_selector.rs (Shader selector - new)
│       ├── help_overlay.rs    (Help overlay - extracted from lifecycle.rs)
│       └── context_menu.rs    (Context menu - new)
├── input/
│   ├── mod.rs                 (Input manager - moved from current)
│   └── shortcuts.rs           (Keyboard shortcuts - moved from current)
├── graphics/
│   ├── mod.rs                 (Graphics manager - from gpu.rs)
│   ├── context.rs             (GPU context - from gpu.rs)
│   ├── buffers.rs             (Buffer management - from gpu.rs)
│   ├── analysis.rs            (FFT/analysis - from gpu.rs)
│   ├── pipeline.rs            (Pipeline creation - from gpu.rs)
│   └── render.rs              (Rendering logic - from gpu.rs)
├── visualization/
│   ├── mod.rs                 (Visualization manager - from plugin.rs)
│   ├── plugin.rs              (Plugin struct - from plugin.rs)
│   ├── loader.rs              (Plugin loading - from plugin.rs)
│   ├── registry.rs            (Shader registry - from plugin.rs)
│   └── shader_info.rs         (Shader info - from plugin.rs)
├── config/
│   ├── mod.rs                 (Config manager)
│   ├── settings.rs            (App settings - moved from current)
│   ├── constants.rs           (Constants - moved from current)
│   ├── colors.rs              (Color schemes - new)
│   └── persistence.rs         (Save/load logic - new)
├── platform/
│   ├── mod.rs                 (Platform abstraction)
│   └── windows.rs             (Windows-specific code - extracted from app/mod.rs)
├── types.rs                   (Type definitions - moved from current)
├── error.rs                   (Error types - moved from current)
├── audio.rs                   (Audio handling - moved from current)
└── main.rs                    (Entry point - minimal, moved from current)
```

### MV/shaders/
```
MV/shaders/
├── common.wgsl               (Utilities library - moved from current)
├── spectrum/                 (Spectrum visualizations)
│   ├── bars_3d.wgsl          (moved)
│   ├── circular_spectrum.wgsl (moved)
│   ├── gradient_bars.wgsl    (moved)
│   ├── heatmap.wgsl          (moved)
│   ├── simple_bars.wgsl      (moved)
│   ├── kaleidoscope.wgsl     (moved)
│   └── energy_field.wgsl     (moved from abstract/)
├── waveform/                 (Waveform visualizations)
│   ├── waveform.wgsl         (moved)
│   ├── waveform_glow.wgsl    (moved)
│   ├── waveform_history.wgsl (moved)
│   ├── oscilloscope.wgsl     (moved)
│   └── neon_pulse.wgsl       (moved)
├── geometry_3d/              (3D geometry effects)
│   ├── cubes_3d.wgsl         (moved)
│   ├── sphere_3d.wgsl        (moved)
│   ├── terrain_3d.wgsl       (moved)
│   └── water_droplets_3d.wgsl (moved)
├── abstract/                 (Abstract effects)
│   ├── mandala.wgsl          (moved)
│   ├── ripple.wgsl           (moved)
│   ├── tunnel_3d.wgsl        (moved)
│   ├── wave_3d.wgsl          (moved)
│   ├── depth_wave_3d.wgsl    (moved)
│   └── plasma_sphere_3d.wgsl (moved)
├── common_history.wgsl       (History utilities - moved from current)
├── compute_particles.wgsl    (Particle compute - moved from current)
└── particle_render.wgsl      (Particle render - moved from current)
```

## Implementation Steps
1. Create new directory structure
2. Split large files (gpu.rs, plugin.rs, app/mod.rs) into smaller modules
3. Extract UI code from lifecycle.rs into ui/ modules
4. Move and refactor existing code
5. Update all mod declarations and imports
6. Reorganize shader files
7. Test compilation and fix any issues

## Benefits
- Better separation of concerns
- Easier maintenance and testing
- More modular architecture
- Clearer code organization
