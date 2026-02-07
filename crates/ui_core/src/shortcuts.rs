//! Keyboard shortcuts and actions.

use serde::{Deserialize, Serialize};

/// Keyboard key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Special keys
    Escape,
    Enter,
    Tab,
    Space,
}

/// Keyboard modifiers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub command: bool,
}

/// Application actions that can be triggered by shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    OpenRepo,
    CloseRepo,
    RefreshRepo,
    Quit,
}

/// A keyboard shortcut binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: Key,
    pub modifiers: Modifiers,
    pub action: Action,
}

impl Shortcut {
    /// Get default shortcuts.
    pub fn defaults() -> Vec<Self> {
        vec![
            Shortcut {
                key: Key::O,
                modifiers: Modifiers { ctrl: true, ..Default::default() },
                action: Action::OpenRepo,
            },
            Shortcut {
                key: Key::R,
                modifiers: Modifiers { ctrl: true, ..Default::default() },
                action: Action::RefreshRepo,
            },
            Shortcut {
                key: Key::Q,
                modifiers: Modifiers { ctrl: true, ..Default::default() },
                action: Action::Quit,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shortcuts() {
        let shortcuts = Shortcut::defaults();
        assert_eq!(shortcuts.len(), 3);

        // Check Ctrl+O for OpenRepo
        assert!(shortcuts.iter().any(|s| {
            s.key == Key::O && s.modifiers.ctrl && s.action == Action::OpenRepo
        }));
    }
}
