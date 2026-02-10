/// Top panel component for the main toolbar.
///
/// Displays the application name, repository controls (Open, Refresh, Close),
/// and a loading spinner when operations are in progress.

use crabontree_app::AppMessage;
use eframe::egui;
use std::path::PathBuf;

/// Action to be taken after rendering the top panel.
pub enum TopPanelAction {
    None,
    OpenRepo(PathBuf),
    RefreshRepo,
    CloseRepo,
}

/// Renders the top panel toolbar.
///
/// Returns an action that the caller should handle.
pub fn render(ctx: &egui::Context, has_repo: bool, loading: bool) -> TopPanelAction {
    let mut action = TopPanelAction::None;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
            }

            // Loading spinner
            if loading {
                ui.add_space(10.0);
                ui.spinner();
            }
        });
    });

    action
}

/// Converts a TopPanelAction to an AppMessage.
pub fn action_to_message(action: TopPanelAction) -> Option<AppMessage> {
    match action {
        TopPanelAction::None => None,
        TopPanelAction::OpenRepo(path) => Some(AppMessage::OpenRepoRequested(path)),
        TopPanelAction::RefreshRepo => Some(AppMessage::RefreshRepo),
        TopPanelAction::CloseRepo => Some(AppMessage::CloseRepo),
    }
}
