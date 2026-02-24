# Windows Fix

![logo](./SM/src-tauri/icons/logo.png)

A collection of tools and scripts to fix and enhance Windows experience :)

## Music Visualizer (MV)

A Rust application that visualises audio input in real-time using GPU-accelerated shaders (wgpu).

### Features

- Multiple visualisation modes (spectrum, waveform, bars, mandala, particles …)
- GPU particle system overlay
- Fullscreen support
- Window transparency toggle (Windows)
- Adjustable intensity

### Running

```bash
cd MV
cargo run
```

Select your audio input device when prompted – the visualiser window opens immediately.

### Keyboard Controls

| Key | Action |
|-----|--------|
| `Space` / `P` | Cycle through visualisation modes |
| `F` | Toggle fullscreen |
| `T` | Toggle window transparency |
| `↑` / `↓` | Increase / decrease intensity |
| `Esc` | Exit (or leave fullscreen) |

> Controls are shown on screen for the first 10 seconds after launch.

## Services Manager

A Tauri + Vue3 application to manage Windows Services

With AI integration to help you understand what each service does and if safe to disable it

![web](./.github/screenshots/sm-home.jpg)
![web](./.github/screenshots/sm-info.jpg)

## Scripts

They can:

- explorer
  - set grouping to none
  - unpin useless folders
  - unpin network tab

```bash
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```
