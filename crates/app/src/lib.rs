//! Application layer for CrabOnTree.
//!
//! This crate provides the core application logic, state management, and async job system.
//! It sits between the Git layer and UI layer, handling business logic and orchestration.

pub mod config;
pub mod effect;
pub mod executor;
pub mod job;
pub mod message;
pub mod reducer;
pub mod state;

/// Special constant to represent the working directory as a virtual commit.
pub const WORKING_DIR_HASH: &str = "WORKING_DIRECTORY";

pub use config::{load_config, save_config, themes_dir, AppConfig, CommitDraft};
pub use crabontree_git::{
    Commit, DiffHunk, DiffLine, DiffLineType, FileDiff, FileStatus, WorkingDirFile,
    WorkingDirStatus,
};
pub use effect::Effect;
pub use executor::JobExecutor;
pub use job::{Job, JobId};
pub use message::AppMessage;
pub use reducer::reduce;
pub use state::{
    AppState, BranchConflictDialog, BranchInfo, BranchTreeState, ChangedFilesState,
    CheckoutChangesDialog, CommitInfo, DiffViewMode, FileViewState, RepoState, StagingProgress,
    TagInfo,
};
