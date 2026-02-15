//! Job types for async operations.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

impl JobId {
    /// Generate a new unique job ID.
    pub fn new() -> Self {
        Self(NEXT_JOB_ID.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

/// Jobs that can be executed asynchronously.
#[derive(Debug, Clone)]
pub enum Job {
    /// Open a repository at the given path.
    OpenRepo(PathBuf),

    /// Refresh data for the repository at the given path.
    RefreshRepo(PathBuf),

    /// Load commit history for the repository at the given path.
    LoadCommitHistory(PathBuf),

    /// Load diff for a specific commit.
    LoadCommitDiff {
        repo_path: PathBuf,
        commit_hash: String,
    },

    /// Load working directory status.
    LoadWorkingDirStatus(PathBuf),

    /// Stage a file.
    StageFile {
        repo_path: PathBuf,
        file_path: PathBuf,
    },

    /// Unstage a file.
    UnstageFile {
        repo_path: PathBuf,
        file_path: PathBuf,
    },

    /// Stage all changes.
    StageAll(PathBuf),

    /// Unstage all changes.
    UnstageAll(PathBuf),

    /// Create a commit with the given message.
    CreateCommit {
        repo_path: PathBuf,
        message: String,
        amend: bool,
        push: bool,
    },

    /// Load author identity from git config.
    LoadAuthorIdentity(PathBuf),

    // ===== 4-Pane Layout Jobs =====

    /// Load branch tree.
    LoadBranchTree(PathBuf),

    /// Checkout a branch.
    CheckoutBranch {
        repo_path: PathBuf,
        branch_name: String,
    },

    /// Check if repository has uncommitted changes.
    CheckUncommittedChanges {
        repo_path: PathBuf,
        branch_name: String,
        is_remote: bool,
    },

    /// Stash changes and checkout branch.
    StashAndCheckout {
        repo_path: PathBuf,
        branch_name: String,
        is_remote: bool,
        from_branch: String,
    },

    /// Discard changes and checkout branch.
    DiscardAndCheckout {
        repo_path: PathBuf,
        branch_name: String,
        is_remote: bool,
    },

    /// Check if local branch name exists (for remote branch checkout).
    CheckLocalBranchExists {
        repo_path: PathBuf,
        remote_branch: String,
        local_name: String,
    },

    /// Checkout remote branch (create local tracking branch).
    CheckoutRemoteBranch {
        repo_path: PathBuf,
        remote_branch: String,
        local_name: String,
        override_existing: bool,
    },

    /// Load changed files.
    LoadChangedFiles(PathBuf),

    /// Load file content.
    LoadFileContent {
        repo_path: PathBuf,
        file_path: PathBuf,
    },

    /// Load file diff.
    LoadFileDiff {
        repo_path: PathBuf,
        file_path: PathBuf,
    },

    /// Load multiple file diffs.
    LoadMultipleFileDiffs {
        repo_path: PathBuf,
        file_paths: Vec<PathBuf>,
    },
}
