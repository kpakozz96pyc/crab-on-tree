use crate::widgets::{FileRow, FileRowInteraction};
use crabontree_app::{AppMessage, ChangedFilesState};
use eframe::egui;
use std::path::PathBuf;

pub enum ChangedFilesAction {
    None,
    SelectFile(PathBuf),
    ToggleStage { path: PathBuf, is_staged: bool },
}

fn render_section(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    files: &[crabontree_app::WorkingDirFile],
    selected_file: Option<&PathBuf>,
    is_commit_view: bool,
    action: &mut ChangedFilesAction,
) {
    if files.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("{} ({})", title, files.len()))
        .id_source(id)
        .default_open(true)
        .show(ui, |ui| {
            for (idx, file) in files.iter().enumerate() {
                ui.push_id(format!("{}_{}", id, idx), |ui| {
                    let is_selected = selected_file == Some(&file.path);
                    let interaction = FileRow::new(&file.path, &file.status, is_selected).render(ui);

                    match interaction {
                        FileRowInteraction::SingleClick => {
                            *action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                        FileRowInteraction::DoubleClick => {
                            // Only allow staging/unstaging in working directory view
                            if !is_commit_view {
                                *action = ChangedFilesAction::ToggleStage {
                                    path: file.path.clone(),
                                    is_staged: file.is_staged,
                                };
                            }
                        }
                        FileRowInteraction::None => {}
                    }
                });
            }
        });
}

pub fn render(ui: &mut egui::Ui, files: &ChangedFilesState) -> ChangedFilesAction {
    let mut action = ChangedFilesAction::None;
    let selected_file = files.selected_file.as_ref();

    // Render commit message section if available
    if !files.commit_message.is_empty() {
        egui::CollapsingHeader::new("Commit Message")
            .id_source("changed_files_commit_message")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut files.commit_message.as_str())
                                .desired_width(f32::INFINITY)
                                .interactive(false)
                                .font(egui::TextStyle::Monospace)
                        );
                    });
            });
        ui.add_space(5.0);
    }

    if !files.staged.is_empty() {
        render_section(
            ui,
            "changed_files_staged",
            "Staged",
            &files.staged,
            selected_file,
            files.is_commit_view,
            &mut action,
        );
        ui.add_space(5.0);
    }

    if !files.unstaged.is_empty() {
        render_section(
            ui,
            "changed_files_unstaged",
            "Unstaged",
            &files.unstaged,
            selected_file,
            files.is_commit_view,
            &mut action,
        );
        ui.add_space(5.0);
    }

    if !files.untracked.is_empty() {
        render_section(
            ui,
            "changed_files_untracked",
            "Untracked",
            &files.untracked,
            selected_file,
            files.is_commit_view,
            &mut action,
        );
        ui.add_space(5.0);
    }

    if !files.conflicted.is_empty() {
        render_section(
            ui,
            "changed_files_conflicted",
            "Conflicted",
            &files.conflicted,
            selected_file,
            files.is_commit_view,
            &mut action,
        );
    }

    action
}

pub fn action_to_message(action: ChangedFilesAction) -> Option<AppMessage> {
    match action {
        ChangedFilesAction::None => None,
        ChangedFilesAction::SelectFile(path) => Some(AppMessage::ChangedFileSelected(path)),
        ChangedFilesAction::ToggleStage { path, is_staged } => {
            if is_staged {
                Some(AppMessage::UnstageFileRequested(path))
            } else {
                Some(AppMessage::StageFileRequested(path))
            }
        }
    }
}
