//! Dialog for handling remote branch name conflicts.

use crabontree_app::{AppMessage, BranchConflictDialog};
use eframe::egui;

#[derive(Clone)]
pub enum BranchConflictAction {
    None,
    Override,
    Rename(String),
    Cancel,
}

pub fn render(ctx: &egui::Context, dialog: &mut BranchConflictDialog) -> BranchConflictAction {
    let mut action = BranchConflictAction::None;

    egui::Window::new("Branch Name Conflict")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("⚠ Local branch name already exists.");
                ui.add_space(10.0);

                ui.label(format!("Remote branch: {}", dialog.remote_branch));
                ui.label(format!("Conflicting local branch: {}", dialog.local_name));

                ui.add_space(10.0);
                ui.label("What would you like to do?");
                ui.add_space(20.0);

                // Option 1: Override existing branch
                ui.horizontal(|ui| {
                    if ui.button("🔄 Override existing branch").clicked() {
                        action = BranchConflictAction::Override;
                    }
                    ui.label("(deletes existing local branch)");
                });

                ui.add_space(10.0);

                // Option 2: Rename to a different name
                ui.horizontal(|ui| {
                    ui.label("📝 Rename to:");
                    ui.text_edit_singleline(&mut dialog.new_name_input);
                });

                ui.horizontal(|ui| {
                    if ui.button("Create with new name").clicked()
                        && !dialog.new_name_input.is_empty()
                    {
                        action = BranchConflictAction::Rename(dialog.new_name_input.clone());
                    }
                });

                ui.add_space(20.0);

                // Option 3: Cancel
                if ui.button("✖ Cancel").clicked() {
                    action = BranchConflictAction::Cancel;
                }
            });
        });

    action
}

pub fn action_to_message(
    action: BranchConflictAction,
    dialog: &BranchConflictDialog,
) -> Option<AppMessage> {
    match action {
        BranchConflictAction::None => None,
        BranchConflictAction::Override => Some(AppMessage::CheckoutRemoteOverride {
            remote_branch: dialog.remote_branch.clone(),
            local_name: dialog.local_name.clone(),
        }),
        BranchConflictAction::Rename(new_name) => Some(AppMessage::CheckoutRemoteRename {
            remote_branch: dialog.remote_branch.clone(),
            new_local_name: new_name,
        }),
        BranchConflictAction::Cancel => None,
    }
}
