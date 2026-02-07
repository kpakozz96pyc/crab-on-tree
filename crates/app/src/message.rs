//! Application messages for state updates.

use std::path::PathBuf;
use crabontree_git::{Commit, FileDiff, StatusSummary};

/// Messages that drive application state changes.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// User requested to open a repository.
    OpenRepoRequested(PathBuf),

    /// Repository was successfully opened.
    RepoOpened {
        path: PathBuf,
        head: String,
        branches: Vec<String>,
        status: StatusSummary,
    },

    /// User requested to close the current repository.
    CloseRepo,

    /// User requested to refresh repository data.
    RefreshRepo,

    /// Repository data was refreshed.
    RepoRefreshed {
        head: String,
        branches: Vec<String>,
        status: StatusSummary,
    },

    /// An error occurred.
    Error(String),

    /// User dismissed the error.
    ClearError,

    /// User requested to load commit history.
    LoadCommitHistoryRequested,

    /// Commit history was loaded.
    CommitHistoryLoaded(Vec<Commit>),

    /// User selected a commit.
    CommitSelected(String),

    /// User deselected the current commit.
    CommitDeselected,

    /// Commit diff was loaded.
    CommitDiffLoaded {
        commit_hash: String,
        diff: Vec<FileDiff>,
    },
}
