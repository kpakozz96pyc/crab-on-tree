//! Keyboard handling utilities for the application.
//!
//! Handles pane navigation, shortcuts, and keyboard events.

use crabontree_app::{AppMessage, RepoState, WORKING_DIR_HASH};
use eframe::egui;
use std::path::PathBuf;

const PANE_COMMIT_HISTORY: usize = 0;
const PANE_BRANCHES: usize = 1;
const PANE_CHANGED_FILES: usize = 2;
const PANE_COUNT: usize = 4;

/// Keyboard action to be taken.
pub enum KeyboardAction {
    None,
    SetActivePane,
    RefreshRepo,
    ToggleHelp,
    SelectCommit(String),
    SelectFile(PathBuf),
}

/// Handles keyboard shortcuts and returns an action.
///
/// Returns both a keyboard action and the new active pane state.
pub fn handle_shortcuts(
    ui: &egui::Ui,
    current_pane: usize,
    repo: Option<&RepoState>,
) -> (KeyboardAction, usize) {
    let any_text_focused = ui.memory(|mem| mem.focused().is_some());

    if any_text_focused {
        return (KeyboardAction::None, current_pane);
    }

    // Pane selection: 1, 2, 3, 4
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        return (KeyboardAction::SetActivePane, 0);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
        return (KeyboardAction::SetActivePane, 1);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
        return (KeyboardAction::SetActivePane, 2);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num4)) {
        return (KeyboardAction::SetActivePane, 3);
    }

    // Tab to cycle through panes
    if ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
        let new_pane = (current_pane + 1) % PANE_COUNT;
        return (KeyboardAction::SetActivePane, new_pane);
    }

    // Shift+Tab to cycle backward
    if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift) {
        let new_pane = if current_pane == 0 {
            PANE_COUNT - 1
        } else {
            current_pane - 1
        };
        return (KeyboardAction::SetActivePane, new_pane);
    }

    // Left/Right to switch active pane.
    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
        let new_pane = (current_pane + 1) % PANE_COUNT;
        return (KeyboardAction::SetActivePane, new_pane);
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
        let new_pane = if current_pane == 0 {
            PANE_COUNT - 1
        } else {
            current_pane - 1
        };
        return (KeyboardAction::SetActivePane, new_pane);
    }

    // Up/Down to navigate selected item in active pane.
    if let Some(repo) = repo {
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            return navigate_down(current_pane, repo)
                .map_or((KeyboardAction::None, current_pane), |action| {
                    (action, current_pane)
                });
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            return navigate_up(current_pane, repo)
                .map_or((KeyboardAction::None, current_pane), |action| {
                    (action, current_pane)
                });
        }
    }

    // 'r' key to refresh
    if ui.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.ctrl) {
        return (KeyboardAction::RefreshRepo, current_pane);
    }

    // '?' to show help
    if ui.input(|i| {
        i.key_pressed(egui::Key::Questionmark)
            || (i.key_pressed(egui::Key::Slash) && i.modifiers.shift)
    }) {
        return (KeyboardAction::ToggleHelp, current_pane);
    }

    (KeyboardAction::None, current_pane)
}

/// Converts a KeyboardAction to an AppMessage.
pub fn action_to_message(action: KeyboardAction) -> Option<AppMessage> {
    match action {
        KeyboardAction::None | KeyboardAction::SetActivePane => None,
        KeyboardAction::RefreshRepo => Some(AppMessage::RefreshRepo),
        KeyboardAction::ToggleHelp => None, // Handled in main app state
        KeyboardAction::SelectCommit(hash) => Some(AppMessage::CommitSelected(hash)),
        KeyboardAction::SelectFile(path) => Some(AppMessage::ChangedFileSelected(path)),
    }
}

fn navigate_down(current_pane: usize, repo: &RepoState) -> Option<KeyboardAction> {
    match current_pane {
        PANE_COMMIT_HISTORY => next_commit(repo, true).map(KeyboardAction::SelectCommit),
        PANE_CHANGED_FILES => next_file(repo, true).map(KeyboardAction::SelectFile),
        PANE_BRANCHES | _ => None,
    }
}

fn navigate_up(current_pane: usize, repo: &RepoState) -> Option<KeyboardAction> {
    match current_pane {
        PANE_COMMIT_HISTORY => next_commit(repo, false).map(KeyboardAction::SelectCommit),
        PANE_CHANGED_FILES => next_file(repo, false).map(KeyboardAction::SelectFile),
        PANE_BRANCHES | _ => None,
    }
}

fn next_commit(repo: &RepoState, forward: bool) -> Option<String> {
    let mut commits = Vec::with_capacity(repo.commits.len() + 1);
    commits.push(WORKING_DIR_HASH.to_string());
    commits.extend(repo.commits.iter().map(|c| c.hash.clone()));
    step_selection(&commits, repo.selected_commit.as_ref(), forward)
}

fn next_file(repo: &RepoState, forward: bool) -> Option<PathBuf> {
    let files = repo.changed_files.as_ref()?;

    let mut paths = Vec::new();
    paths.extend(files.staged.iter().map(|f| f.path.clone()));
    paths.extend(files.unstaged.iter().map(|f| f.path.clone()));
    paths.extend(files.untracked.iter().map(|f| f.path.clone()));
    paths.extend(files.conflicted.iter().map(|f| f.path.clone()));

    step_selection(&paths, files.selected_file.as_ref(), forward)
}

fn step_selection<T: Clone + PartialEq>(
    items: &[T],
    current: Option<&T>,
    forward: bool,
) -> Option<T> {
    if items.is_empty() {
        return None;
    }

    let next_idx = match current.and_then(|c| items.iter().position(|i| i == c)) {
        Some(idx) => {
            if forward {
                (idx + 1).min(items.len() - 1)
            } else {
                idx.saturating_sub(1)
            }
        }
        None => {
            if forward {
                0
            } else {
                items.len() - 1
            }
        }
    };

    Some(items[next_idx].clone())
}
