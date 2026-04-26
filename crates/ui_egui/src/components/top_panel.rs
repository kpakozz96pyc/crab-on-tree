/// Top panel component for the main toolbar.
///
/// Displays the application name, repository controls (Open, Refresh, Close),
/// and a loading spinner when operations are in progress.
use crabontree_app::AppMessage;
use crabontree_ui_core::Theme;
use eframe::egui;
use std::path::PathBuf;

/// Action to be taken after rendering the top panel.
pub enum TopPanelAction {
    None,
    OpenRepo(PathBuf),
    RefreshRepo,
    CloseRepo,
    TogglePane(crate::panes::Pane),
    ResetLayout,
    SetTheme(String),
}

/// Renders the top panel toolbar.
///
/// Returns an action that the caller should handle.
pub fn render(
    ctx: &egui::Context,
    has_repo: bool,
    loading: bool,
    visible_panes: &[crate::panes::Pane],
    available_themes: &[(String, Theme)],
    current_theme: &str,
) -> TopPanelAction {
    let mut action = TopPanelAction::None;

    egui::TopBottomPanel::top("top_panel")
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .inner_margin(egui::Margin::symmetric(8.0, 5.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("CrabOnTree");

                ui.add_space(20.0);

                if ui.button("📂 Open Repository").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        action = TopPanelAction::OpenRepo(path);
                    }
                }

                if has_repo {
                    if ui.button("🔄 Refresh").clicked() {
                        action = TopPanelAction::RefreshRepo;
                    }

                    if ui.button("✖ Close").clicked() {
                        action = TopPanelAction::CloseRepo;
                    }

                    // Pane visibility toggles (dropdown)
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.menu_button("Panels", |ui| {
                        let commit_visible =
                            visible_panes.contains(&crate::panes::Pane::CommitHistory);
                        let commit_text = if commit_visible {
                            "★ History"
                        } else {
                            "☆ History"
                        };
                        if ui.button(commit_text).clicked() {
                            action = TopPanelAction::TogglePane(crate::panes::Pane::CommitHistory);
                            ui.close_menu();
                        }

                        let branches_visible =
                            visible_panes.contains(&crate::panes::Pane::Branches);
                        let branches_text = if branches_visible {
                            "★ Branches"
                        } else {
                            "☆ Branches"
                        };
                        if ui.button(branches_text).clicked() {
                            action = TopPanelAction::TogglePane(crate::panes::Pane::Branches);
                            ui.close_menu();
                        }

                        let files_visible =
                            visible_panes.contains(&crate::panes::Pane::ChangedFiles);
                        let files_text = if files_visible {
                            "★ Files"
                        } else {
                            "☆ Files"
                        };
                        if ui.button(files_text).clicked() {
                            action = TopPanelAction::TogglePane(crate::panes::Pane::ChangedFiles);
                            ui.close_menu();
                        }

                        let diff_visible =
                            visible_panes.contains(&crate::panes::Pane::DiffViewer);
                        let diff_text = if diff_visible { "★ Diff" } else { "☆ Diff" };
                        if ui.button(diff_text).clicked() {
                            action = TopPanelAction::TogglePane(crate::panes::Pane::DiffViewer);
                            ui.close_menu();
                        }
                    });

                    if ui.button("Reset Layout").clicked() {
                        action = TopPanelAction::ResetLayout;
                    }
                }

                // Theme selector
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if loading {
                        ui.spinner();
                        ui.add_space(10.0);
                    }

                    ui.menu_button("🎨 Theme", |ui| {
                        for (id, theme) in available_themes {
                            let label = if theme.name.is_empty() {
                                id.as_str()
                            } else {
                                theme.name.as_str()
                            };
                            if ui
                                .selectable_label(current_theme == id.as_str(), label)
                                .clicked()
                            {
                                action = TopPanelAction::SetTheme(id.clone());
                                ui.close_menu();
                            }
                        }
                    });
                });
            });
        });

    action
}

/// Converts a TopPanelAction to an AppMessage.
pub fn action_to_message(action: &TopPanelAction) -> Option<AppMessage> {
    match action {
        TopPanelAction::None => None,
        TopPanelAction::OpenRepo(path) => Some(AppMessage::OpenRepoRequested(path.clone())),
        TopPanelAction::RefreshRepo => Some(AppMessage::RefreshRepo),
        TopPanelAction::CloseRepo => Some(AppMessage::CloseRepo),
        TopPanelAction::TogglePane(_) => None,  // Handled separately in lifecycle.rs
        TopPanelAction::ResetLayout => None,   // Handled separately in lifecycle.rs
        TopPanelAction::SetTheme(_) => None,   // Handled separately in lifecycle.rs
    }
}
