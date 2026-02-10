/// Commit history pane for displaying the commit list.
///
/// This pane shows the working directory as a special "commit" followed
/// by the repository's commit history.

use crate::widgets;
use crabontree_app::{AppMessage, Commit, WORKING_DIR_HASH};
use eframe::egui;

/// Action to be taken after rendering the commit history pane.
pub enum CommitHistoryAction {
    None,
    LoadHistory,
    SelectCommit(String),
    DeselectCommit,
}

/// Renders the commit history pane.
///
/// Returns an action that the caller should handle (e.g., load history, select/deselect commit).
pub fn render(
    ui: &mut egui::Ui,
    commits: &[Commit],
    selected_commit: Option<&String>,
    has_working_dir_changes: bool,
) -> CommitHistoryAction {
    if commits.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            if ui.button("Load Commit History").clicked() {
                return CommitHistoryAction::LoadHistory;
            }
            CommitHistoryAction::None
        })
        .inner
    } else {
        let mut action = CommitHistoryAction::None;

        // Show Working Directory as first commit (0000000)
        ui.push_id("working_directory", |ui| {
            let is_selected = selected_commit == Some(&WORKING_DIR_HASH.to_string());
            let status_indicator = if has_working_dir_changes { " *" } else { "" };
            let text = format!("0000000 - Working Directory{}", status_indicator);

            if widgets::selectable_row(ui, text, is_selected) {
                action = if is_selected {
                    CommitHistoryAction::DeselectCommit
                } else {
                    CommitHistoryAction::SelectCommit(WORKING_DIR_HASH.to_string())
                };
            }
        });

        // Show regular commits
        for (idx, commit) in commits.iter().enumerate() {
            ui.push_id(format!("commit_{}", idx), |ui| {
                let is_selected = selected_commit == Some(&commit.hash);
                let text = format!("{} - {}", &commit.hash[..7], commit.message_summary);

                if widgets::selectable_row(ui, text, is_selected) {
                    action = if is_selected {
                        CommitHistoryAction::DeselectCommit
                    } else {
                        CommitHistoryAction::SelectCommit(commit.hash.clone())
                    };
                }
            });
        }

        action
    }
}

/// Converts a CommitHistoryAction to an AppMessage.
pub fn action_to_message(action: CommitHistoryAction) -> Option<AppMessage> {
    match action {
        CommitHistoryAction::None => None,
        CommitHistoryAction::LoadHistory => Some(AppMessage::LoadCommitHistoryRequested),
        CommitHistoryAction::SelectCommit(hash) => Some(AppMessage::CommitSelected(hash)),
        CommitHistoryAction::DeselectCommit => Some(AppMessage::CommitDeselected),
    }
}
