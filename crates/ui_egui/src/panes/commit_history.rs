use crate::widgets;
use crabontree_app::{AppMessage, Commit, WORKING_DIR_HASH};
use eframe::egui;

pub enum CommitHistoryAction {
    None,
    LoadHistory,
    SelectCommit(String),
    DeselectCommit,
}

pub fn render(
    ui: &mut egui::Ui,
    commits: &[Commit],
    selected_commit: Option<&String>,
    has_working_dir_changes: bool,
    is_focused: bool,
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

        ui.push_id("working_directory", |ui| {
            let is_selected = selected_commit.map(String::as_str) == Some(WORKING_DIR_HASH);
            let is_row_focused = is_focused && is_selected;
            let status_indicator = if has_working_dir_changes { " *" } else { "" };
            let text = format!("0000000 - Working Directory{}", status_indicator);

            let response = widgets::selectable_row(ui, text, is_selected, is_row_focused);
            if response.clicked() {
                action = if is_selected {
                    CommitHistoryAction::DeselectCommit
                } else {
                    CommitHistoryAction::SelectCommit(WORKING_DIR_HASH.to_string())
                };
            }
        });

        for (idx, commit) in commits.iter().enumerate() {
            ui.push_id(format!("commit_{}", idx), |ui| {
                let is_selected = selected_commit == Some(&commit.hash);
                let is_row_focused = is_focused && is_selected;
                let text = format!("{} - {}", &commit.hash[..7], commit.message_summary);

                let response = widgets::selectable_row(ui, text, is_selected, is_row_focused);
                if response.clicked() {
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

pub fn action_to_message(action: CommitHistoryAction) -> Option<AppMessage> {
    match action {
        CommitHistoryAction::None => None,
        CommitHistoryAction::LoadHistory => Some(AppMessage::LoadCommitHistoryRequested),
        CommitHistoryAction::SelectCommit(hash) => Some(AppMessage::CommitSelected(hash)),
        CommitHistoryAction::DeselectCommit => Some(AppMessage::CommitDeselected),
    }
}
