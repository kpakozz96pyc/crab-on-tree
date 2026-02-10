/// Changed files pane for displaying file changes.
///
/// This pane shows staged, unstaged, untracked, and conflicted files
/// organized into collapsible sections.

use crate::widgets::FileRow;
use crabontree_app::{AppMessage, ChangedFilesState};
use eframe::egui;
use std::path::PathBuf;

/// Action to be taken after rendering the changed files pane.
pub enum ChangedFilesAction {
    None,
    SelectFile(PathBuf),
}

/// Renders the changed files pane.
///
/// Returns an action that the caller should handle (e.g., select a file).
pub fn render(ui: &mut egui::Ui, files: &ChangedFilesState) -> ChangedFilesAction {
    let mut action = ChangedFilesAction::None;

    // Staged files
    if !files.staged.is_empty() {
        egui::CollapsingHeader::new(format!("Staged ({})", files.staged.len()))
            .id_source("changed_files_staged")
            .default_open(true)
            .show(ui, |ui| {
                for (idx, file) in files.staged.iter().enumerate() {
                    ui.push_id(format!("staged_{}", idx), |ui| {
                        let is_selected = files.selected_file.as_ref() == Some(&file.path);
                        if FileRow::new(&file.path, &file.status, is_selected).render(ui) {
                            action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                    });
                }
            });
        ui.add_space(5.0);
    }

    // Unstaged files
    if !files.unstaged.is_empty() {
        egui::CollapsingHeader::new(format!("Unstaged ({})", files.unstaged.len()))
            .id_source("changed_files_unstaged")
            .default_open(true)
            .show(ui, |ui| {
                for (idx, file) in files.unstaged.iter().enumerate() {
                    ui.push_id(format!("unstaged_{}", idx), |ui| {
                        let is_selected = files.selected_file.as_ref() == Some(&file.path);
                        if FileRow::new(&file.path, &file.status, is_selected).render(ui) {
                            action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                    });
                }
            });
        ui.add_space(5.0);
    }

    // Untracked files
    if !files.untracked.is_empty() {
        egui::CollapsingHeader::new(format!("Untracked ({})", files.untracked.len()))
            .id_source("changed_files_untracked")
            .default_open(true)
            .show(ui, |ui| {
                for (idx, file) in files.untracked.iter().enumerate() {
                    ui.push_id(format!("untracked_{}", idx), |ui| {
                        let is_selected = files.selected_file.as_ref() == Some(&file.path);
                        if FileRow::new(&file.path, &file.status, is_selected).render(ui) {
                            action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                    });
                }
            });
        ui.add_space(5.0);
    }

    // Conflicted files
    if !files.conflicted.is_empty() {
        egui::CollapsingHeader::new(format!("Conflicted ({})", files.conflicted.len()))
            .id_source("changed_files_conflicted")
            .default_open(true)
            .show(ui, |ui| {
                for (idx, file) in files.conflicted.iter().enumerate() {
                    ui.push_id(format!("conflicted_{}", idx), |ui| {
                        let is_selected = files.selected_file.as_ref() == Some(&file.path);
                        if FileRow::new(&file.path, &file.status, is_selected).render(ui) {
                            action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                    });
                }
            });
    }

    action
}

/// Converts a ChangedFilesAction to an AppMessage.
pub fn action_to_message(action: ChangedFilesAction) -> Option<AppMessage> {
    match action {
        ChangedFilesAction::None => None,
        ChangedFilesAction::SelectFile(path) => Some(AppMessage::ChangedFileSelected(path)),
    }
}
