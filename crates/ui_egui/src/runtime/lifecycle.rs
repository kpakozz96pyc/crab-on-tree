use crate::runtime::CrabOnTreeApp;
use crate::{components, utils::theme};
use crabontree_app::save_config;
use eframe::egui;

impl eframe::App for CrabOnTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        theme::apply_theme(ctx, &self.theme);

        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        let visible_panes = self.get_visible_panes();

        let top_action = components::top_panel::render(
            ctx,
            self.state.current_repo.is_some(),
            self.state.loading,
            &visible_panes,
            &self.available_themes,
            &self.state.config.theme,
        );

        if let components::top_panel::TopPanelAction::TogglePane(pane) = &top_action {
            self.toggle_pane(*pane);
        }

        if let components::top_panel::TopPanelAction::SetTheme(name) = &top_action {
            if let Some((_, new_theme)) = self.available_themes.iter().find(|(id, _)| id == name) {
                self.theme = new_theme.clone();
                self.state.config.theme = name.clone();
                if let Err(e) = save_config(&self.state.config) {
                    tracing::warn!("Failed to save config after theme change: {}", e);
                }
            }
        }

        if let Some(msg) = components::top_panel::action_to_message(&top_action) {
            self.handle_message(msg);
        }

        let error_action =
            components::error_panel::render(ctx, self.state.error.as_ref(), self.theme.error);
        if let Some(msg) = components::error_panel::action_to_message(error_action) {
            self.handle_message(msg);
        }

        if self.show_shortcuts_help {
            components::shortcuts_help::render(ctx, &mut self.show_shortcuts_help);
        }

        if let Some(dialog) = &self.state.checkout_changes_dialog {
            let action = components::checkout_changes_dialog::render(ctx, dialog);
            if let Some(msg) =
                components::checkout_changes_dialog::action_to_message(action, dialog)
            {
                self.handle_message(msg);
            }
            if matches!(
                action,
                components::checkout_changes_dialog::CheckoutChangesAction::Cancel
            ) {
                self.state.checkout_changes_dialog = None;
            }
        }

        if let Some(dialog) = &mut self.state.branch_conflict_dialog {
            let action = components::branch_conflict_dialog::render(ctx, dialog);
            if let Some(msg) =
                components::branch_conflict_dialog::action_to_message(action.clone(), dialog)
            {
                self.handle_message(msg);
            }
            if matches!(
                action,
                components::branch_conflict_dialog::BranchConflictAction::Cancel
            ) {
                self.state.branch_conflict_dialog = None;
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                if self.state.current_repo.is_some() {
                    self.render_repository_view(ui);
                } else {
                    let welcome_action =
                        components::welcome_view::render(ui, &self.state.config.recent_repos);
                    if let Some(msg) = components::welcome_view::action_to_message(welcome_action) {
                        self.handle_message(msg);
                    }
                }
            });
    }
}

impl Drop for CrabOnTreeApp {
    fn drop(&mut self) {
        if let Some(repo) = &self.state.current_repo {
            if let Some(changed_files) = &repo.changed_files {
                if !changed_files.is_commit_view {
                    let repo_key = repo.path.to_string_lossy().to_string();
                    if changed_files.commit_summary.is_empty()
                        && changed_files.commit_description.is_empty()
                    {
                        self.state.config.commit_drafts.remove(&repo_key);
                    } else {
                        self.state.config.commit_drafts.insert(
                            repo_key,
                            crabontree_app::CommitDraft {
                                summary: changed_files.commit_summary.clone(),
                                description: changed_files.commit_description.clone(),
                            },
                        );
                    }
                }
            }
        }

        match serde_json::to_string(&self.dock_state) {
            Ok(layout_json) => {
                self.state.config.dock_layout = Some(layout_json);
                if let Err(e) = save_config(&self.state.config) {
                    tracing::warn!("Failed to save config on exit: {}", e);
                } else {
                    tracing::info!("Saved dock layout to config");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize dock layout: {}", e);
            }
        }
    }
}
