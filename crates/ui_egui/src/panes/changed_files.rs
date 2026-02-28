use crate::utils::theme::ThemeColors;
use crate::widgets::{render_with_bottom_panel, FileRow, FileRowInteraction};
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
    let is_commit_view = files.is_commit_view;
    let panel_height = if is_commit_view { 230.0 } else { 220.0 };

    // Each closure needs its own local to avoid a double-mutable-borrow.
    // keyboard_action captures Enter-key shortcuts before the closures run.
    let mut keyboard_action = ChangedFilesAction::None;
    let mut list_action = ChangedFilesAction::None;
    let mut panel_action = ChangedFilesAction::None;

    // Handle Enter key for staging/unstaging selected files
    if !is_commit_view
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !files.selected_files.is_empty()
    {
        let has_unstaged = files.selected_files.iter().any(|path| {
            files.unstaged.iter().any(|f| &f.path == path)
                || files.untracked.iter().any(|f| &f.path == path)
        });

        keyboard_action = if has_unstaged {
            ChangedFilesAction::StageSelectedFiles
        } else {
            ChangedFilesAction::UnstageSelectedFiles
        };
    }

    render_with_bottom_panel(
        ui,
        panel_height,
        |ui| {
            egui::ScrollArea::vertical()
                .id_source("changed_files_scroll")
                .show(ui, |ui| {
                    render_section(
                        ui,
                        "changed_files_staged",
                        "Staged",
                        &files.staged,
                        &files.selected_files,
                        is_commit_view,
                        false,
                        &mut list_action,
                    );
                    if !is_commit_view {
                        ui.add_space(5.0);
                    }
                    render_section(
                        ui,
                        "changed_files_unstaged",
                        "Unstaged",
                        &files.unstaged,
                        &files.selected_files,
                        is_commit_view,
                        !is_commit_view, // context menu only in working dir view
                        &mut list_action,
                    );
                    if !is_commit_view {
                        ui.add_space(5.0);
                    }
                    render_section(
                        ui,
                        "changed_files_untracked",
                        "Untracked",
                        &files.untracked,
                        &files.selected_files,
                        is_commit_view,
                        false,
                        &mut list_action,
                    );
                    if !is_commit_view {
                        ui.add_space(5.0);
                    }
                    render_section(
                        ui,
                        "changed_files_conflicted",
                        "Conflicted",
                        &files.conflicted,
                        &files.selected_files,
                        is_commit_view,
                        false,
                        &mut list_action,
                    );
                });
        },
        |ui, panel_rect| {
            if is_commit_view {
                if let Some(info) = &files.commit_info {
                    super::commit_info::render(ui, info, &files.commit_message);
                } else if !files.commit_message.is_empty() {
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
                }
            } else {
                let has_staged_files = !files.staged.is_empty();

                // Commit summary
                ui.horizontal(|ui| {
                    ui.label("Summary:");
                    let summary_len = files.commit_summary.chars().count();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}", summary_len));
                    });
                });

                let mut summary = files.commit_summary.clone();
                let summary_response = ui.add(
                    egui::TextEdit::singleline(&mut summary)
                        .desired_width(f32::INFINITY)
                        .hint_text(
                            egui::RichText::new("Commit summary")
                                .color(ThemeColors::get(ui.ctx()).hint_fg),
                        ),
                );
                if summary_response.changed() {
                    panel_action = ChangedFilesAction::CommitSummaryUpdated(summary);
                }

                ui.add_space(5.0);

                // Commit description
                ui.label("Description:");
                let mut description = files.commit_description.clone();
                let description_response = ui.add(
                    egui::TextEdit::multiline(&mut description)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text(
                            egui::RichText::new("Optional description")
                                .color(ThemeColors::get(ui.ctx()).hint_fg),
                        ),
                );
                if description_response.changed() {
                    panel_action = ChangedFilesAction::CommitDescriptionUpdated(description);
                }

                ui.add_space(5.0);

                // Checkboxes (left column) and Commit button (right side).
                // Declared outside closures so the button can read up-to-date toggled values
                // even when both are interacted with in the same frame.
                let mut amend = files.amend_last_commit;
                let mut push = files.push_after_commit;

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut amend, "Amend last commit").changed() {
                            panel_action = ChangedFilesAction::AmendLastCommitToggled(amend);
                        }
                        if ui.checkbox(&mut push, "Push after commit").changed() {
                            panel_action = ChangedFilesAction::PushAfterCommitToggled(push);
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let commit_enabled =
                            has_staged_files && !files.commit_summary.is_empty() && !loading;
                        if ui
                            .add_enabled(commit_enabled, egui::Button::new("Commit"))
                            .clicked()
                        {
                            panel_action = ChangedFilesAction::CommitChangesRequested {
                                summary: files.commit_summary.clone(),
                                description: files.commit_description.clone(),
                                amend,
                                push,
                            };
                        }
                    });
                });

                // Loading overlay
                if loading {
                    let tc = ThemeColors::get(ui.ctx());
                    let painter = ui.painter();
                    painter.rect_filled(panel_rect, 0.0, tc.overlay_bg);

                    let spinner_rect =
                        egui::Rect::from_center_size(panel_rect.center(), egui::vec2(100.0, 50.0));
                    ui.allocate_ui_at_rect(spinner_rect, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(5.0);
                            ui.spinner();
                            ui.label(
                                egui::RichText::new("Committing...")
                                    .color(tc.overlay_fg)
                                    .strong(),
                            );
                        });
                    });
                }
            }
        },
    );

    match list_action {
        ChangedFilesAction::None => match panel_action {
            ChangedFilesAction::None => keyboard_action,
            other => other,
        },
        other => other,
    }
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
