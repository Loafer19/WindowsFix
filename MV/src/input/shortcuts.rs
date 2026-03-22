//! Centralised keyboard shortcut registry.

use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    NextVisualization,
    PrevVisualization,
    CycleWindowMode,
    CycleBeatSensitivity,
    DecreaseOpacity,
    IncreaseOpacity,
    IncreaseIntensity,
    DecreaseIntensity,
    ToggleFullscreen,
    ToggleInfo,
    ToggleSettings,
    ToggleDeviceSelector,
    ToggleShaderBrowser,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub struct ShortcutDef {
    pub key_label:   &'static str,
    pub description: &'static str,
    pub category:    &'static str,
    pub action:      ShortcutAction,
}

pub const SHORTCUTS: &[ShortcutDef] = &[
    ShortcutDef { key_label: "Space / M",   description: "Next visualization",                       category: "Visualizations", action: ShortcutAction::NextVisualization },
    ShortcutDef { key_label: "Shift+Space", description: "Previous visualization",                   category: "Visualizations", action: ShortcutAction::PrevVisualization },
    ShortcutDef { key_label: "T",           description: "Cycle: Normal → Transparent → Overlay",   category: "Window",         action: ShortcutAction::CycleWindowMode },
    ShortcutDef { key_label: "← / →",      description: "Decrease / increase opacity",              category: "Window",         action: ShortcutAction::DecreaseOpacity },
    ShortcutDef { key_label: "F11",         description: "Toggle fullscreen",                        category: "Window",         action: ShortcutAction::ToggleFullscreen },
    ShortcutDef { key_label: "B",           description: "Cycle beat sensitivity: Low / Med / High", category: "Audio",          action: ShortcutAction::CycleBeatSensitivity },
    ShortcutDef { key_label: "↑ / ↓",      description: "Increase / decrease intensity",            category: "Audio",          action: ShortcutAction::IncreaseIntensity },
    ShortcutDef { key_label: "F1",          description: "Toggle help panel",                        category: "Interface",      action: ShortcutAction::ToggleInfo },
    ShortcutDef { key_label: "F2",          description: "Open settings",                            category: "Interface",      action: ShortcutAction::ToggleSettings },
    ShortcutDef { key_label: "F3",          description: "Select audio device",                      category: "Interface",      action: ShortcutAction::ToggleDeviceSelector },
    ShortcutDef { key_label: "F4",          description: "Open shader browser",                      category: "Interface",      action: ShortcutAction::ToggleShaderBrowser },
    ShortcutDef { key_label: "Esc",         description: "Exit (or exit fullscreen)",                category: "Application",    action: ShortcutAction::Exit },
];

pub fn key_to_action(key: KeyCode) -> Option<ShortcutAction> {
    match key {
        KeyCode::Space | KeyCode::KeyM => Some(ShortcutAction::NextVisualization),
        KeyCode::KeyT                  => Some(ShortcutAction::CycleWindowMode),
        KeyCode::KeyB                  => Some(ShortcutAction::CycleBeatSensitivity),
        KeyCode::ArrowLeft             => Some(ShortcutAction::DecreaseOpacity),
        KeyCode::ArrowRight            => Some(ShortcutAction::IncreaseOpacity),
        KeyCode::ArrowUp               => Some(ShortcutAction::IncreaseIntensity),
        KeyCode::ArrowDown             => Some(ShortcutAction::DecreaseIntensity),
        KeyCode::F11                   => Some(ShortcutAction::ToggleFullscreen),
        KeyCode::F1                    => Some(ShortcutAction::ToggleInfo),
        KeyCode::F2                    => Some(ShortcutAction::ToggleSettings),
        KeyCode::F3                    => Some(ShortcutAction::ToggleDeviceSelector),
        KeyCode::F4                    => Some(ShortcutAction::ToggleShaderBrowser),
        KeyCode::Escape                => Some(ShortcutAction::Exit),
        _                              => None,
    }
}
