pub mod branches;
pub mod changed_files;
pub mod commit_history;
pub mod diff_viewer;
pub mod scrollable_pane;

/// Represents the four main panes in the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Pane {
    CommitHistory,
    ChangedFiles,
    DiffViewer,
    Branches,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::CommitHistory => "Commit History",
            Pane::ChangedFiles => "Changed Files",
            Pane::DiffViewer => "Diff Viewer",
            Pane::Branches => "Branches",
        }
    }
}
