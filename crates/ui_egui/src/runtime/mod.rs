use crate::panes;
use crabontree_app::{load_config, reduce, themes_dir, AppState, JobExecutor};
use crabontree_ui_core::Theme;
use egui_dock::{DockState, NodeIndex};

mod dock;
mod effects;
mod lifecycle;

pub(crate) struct CrabOnTreeApp {
    pub(crate) state: AppState,
    pub(crate) executor: JobExecutor,
    pub(crate) message_rx: tokio::sync::mpsc::Receiver<crabontree_app::AppMessage>,
    pub(crate) theme: Theme,
    pub(crate) available_themes: Vec<(String, Theme)>,
    pub(crate) show_shortcuts_help: bool,
    pub(crate) active_pane: usize,
    pub(crate) dock_state: DockState<panes::Pane>,
    pub(crate) saved_dock_states: std::collections::HashMap<panes::Pane, DockState<panes::Pane>>,
}

impl CrabOnTreeApp {
    /// Creates the default 4-pane dock layout
    /// Layout: [CommitHistory, Branches] | ChangedFiles | DiffViewer
    pub(crate) fn create_default_dock_layout() -> DockState<panes::Pane> {
        let mut dock_state =
            DockState::new(vec![panes::Pane::CommitHistory, panes::Pane::Branches]);

        let [left_node, _diff_node] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.70,
            vec![panes::Pane::DiffViewer],
        );

        let _changed_node = dock_state.main_surface_mut().split_right(
            left_node,
            0.57,
            vec![panes::Pane::ChangedFiles],
        );

        dock_state
    }

    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let available_themes = load_all_themes();
        let theme = available_themes
            .iter()
            .find(|(id, _)| id == &config.theme)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(Theme::fallback);
        let (executor, message_rx) = JobExecutor::new();

        let dock_state = if let Some(layout_json) = &config.dock_layout {
            match serde_json::from_str(layout_json) {
                Ok(layout) => {
                    tracing::info!("Restored dock layout from config");
                    layout
                }
                Err(e) => {
                    tracing::warn!("Failed to parse dock layout: {}, using default", e);
                    Self::create_default_dock_layout()
                }
            }
        } else {
            tracing::info!("No saved dock layout, using default");
            Self::create_default_dock_layout()
        };

        Self {
            available_themes,
            state: AppState {
                current_repo: None,
                loading: false,
                committing: false,
                error: None,
                config,
                staging_progress: None,
                checkout_changes_dialog: None,
                branch_conflict_dialog: None,
            },
            executor,
            message_rx,
            theme,
            show_shortcuts_help: false,
            active_pane: 0,
            dock_state,
            saved_dock_states: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn poll_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            self.handle_message(msg);
        }
    }

    pub(crate) fn handle_message(&mut self, message: crabontree_app::AppMessage) {
        let effect = reduce(&mut self.state, message);
        self.execute_effect(effect);
    }
}

/// Loads all available themes: built-ins (embedded) followed by any `.toml` files
/// found in the user themes directory (`<config_dir>/themes/`).
///
/// Built-in ids are never overridden by user files with the same stem name.
fn load_all_themes() -> Vec<(String, Theme)> {
    let builtin_ids = [
        "dark",
        "light",
        "jetbrains",
        "visual_studio",
        "crema",
        "ide-like",
    ];
    let mut themes: Vec<(String, Theme)> = builtin_ids
        .iter()
        .filter_map(|&id| Theme::by_name(id).map(|t| (id.to_string(), t)))
        .collect();

    let Some(dir) = themes_dir() else {
        return themes;
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return themes;
    };

    let mut user_entries: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    user_entries.sort_by_key(|e| e.file_name());

    for entry in user_entries {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
            continue;
        };
        if themes.iter().any(|(id, _)| id == &stem) {
            continue; // don't override built-ins
        }
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|s| toml::from_str::<Theme>(&s).map_err(|e| e.to_string()))
        {
            Ok(theme) => {
                tracing::info!("Loaded user theme '{}'", stem);
                themes.push((stem, theme));
            }
            Err(e) => tracing::warn!("Skipping theme '{}': {}", stem, e),
        }
    }

    themes
}
