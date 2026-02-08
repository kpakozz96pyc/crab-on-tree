//! Application messages for state updates.

use std::path::PathBuf;
use crabontree_git::{Commit, FileDiff, StatusSummary, WorkingDirFile};

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

    /// User requested to load working directory status.
    LoadWorkingDirStatusRequested,

    /// Working directory status was loaded.
    WorkingDirStatusLoaded(Vec<WorkingDirFile>),

    /// User requested to stage a file.
    StageFileRequested(PathBuf),

    /// User requested to unstage a file.
    UnstageFileRequested(PathBuf),

    /// User requested to stage all changes.
    StageAllRequested,

    /// User requested to unstage all changes.
    UnstageAllRequested,

    /// Staging operation completed successfully.
    StagingCompleted,

    /// Staging progress update.
    StagingProgress {
        current: usize,
        total: usize,
        operation: String,
    },

    /// User updated the commit message.
    CommitMessageUpdated(String),

    /// User requested to create a commit.
    CreateCommitRequested,

    /// Commit was created successfully.
    CommitCreated {
        hash: String,
        message: String,
    },

    /// Author identity was loaded.
    AuthorIdentityLoaded {
        name: String,
        email: String,
    },
}
