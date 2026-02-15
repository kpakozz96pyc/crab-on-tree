/// Dialog for handling uncommitted changes before branch checkout.

use crabontree_app::{AppMessage, CheckoutChangesDialog};
use eframe::egui;

#[derive(Clone, Copy)]
pub enum CheckoutChangesAction {
    None,
    Stash,
    Discard,
    Cancel,
}

pub fn render(ctx: &egui::Context, dialog: &CheckoutChangesDialog) -> CheckoutChangesAction {
    let mut action = CheckoutChangesAction::None;

    egui::Window::new("Uncommitted Changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("⚠ You have uncommitted changes.");
                ui.add_space(10.0);

                let branch_type = if dialog.is_remote { "remote" } else { "local" };
                ui.label(format!(
                    "You're about to switch to {} branch: {}",
                    branch_type, dialog.branch_name
                ));

                ui.add_space(10.0);
                ui.label("What would you like to do with your uncommitted changes?");
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button("💾 Stash & Switch").clicked() {
                        action = CheckoutChangesAction::Stash;
                    }

                    if ui.button("🗑 Discard & Switch").clicked() {
                        action = CheckoutChangesAction::Discard;
                    }

                    if ui.button("✖ Cancel").clicked() {
                        action = CheckoutChangesAction::Cancel;
                    }
                });
            });
        });

    action
}

pub fn action_to_message(
    action: CheckoutChangesAction,
    dialog: &CheckoutChangesDialog,
) -> Option<AppMessage> {
    match action {
        CheckoutChangesAction::None => None,
        CheckoutChangesAction::Stash => Some(AppMessage::CheckoutWithStash {
            branch_name: dialog.branch_name.clone(),
            is_remote: dialog.is_remote,
        }),
        CheckoutChangesAction::Discard => Some(AppMessage::CheckoutWithDiscard {
            branch_name: dialog.branch_name.clone(),
            is_remote: dialog.is_remote,
        }),
        CheckoutChangesAction::Cancel => {
            // Just close the dialog by clearing the state
            // This will be handled by the reducer
            None
        }
    }
}
