//! Theme definitions for the application.

use crate::Color;
use serde::{Deserialize, Serialize};

/// Application theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
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
}

impl Theme {
    /// Get a theme by name.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            _ => None,
        }
    }

    /// GitHub-inspired dark theme.
    pub fn dark() -> Self {
        Self {
            bg_primary: Color::from_hex("#0d1117").unwrap(),
            bg_secondary: Color::from_hex("#161b22").unwrap(),
            bg_tertiary: Color::from_hex("#21262d").unwrap(),

            fg_primary: Color::from_hex("#c9d1d9").unwrap(),
            fg_secondary: Color::from_hex("#8b949e").unwrap(),
            fg_tertiary: Color::from_hex("#6e7681").unwrap(),

            accent_primary: Color::from_hex("#58a6ff").unwrap(),
            accent_secondary: Color::from_hex("#1f6feb").unwrap(),

            error: Color::from_hex("#f85149").unwrap(),
            warning: Color::from_hex("#d29922").unwrap(),
            success: Color::from_hex("#3fb950").unwrap(),
            info: Color::from_hex("#58a6ff").unwrap(),

            git_added: Color::from_hex("#3fb950").unwrap(),
            git_modified: Color::from_hex("#d29922").unwrap(),
            git_deleted: Color::from_hex("#f85149").unwrap(),
            git_untracked: Color::from_hex("#8b949e").unwrap(),
            git_branch: Color::from_hex("#58a6ff").unwrap(),
        }
    }

    /// GitHub-inspired light theme.
    pub fn light() -> Self {
        Self {
            bg_primary: Color::from_hex("#ffffff").unwrap(),
            bg_secondary: Color::from_hex("#f6f8fa").unwrap(),
            bg_tertiary: Color::from_hex("#eaeef2").unwrap(),

            fg_primary: Color::from_hex("#24292e").unwrap(),
            fg_secondary: Color::from_hex("#57606a").unwrap(),
            fg_tertiary: Color::from_hex("#6e7781").unwrap(),

            accent_primary: Color::from_hex("#0969da").unwrap(),
            accent_secondary: Color::from_hex("#0550ae").unwrap(),

            error: Color::from_hex("#cf222e").unwrap(),
            warning: Color::from_hex("#9a6700").unwrap(),
            success: Color::from_hex("#1a7f37").unwrap(),
            info: Color::from_hex("#0969da").unwrap(),

            git_added: Color::from_hex("#1a7f37").unwrap(),
            git_modified: Color::from_hex("#9a6700").unwrap(),
            git_deleted: Color::from_hex("#cf222e").unwrap(),
            git_untracked: Color::from_hex("#57606a").unwrap(),
            git_branch: Color::from_hex("#0969da").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("light").is_some());
        assert!(Theme::by_name("invalid").is_none());
    }
}
