//! Application state types.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crabontree_git::{Commit, DiffHunk, FileDiff, StatusSummary, WorkingDirFile};

/// Dialog for handling uncommitted changes before checkout.
#[derive(Debug, Clone)]
pub struct CheckoutChangesDialog {
    pub branch_name: String,
    pub is_remote: bool,
}

/// Dialog for handling remote branch name conflicts.
#[derive(Debug, Clone)]
pub struct BranchConflictDialog {
    pub remote_branch: String,
    pub local_name: String,
    pub new_name_input: String,
}

/// Main application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_repo: Option<RepoState>,
    pub loading: bool,
    pub error: Option<String>,
    pub config: super::AppConfig,
    pub staging_progress: Option<StagingProgress>,
    pub checkout_changes_dialog: Option<CheckoutChangesDialog>,
    pub branch_conflict_dialog: Option<BranchConflictDialog>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_repo: None,
            loading: false,
            error: None,
            config: super::AppConfig::default(),
            staging_progress: None,
            checkout_changes_dialog: None,
            branch_conflict_dialog: None,
        }
    }
}

/// Progress information for staging operations.
#[derive(Debug, Clone)]
pub struct StagingProgress {
    pub current: usize,
    pub total: usize,
    pub operation: String,
}

/// Branch information.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub commit_hash: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Tag information.
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>,
}

/// Branch and tag tree state (Pane 1).
#[derive(Debug, Clone)]
pub struct BranchTreeState {
    pub local_branches: Vec<BranchInfo>,
    pub remote_branches: HashMap<String, Vec<BranchInfo>>,
    pub tags: Vec<TagInfo>,
    pub current_branch: String,
    pub expanded_sections: HashSet<String>,
    pub selected_branch: Option<String>, // Selected branch (for keyboard navigation)
}

/// Changed files state (Pane 3).
#[derive(Debug, Clone)]
pub struct ChangedFilesState {
    pub staged: Vec<WorkingDirFile>,
    pub unstaged: Vec<WorkingDirFile>,
    pub untracked: Vec<WorkingDirFile>,
    pub conflicted: Vec<WorkingDirFile>,
    pub selected_file: Option<PathBuf>,
    pub selected_files: HashSet<PathBuf>,
    pub last_clicked_file: Option<PathBuf>, // For range selection
    pub commit_message: String,
    pub is_commit_view: bool, // true if viewing a commit, false if viewing working directory
    // Commit panel fields
    pub commit_summary: String,
    pub commit_description: String,
    pub amend_last_commit: bool,
    pub push_after_commit: bool,
}

/// Diff view mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
    ContentOnly,
}

/// File viewer state (Pane 4).
#[derive(Debug, Clone)]
pub enum FileViewState {
    None,
    Content {
        path: PathBuf,
        content: String,
        language: Option<String>,
    },
    Diff {
        path: PathBuf,
        hunks: Vec<DiffHunk>,
        view_mode: DiffViewMode,
    },
    MultipleDiffs {
        files: Vec<(PathBuf, Vec<DiffHunk>)>,
        view_mode: DiffViewMode,
    },
    Binary {
        path: PathBuf,
        size: u64,
    },
}

impl Default for FileViewState {
    fn default() -> Self {
        Self::None
    }
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
    // Docking-pane state
    pub branch_tree: Option<BranchTreeState>,
    pub changed_files: Option<ChangedFilesState>,
    pub file_view: FileViewState,
}

impl RepoState {
    pub fn new(path: PathBuf, head: String, branches: Vec<String>, status_summary: StatusSummary) -> Self {
        Self {
            path,
            head,
            branches,
            status_summary,
            commits: Vec::new(),
            selected_commit: None,
            commit_diff: None,
            working_dir_files: Vec::new(),
            commit_message: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            branch_tree: None,
            changed_files: None,
            file_view: FileViewState::default(),
        }
    }
}
