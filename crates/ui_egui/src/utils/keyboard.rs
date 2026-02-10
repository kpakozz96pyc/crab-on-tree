/// Keyboard handling utilities for the application.
///
/// Handles pane navigation, shortcuts, and keyboard events.

use crabontree_app::AppMessage;
use eframe::egui;

/// Active panel for keyboard navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    BranchTree,
    ChangedFiles,
    FileViewer,
}

/// Keyboard action to be taken.
pub enum KeyboardAction {
    None,
    SetActivePane,
    RefreshRepo,
    ToggleHelp,
}

/// Handles keyboard shortcuts and returns an action.
///
/// Returns both a keyboard action and the new active pane state.
pub fn handle_shortcuts(
    ui: &egui::Ui,
    current_pane: usize,
    current_panel: ActivePanel,
) -> (KeyboardAction, usize, ActivePanel) {
    let any_text_focused = ui.memory(|mem| mem.focused().is_some());

    if any_text_focused {
        return (KeyboardAction::None, current_pane, current_panel);
    }

    // Pane selection: 1, 2, 3
    if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
        return (
            KeyboardAction::SetActivePane,
            0,
            ActivePanel::BranchTree,
        );
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
        return (
            KeyboardAction::SetActivePane,
            1,
            ActivePanel::ChangedFiles,
        );
    }
    if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
        return (
            KeyboardAction::SetActivePane,
            2,
            ActivePanel::FileViewer,
        );
    }

    // Tab to cycle through panes
    if ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
        let new_pane = (current_pane + 1) % 3;
        let new_panel = match new_pane {
            0 => ActivePanel::BranchTree,
            1 => ActivePanel::ChangedFiles,
            _ => ActivePanel::FileViewer,
        };
        return (
            KeyboardAction::SetActivePane,
            new_pane,
            new_panel,
        );
    }

    // Shift+Tab to cycle backward
    if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift) {
        let new_pane = if current_pane == 0 {
            2
        } else {
            current_pane - 1
        };
        let new_panel = match new_pane {
            0 => ActivePanel::BranchTree,
            1 => ActivePanel::ChangedFiles,
            _ => ActivePanel::FileViewer,
        };
        return (
            KeyboardAction::SetActivePane,
            new_pane,
            new_panel,
        );
    }

    // 'r' key to refresh
    if ui.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.ctrl) {
        return (KeyboardAction::RefreshRepo, current_pane, current_panel);
    }

    // '?' to show help
    if ui.input(|i| {
        i.key_pressed(egui::Key::Questionmark) || (i.key_pressed(egui::Key::Slash) && i.modifiers.shift)
    }) {
        return (KeyboardAction::ToggleHelp, current_pane, current_panel);
    }

    (KeyboardAction::None, current_pane, current_panel)
}

/// Converts a KeyboardAction to an AppMessage.
pub fn action_to_message(action: KeyboardAction) -> Option<AppMessage> {
    match action {
        KeyboardAction::None | KeyboardAction::SetActivePane => None,
        KeyboardAction::RefreshRepo => Some(AppMessage::RefreshRepo),
        KeyboardAction::ToggleHelp => None, // Handled in main app state
    }
}
