//! Application state types.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crabontree_git::{Commit, FileDiff, StatusSummary, WorkingDirFile, WorkingDirStatus, DiffHunk};

/// Layout configuration for 3-pane mode.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub pane_widths: [f32; 3],
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            pane_widths: [0.25, 0.35, 0.40], // Commit History, Changed Files, Diff Viewer
        }
    }
}

/// Main application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_repo: Option<RepoState>,
    pub loading: bool,
    pub error: Option<String>,
    pub config: super::AppConfig,
    pub staging_progress: Option<StagingProgress>,
    pub layout_config: LayoutConfig,
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
}

/// File tree node.
#[derive(Debug, Clone)]
pub enum FileTreeNode {
    Directory {
        path: PathBuf,
        name: String,
        children: Vec<FileTreeNode>,
        is_expanded: bool,
        has_changes: bool,
    },
    File {
        path: PathBuf,
        name: String,
        status: Option<WorkingDirStatus>,
        size: u64,
    },
}

/// File tree state (Pane 2).
#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub root: FileTreeNode,
    pub expanded_paths: HashSet<PathBuf>,
    pub selected_path: Option<PathBuf>,
}

/// Changed files state (Pane 3).
#[derive(Debug, Clone)]
pub struct ChangedFilesState {
    pub staged: Vec<WorkingDirFile>,
    pub unstaged: Vec<WorkingDirFile>,
    pub untracked: Vec<WorkingDirFile>,
    pub conflicted: Vec<WorkingDirFile>,
    pub selected_file: Option<PathBuf>,
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
    // New 4-pane state
    pub branch_tree: Option<BranchTreeState>,
    pub file_tree: Option<FileTreeState>,
    pub changed_files: Option<ChangedFilesState>,
    pub file_view: FileViewState,
}
