//! Theme definitions for the application.
//!
//! Themes are stored as TOML files embedded at compile time.
//! Use [`Theme::by_name`] to load a built-in theme by its identifier.

use crate::Color;
use serde::{Deserialize, Serialize};

const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("dark", include_str!("themes/dark.toml")),
    ("light", include_str!("themes/light.toml")),
    ("high_contrast", include_str!("themes/high_contrast.toml")),
    ("jetbrains", include_str!("themes/jetbrains.toml")),
    ("visual_studio", include_str!("themes/visual_studio.toml")),
    ("crema", include_str!("themes/crema.toml")),
    ("ide-like", include_str!("themes/ide-like.toml")),
];

/// Application theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Human-readable display name shown in the theme picker (e.g. "Dark (GitHub)").
    /// Defaults to empty string; callers fall back to the theme's file-stem id.
    #[serde(default)]
    pub name: String,

    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,

    // Foreground colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_tertiary: Color,

    // Accent colors
    pub accent_primary: Color,
    pub accent_secondary: Color,

    // Semantic colors
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,

    // Git-specific colors
    pub git_added: Color,
    pub git_modified: Color,
    pub git_deleted: Color,
    pub git_untracked: Color,
    pub git_branch: Color,
    pub git_renamed: Color,
    pub git_conflicted: Color,
    pub git_type_changed: Color,

    // UI structural colors
    pub pane_border: Color,
    pub selection_fg: Color,
    pub overlay_bg: Color,
    pub overlay_fg: Color,
    pub hint_fg: Color,
}

impl Theme {
    /// Returns identifiers of built-in themes.
    pub fn builtin_theme_ids() -> impl Iterator<Item = &'static str> {
        BUILTIN_THEMES.iter().map(|(id, _)| *id)
    }

    /// Load a built-in theme by name.
    ///
    /// Built-in names are exposed via [`Theme::builtin_theme_ids`].
    pub fn by_name(name: &str) -> Option<Self> {
        let toml = BUILTIN_THEMES
            .iter()
            .find_map(|(id, toml)| (*id == name).then_some(*toml))?;
        match toml::from_str(toml) {
            Ok(theme) => Some(theme),
            Err(e) => {
                eprintln!("Failed to parse built-in theme '{}': {}", name, e);
                None
            }
        }
    }

    /// Returns a hard-coded dark theme as a last-resort fallback.
    pub fn fallback() -> Self {
        Self::by_name("dark").expect("built-in dark theme must always parse")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_by_name() {
        for name in Theme::builtin_theme_ids() {
            assert!(
                Theme::by_name(name).is_some(),
                "missing built-in theme: {name}"
            );
        }
        assert!(Theme::by_name("invalid").is_none());
    }

    #[test]
    fn test_all_themes_have_pane_border() {
        for name in Theme::builtin_theme_ids() {
            let theme = Theme::by_name(name).unwrap();
            // pane_border must be a valid, non-zero color
            let c = theme.pane_border;
            assert!(
                c.r >= 0.0 && c.r <= 1.0,
                "{name}: pane_border.r out of range"
            );
        }
    }
}
