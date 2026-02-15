//! Application messages for state updates.

use std::path::PathBuf;
use crabontree_git::{Commit, DiffHunk, FileDiff, StatusSummary, WorkingDirFile};
use crate::state::{BranchTreeState, ChangedFilesState, DiffViewMode};

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

    // ===== 4-Pane Layout Messages =====

    /// User requested to load branch tree.
    LoadBranchTreeRequested,

    /// Branch tree was loaded.
    BranchTreeLoaded(BranchTreeState),

    /// User toggled a branch tree section (expand/collapse).
    BranchSectionToggled(String),

    /// User selected a branch (single-click or keyboard navigation).
    BranchSelected { name: String, is_remote: bool },

    /// User requested to checkout a branch (double-click or Enter).
    BranchCheckoutRequested { name: String, is_remote: bool },

    /// Show dialog for handling uncommitted changes before checkout.
    ShowCheckoutWithChangesDialog {
        branch_name: String,
        is_remote: bool,
    },

    /// User chose to stash changes and checkout.
    CheckoutWithStash { branch_name: String, is_remote: bool },

    /// User chose to discard changes and checkout.
    CheckoutWithDiscard { branch_name: String, is_remote: bool },

    /// Show dialog for remote branch name conflict.
    ShowRemoteBranchConflictDialog {
        remote_branch: String,
        local_name: String,
    },

    /// User chose to override existing local branch.
    CheckoutRemoteOverride {
        remote_branch: String,
        local_name: String,
    },

    /// User chose to rename local branch for remote checkout.
    CheckoutRemoteRename {
        remote_branch: String,
        new_local_name: String,
    },

    /// Branch was checked out successfully.
    BranchCheckedOut(String),

    /// Changes were stashed successfully.
    ChangesStashed { stash_name: String },

    /// Changes were discarded successfully.
    ChangesDiscarded,

    /// User requested to load changed files.
    LoadChangedFilesRequested,

    /// Changed files were loaded.
    ChangedFilesLoaded(ChangedFilesState),

    /// User selected a changed file.
    ChangedFileSelected(PathBuf),

    /// User selected a file with modifiers (Ctrl/Shift).
    SelectFileWithModifiers {
        path: PathBuf,
        ctrl: bool,
        shift: bool,
    },

    /// User requested to stage all selected files.
    StageSelectedFilesRequested,

    /// User requested to unstage all selected files.
    UnstageSelectedFilesRequested,

    /// User requested to view file content.
    FileContentRequested(PathBuf),

    /// File content was loaded.
    FileContentLoaded {
        path: PathBuf,
        content: String,
        language: Option<String>,
    },

    /// User requested to view file diff.
    FileDiffRequested(PathBuf),

    /// File diff was loaded.
    FileDiffLoaded {
        path: PathBuf,
        hunks: Vec<DiffHunk>,
    },

    /// Multiple file diffs were loaded.
    MultipleFileDiffsLoaded {
        files: Vec<(PathBuf, Vec<DiffHunk>)>,
    },

    /// Binary file was detected.
    BinaryFileDetected {
        path: PathBuf,
        size: u64,
    },

    /// User changed diff view mode.
    DiffViewModeChanged(DiffViewMode),

    /// User updated the commit summary.
    CommitSummaryUpdated(String),

    /// User updated the commit description.
    CommitDescriptionUpdated(String),

    /// User toggled amend last commit option.
    AmendLastCommitToggled(bool),

    /// User toggled push after commit option.
    PushAfterCommitToggled(bool),

    /// User requested to commit changes.
    CommitChangesRequested {
        summary: String,
        description: String,
        amend: bool,
        push: bool,
    },

}
