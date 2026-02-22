use crate::widgets::{FileRow, FileRowInteraction};
use crabontree_app::{AppMessage, ChangedFilesState};
use eframe::egui;
use std::path::PathBuf;

pub enum ChangedFilesAction {
    None,
    SelectFile(PathBuf),
    SelectFileWithModifiers {
        path: PathBuf,
        ctrl: bool,
        shift: bool,
    },
    ToggleStage {
        path: PathBuf,
        is_staged: bool,
    },
    StageSelectedFiles,
    UnstageSelectedFiles,
    CommitSummaryUpdated(String),
    CommitDescriptionUpdated(String),
    AmendLastCommitToggled(bool),
    PushAfterCommitToggled(bool),
    CommitChangesRequested {
        summary: String,
        description: String,
        amend: bool,
        push: bool,
    },
    RevertFile(PathBuf),
    OpenInEditor(PathBuf),
    OpenFolder(PathBuf),
}

fn render_section(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    files: &[crabontree_app::WorkingDirFile],
    selected_files: &std::collections::HashSet<PathBuf>,
    is_commit_view: bool,
    show_context_menu: bool,
    action: &mut ChangedFilesAction,
) {
    egui::CollapsingHeader::new(format!("{} ({})", title, files.len()))
        .id_source(id)
        .default_open(true)
        .show(ui, |ui| {
            if files.is_empty() {
                ui.label(egui::RichText::new("No files").weak());
                return;
            }

            for (idx, file) in files.iter().enumerate() {
                let (interaction, row_response) = ui
                    .push_id(format!("{}_{}", id, idx), |ui| {
                        let is_selected = selected_files.contains(&file.path);
                        FileRow::new(&file.path, &file.status, is_selected).render(ui)
                    })
                    .inner;

                match interaction {
                    FileRowInteraction::SingleClick { ctrl, shift } => {
                        if ctrl || shift {
                            *action = ChangedFilesAction::SelectFileWithModifiers {
                                path: file.path.clone(),
                                ctrl,
                                shift,
                            };
                        } else {
                            *action = ChangedFilesAction::SelectFile(file.path.clone());
                        }
                    }
                    FileRowInteraction::DoubleClick => {
                        if !is_commit_view {
                            *action = ChangedFilesAction::ToggleStage {
                                path: file.path.clone(),
                                is_staged: file.is_staged,
                            };
                        }
                    }
                    FileRowInteraction::None => {}
                }

                if show_context_menu {
                    let path = file.path.clone();
                    row_response.context_menu(|ui| {
                        if ui.button("Revert Changes").clicked() {
                            *action = ChangedFilesAction::RevertFile(path.clone());
                            ui.close_menu();
                        }
                        if ui.button("Stage File").clicked() {
                            *action = ChangedFilesAction::ToggleStage {
                                path: path.clone(),
                                is_staged: false,
                            };
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Open in External Editor").clicked() {
                            *action = ChangedFilesAction::OpenInEditor(path.clone());
                            ui.close_menu();
                        }
                        if ui.button("Open Folder").clicked() {
                            *action = ChangedFilesAction::OpenFolder(path.clone());
                            ui.close_menu();
                        }
                    });
                }
            }
        });
}

pub fn render(ui: &mut egui::Ui, files: &ChangedFilesState, loading: bool) -> ChangedFilesAction {
    let mut action = ChangedFilesAction::None;

    // Handle Enter key for staging/unstaging selected files
    if !files.is_commit_view
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !files.selected_files.is_empty()
    {
        // Determine if we should stage or unstage based on where the files are
        let has_unstaged = files.selected_files.iter().any(|path| {
            files.unstaged.iter().any(|f| &f.path == path)
                || files.untracked.iter().any(|f| &f.path == path)
        });

        if has_unstaged {
            action = ChangedFilesAction::StageSelectedFiles;
        } else {
            action = ChangedFilesAction::UnstageSelectedFiles;
        }
    }

    if files.is_commit_view {
        // Commit view: commit message and changed files share one scroll area.
        egui::ScrollArea::vertical()
            .id_source("changed_files_scroll")
            .show(ui, |ui| {
                if !files.commit_message.is_empty() {
                    egui::CollapsingHeader::new("Commit Message")
                        .id_source("changed_files_commit_message")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut files.commit_message.as_str())
                                    .desired_width(f32::INFINITY)
                                    .interactive(false)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                    ui.add_space(5.0);
                }

                render_section(
                    ui,
                    "changed_files_staged",
                    "Staged",
                    &files.staged,
                    &files.selected_files,
                    true,
                    false,
                    &mut action,
                );
                render_section(
                    ui,
                    "changed_files_unstaged",
                    "Unstaged",
                    &files.unstaged,
                    &files.selected_files,
                    true,
                    false,
                    &mut action,
                );
                render_section(
                    ui,
                    "changed_files_untracked",
                    "Untracked",
                    &files.untracked,
                    &files.selected_files,
                    true,
                    false,
                    &mut action,
                );
                render_section(
                    ui,
                    "changed_files_conflicted",
                    "Conflicted",
                    &files.conflicted,
                    &files.selected_files,
                    true,
                    false,
                    &mut action,
                );
            });
    } else {
        // Working directory view:
        // commit area is pinned to the bottom at fixed height,
        // file list consumes all remaining height above it.
        const COMMIT_PANEL_HEIGHT: f32 = 220.0;
        const SEPARATOR_HEIGHT: f32 = 8.0;
        let pane_rect = ui.available_rect_before_wrap();
        ui.allocate_rect(pane_rect, egui::Sense::hover());

        let commit_height = COMMIT_PANEL_HEIGHT.min(pane_rect.height());
        let commit_top = pane_rect.bottom() - commit_height;
        let list_bottom = (commit_top - SEPARATOR_HEIGHT).max(pane_rect.top());

        let list_rect =
            egui::Rect::from_min_max(pane_rect.min, egui::pos2(pane_rect.right(), list_bottom));
        let commit_area_rect = egui::Rect::from_min_max(
            egui::pos2(pane_rect.left(), commit_top),
            egui::pos2(pane_rect.right(), pane_rect.bottom()),
        );

        ui.allocate_ui_at_rect(list_rect, |ui| {
            egui::ScrollArea::vertical()
                .id_source("changed_files_scroll")
                .show(ui, |ui| {
                    render_section(
                        ui,
                        "changed_files_staged",
                        "Staged",
                        &files.staged,
                        &files.selected_files,
                        false,
                        false,
                        &mut action,
                    );
                    ui.add_space(5.0);

                    render_section(
                        ui,
                        "changed_files_unstaged",
                        "Unstaged",
                        &files.unstaged,
                        &files.selected_files,
                        false,
                        true, // show context menu for unstaged files
                        &mut action,
                    );
                    ui.add_space(5.0);

                    render_section(
                        ui,
                        "changed_files_untracked",
                        "Untracked",
                        &files.untracked,
                        &files.selected_files,
                        false,
                        false,
                        &mut action,
                    );
                    ui.add_space(5.0);

                    render_section(
                        ui,
                        "changed_files_conflicted",
                        "Conflicted",
                        &files.conflicted,
                        &files.selected_files,
                        false,
                        false,
                        &mut action,
                    );
                });
        });

        // Separator line between file list and commit panel.
        if list_bottom < commit_top {
            let y = list_bottom + (SEPARATOR_HEIGHT * 0.5);
            ui.painter().line_segment(
                [
                    egui::pos2(pane_rect.left(), y),
                    egui::pos2(pane_rect.right(), y),
                ],
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
        }

        let mut commit_ui = ui.child_ui(commit_area_rect, egui::Layout::top_down(egui::Align::Min));

        let has_staged_files = !files.staged.is_empty();

        // Commit summary
        commit_ui.horizontal(|ui| {
            ui.label("Summary:");
            let summary_len = files.commit_summary.chars().count();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", summary_len));
            });
        });

        let mut summary = files.commit_summary.clone();
        let summary_response = commit_ui.add(
            egui::TextEdit::singleline(&mut summary)
                .desired_width(f32::INFINITY)
                .hint_text(
                    egui::RichText::new("Commit summary").color(egui::Color32::from_gray(80)),
                ),
        );
        if summary_response.changed() {
            action = ChangedFilesAction::CommitSummaryUpdated(summary);
        }

        commit_ui.add_space(5.0);

        // Commit description
        commit_ui.label("Description:");
        let mut description = files.commit_description.clone();
        let description_response = commit_ui.add(
            egui::TextEdit::multiline(&mut description)
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .hint_text(
                    egui::RichText::new("Optional description").color(egui::Color32::from_gray(80)),
                ),
        );
        if description_response.changed() {
            action = ChangedFilesAction::CommitDescriptionUpdated(description);
        }

        commit_ui.add_space(5.0);

        // Checkboxes (left column) and Commit button (right side)
        // Declare outside closures so the button can read the up-to-date toggled values
        // even when the checkbox and the button are both interacted with in the same frame.
        let mut amend = files.amend_last_commit;
        let mut push = files.push_after_commit;

        commit_ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.checkbox(&mut amend, "Amend last commit").changed() {
                    action = ChangedFilesAction::AmendLastCommitToggled(amend);
                }
                if ui.checkbox(&mut push, "Push after commit").changed() {
                    action = ChangedFilesAction::PushAfterCommitToggled(push);
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let commit_enabled =
                    has_staged_files && !files.commit_summary.is_empty() && !loading;
                if ui
                    .add_enabled(commit_enabled, egui::Button::new("Commit"))
                    .clicked()
                {
                    action = ChangedFilesAction::CommitChangesRequested {
                        summary: files.commit_summary.clone(),
                        description: files.commit_description.clone(),
                        amend,
                        push,
                    };
                }
            });
        });

        // Draw loading overlay if loading
        if loading {
            let painter = commit_ui.painter();

            // Draw semi-transparent overlay
            painter.rect_filled(commit_area_rect, 0.0, egui::Color32::from_black_alpha(128));

            // Draw spinner and text in the center
            let center = commit_area_rect.center();
            let spinner_rect = egui::Rect::from_center_size(center, egui::vec2(100.0, 50.0));

            commit_ui.allocate_ui_at_rect(spinner_rect, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(5.0);
                    ui.spinner();
                    ui.label(
                        egui::RichText::new("Committing...")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                });
            });
        }
    }

    action
}

pub fn action_to_message(action: ChangedFilesAction) -> Option<AppMessage> {
    match action {
        ChangedFilesAction::None => None,
        ChangedFilesAction::SelectFile(path) => Some(AppMessage::ChangedFileSelected(path)),
        ChangedFilesAction::SelectFileWithModifiers { path, ctrl, shift } => {
            Some(AppMessage::SelectFileWithModifiers { path, ctrl, shift })
        }
        ChangedFilesAction::ToggleStage { path, is_staged } => {
            if is_staged {
                Some(AppMessage::UnstageFileRequested(path))
            } else {
                Some(AppMessage::StageFileRequested(path))
            }
        }
        ChangedFilesAction::StageSelectedFiles => Some(AppMessage::StageSelectedFilesRequested),
        ChangedFilesAction::UnstageSelectedFiles => Some(AppMessage::UnstageSelectedFilesRequested),
        ChangedFilesAction::CommitSummaryUpdated(summary) => {
            Some(AppMessage::CommitSummaryUpdated(summary))
        }
        ChangedFilesAction::CommitDescriptionUpdated(description) => {
            Some(AppMessage::CommitDescriptionUpdated(description))
        }
        ChangedFilesAction::AmendLastCommitToggled(amend) => {
            Some(AppMessage::AmendLastCommitToggled(amend))
        }
        ChangedFilesAction::PushAfterCommitToggled(push) => {
            Some(AppMessage::PushAfterCommitToggled(push))
        }
        ChangedFilesAction::CommitChangesRequested {
            summary,
            description,
            amend,
            push,
        } => Some(AppMessage::CommitChangesRequested {
            summary,
            description,
            amend,
            push,
        }),
        ChangedFilesAction::RevertFile(path) => Some(AppMessage::RevertFileRequested(path)),
        ChangedFilesAction::OpenInEditor(path) => Some(AppMessage::OpenFileInEditorRequested(path)),
        ChangedFilesAction::OpenFolder(path) => Some(AppMessage::OpenFileFolderRequested(path)),
    }
}
