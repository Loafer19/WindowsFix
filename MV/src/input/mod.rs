//! Centralised keyboard shortcut and input management.

pub mod shortcuts;
pub use shortcuts::{ShortcutAction, ShortcutDef, SHORTCUTS, key_to_action};
