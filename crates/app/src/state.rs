//! Application state types.

use std::path::PathBuf;
use crabontree_git::{Commit, FileDiff, StatusSummary, WorkingDirFile};

/// Main application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_repo: Option<RepoState>,
    pub loading: bool,
    pub error: Option<String>,
    pub config: super::AppConfig,
    pub staging_progress: Option<StagingProgress>,
}

/// Progress information for staging operations.
#[derive(Debug, Clone)]
pub struct StagingProgress {
    pub current: usize,
    pub total: usize,
    pub operation: String,
}

/// State of an open repository.
#[derive(Debug, Clone)]
pub struct RepoState {
    pub path: PathBuf,
    pub head: String,
    pub branches: Vec<String>,
    pub status_summary: StatusSummary,
    pub commits: Vec<Commit>,
    pub selected_commit: Option<String>,
    pub commit_diff: Option<Vec<FileDiff>>,
    pub working_dir_files: Vec<WorkingDirFile>,
    pub commit_message: String,
    pub author_name: String,
    pub author_email: String,
}
