//! Welcome view component shown when no repository is open.
//!
//! Displays a welcome message, Open button, and list of recent repositories.

use crabontree_app::AppMessage;
use eframe::egui;
use std::path::PathBuf;

/// Action to be taken after rendering the welcome view.
pub enum WelcomeViewAction {
    None,
    OpenRepo(PathBuf),
}

/// Renders the welcome view.
///
/// Returns an action that the caller should handle.
pub fn render(ui: &mut egui::Ui, recent_repos: &[PathBuf]) -> WelcomeViewAction {
    let mut action = WelcomeViewAction::None;

    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        ui.heading("Welcome to CrabOnTree");
        ui.add_space(20.0);
        ui.label("A Git GUI written in Rust with gitoxide and egui");
        ui.add_space(40.0);

        if ui.button("📂 Open a Repository").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                action = WelcomeViewAction::OpenRepo(path);
            }
        }

        ui.add_space(40.0);

        if !recent_repos.is_empty() {
            ui.heading("Recent Repositories");
            ui.add_space(10.0);

            for repo_path in recent_repos {
                if ui.button(repo_path.display().to_string()).clicked() {
                    action = WelcomeViewAction::OpenRepo(repo_path.clone());
                }
            }
        }
    });

    action
}

/// Converts a WelcomeViewAction to an AppMessage.
pub fn action_to_message(action: WelcomeViewAction) -> Option<AppMessage> {
    match action {
        WelcomeViewAction::None => None,
        WelcomeViewAction::OpenRepo(path) => Some(AppMessage::OpenRepoRequested(path)),
    }
}
