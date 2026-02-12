pub mod changed_files;
pub mod commit_history;
pub mod diff_viewer;
pub mod scrollable_pane;

use crabontree_app::{AppMessage, ChangedFilesState, FileViewState, RepoState};
use eframe::egui;

/// Represents the three main panes in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Pane {
    CommitHistory,
    ChangedFiles,
    DiffViewer,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::CommitHistory => "Commit History",
            Pane::ChangedFiles => "Changed Files",
            Pane::DiffViewer => "Diff Viewer",
        }
    }
}

/// Renders the commit history pane with its content.
/// Returns an optional message to be handled by the app.
pub fn render_commit_history_pane(ui: &mut egui::Ui, state: &RepoState) -> Option<AppMessage> {
    let (commits, selected_commit, has_working_dir_changes) = (
        state.commits.as_slice(),
        state.selected_commit.as_ref(),
        !state.working_dir_files.is_empty(),
    );

    let action = commit_history::render(ui, commits, selected_commit, has_working_dir_changes);
    commit_history::action_to_message(action)
}

/// Renders the changed files pane with its content.
/// Returns an optional message to be handled by the app.
pub fn render_changed_files_pane(
    ui: &mut egui::Ui,
    files: &Option<ChangedFilesState>,
) -> Option<AppMessage> {
    if let Some(files) = files {
        let action = changed_files::render(ui, files);
        changed_files::action_to_message(action)
    } else {
        ui.label("Loading changed files...");
        None
    }
}

/// Renders the diff viewer pane with its content.
/// This pane doesn't generate messages.
pub fn render_diff_viewer_pane(ui: &mut egui::Ui, file_view: &FileViewState) {
    diff_viewer::render(ui, file_view);
}
