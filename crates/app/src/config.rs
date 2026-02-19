//! Application configuration management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Saved commit message draft for a repository (summary + description).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitDraft {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub recent_repos: Vec<PathBuf>,

    #[serde(default = "default_max_recent")]
    pub max_recent: usize,

    #[serde(default = "default_pane_widths")]
    pub pane_widths: [f32; 3],

    /// JSON-serialized dock layout state
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dock_layout: Option<String>,

    /// Saved commit message drafts keyed by repository path string.
    #[serde(default)]
    pub commit_drafts: HashMap<String, CommitDraft>,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_max_recent() -> usize {
    10
}

fn default_pane_widths() -> [f32; 3] {
    [0.25, 0.35, 0.40] // Commit History, Changed Files, Diff Viewer
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            recent_repos: Vec::new(),
            max_recent: default_max_recent(),
            pane_widths: default_pane_widths(),
            dock_layout: None,
            commit_drafts: HashMap::new(),
        }
    }
}

use anyhow::Context;
use directories::ProjectDirs;
use std::fs;

/// Get the configuration file path.
fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "crabontree", "CrabOnTree")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

/// Load configuration from disk, falling back to defaults on error.
pub fn load_config() -> AppConfig {
    let Some(path) = config_path() else {
        tracing::warn!("Could not determine config directory, using defaults");
        return AppConfig::default();
    };

    match fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => {
                tracing::info!("Loaded configuration from {}", path.display());
                config
            }
            Err(e) => {
                tracing::warn!("Failed to parse config file: {}, using defaults", e);
                AppConfig::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!("Config file not found, using defaults");
            AppConfig::default()
        }
        Err(e) => {
            tracing::warn!("Failed to read config file: {}, using defaults", e);
            AppConfig::default()
        }
    }
}

/// Save configuration to disk.
pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = config_path().context("Could not determine config directory")?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let contents = toml::to_string_pretty(config)
        .context("Failed to serialize configuration")?;

    fs::write(&path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    tracing::debug!("Saved configuration to {}", path.display());
    Ok(())
}
