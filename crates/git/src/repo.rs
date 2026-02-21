//! Git repository operations.

mod branch;
mod commit;
mod file;
mod worktree;

use crate::GitError;
use std::path::{Path, PathBuf};
use tracing::instrument;

/// Summary of working directory status.
#[derive(Debug, Clone, Default)]
pub struct StatusSummary {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
}

/// Represents a Git commit with all its metadata.
#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,       // Full SHA-1 hash
    pub hash_short: String, // Short hash (7 chars)
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64, // Unix timestamp
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: i64,
    pub message: String,         // Full commit message
    pub message_summary: String, // First line only
    pub parent_hashes: Vec<String>,
}

/// Local branch metadata.
#[derive(Debug, Clone)]
pub struct LocalBranch {
    pub name: String,
    pub commit_hash: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Remote branch metadata.
#[derive(Debug, Clone)]
pub struct RemoteBranch {
    pub remote: String,
    pub name: String,
    pub commit_hash: String,
}

/// Tag metadata.
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>,
}

/// Represents a file change in a commit.
/// A line in a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

/// Type of a line in a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineType {
    /// Context line (unchanged).
    Context,
    /// Line was added.
    Addition,
    /// Line was deleted.
    Deletion,
}

/// A hunk in a diff (a section of changes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

/// A file diff with hunks showing line-by-line changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

/// Status of a file in a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// Status of a file in the working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkingDirStatus {
    /// File is modified but not staged.
    Modified,
    /// File is new and not tracked.
    Untracked,
    /// File is deleted in working directory.
    Deleted,
    /// File is renamed.
    Renamed,
    /// File has merge conflicts.
    Conflicted,
    /// File type changed (e.g., file -> symlink).
    TypeChanged,
}

/// Represents a file in the working directory.
#[derive(Debug, Clone)]
pub struct WorkingDirFile {
    /// Path to the file relative to repository root.
    pub path: PathBuf,
    /// Status of the file.
    pub status: WorkingDirStatus,
    /// Whether the file is staged (in index).
    pub is_staged: bool,
}

/// High-level Git repository wrapper.
///
/// Uses a hybrid approach:
/// - gix for read operations (fast, safe, pure Rust)
/// - git2 for write operations (mature, proven, well-documented)
pub struct GitRepository {
    path: PathBuf,
    gix_repo: gix::Repository,   // For reading
    git2_repo: git2::Repository, // For writing
}

impl GitRepository {
    /// Opens a Git repository at the specified path.
    ///
    /// Opens both gix (for reading) and git2 (for writing) repositories.
    #[instrument(skip(path))]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let path = path.as_ref();

        // Open with gix (for reading)
        let gix_repo = gix::discover(path).map_err(|e| {
            if !path.exists() {
                GitError::RepoNotFound(path.display().to_string())
            } else {
                GitError::InvalidRepo(format!("Failed to open repository with gix: {}", e))
            }
        })?;

        // Open with git2 (for writing)
        let git2_repo = git2::Repository::open(path)?;

        tracing::debug!("Opened repository (gix + git2) at {}", path.display());

        Ok(Self {
            path: path.to_path_buf(),
            gix_repo,
            git2_repo,
        })
    }

    /// Get the current HEAD reference.
    #[instrument(skip(self))]
    pub fn get_head(&self) -> Result<String, GitError> {
        let head = self
            .gix_repo
            .head()
            .map_err(|e| GitError::RefNotFound(format!("Failed to get HEAD: {}", e)))?;

        let head_name = if head.is_detached() {
            if let Some(id) = head.id() {
                format!("detached at {}", &id.to_string()[..7])
            } else {
                "detached HEAD".to_string()
            }
        } else {
            head.referent_name()
                .and_then(|name| name.shorten().to_string().into())
                .unwrap_or_else(|| "HEAD".to_string())
        };

        tracing::debug!("HEAD: {}", head_name);
        Ok(head_name)
    }

    /// Get all branches in the repository (both local and remote).
    #[instrument(skip(self))]
    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let mut branches = Vec::new();

        let references = self
            .gix_repo
            .references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references
            .local_branches()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate branches: {}", e)))?)
        .flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                branches.push(name);
            }
        }

        let references = self
            .gix_repo
            .references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.remote_branches().map_err(|e| {
            GitError::OperationFailed(format!("Failed to iterate remote branches: {}", e))
        })?)
        .flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                branches.push(format!("remotes/{}", name));
            }
        }

        branches.sort();
        tracing::debug!("Found {} branches (local + remote)", branches.len());
        Ok(branches)
    }

    /// Get a summary of the working directory status.
    #[instrument(skip(self))]
    pub fn get_status(&self) -> Result<StatusSummary, GitError> {
        let mut summary = StatusSummary::default();
        let files = self.get_working_dir_status()?;

        for file in files {
            match file.status {
                WorkingDirStatus::Modified => summary.modified += 1,
                WorkingDirStatus::Untracked => summary.untracked += 1,
                WorkingDirStatus::Deleted => summary.deleted += 1,
                WorkingDirStatus::Renamed | WorkingDirStatus::TypeChanged => summary.modified += 1,
                WorkingDirStatus::Conflicted => summary.modified += 1,
            }

            if file.is_staged {
                summary.added += 1;
            }
        }

        tracing::debug!(
            "Status summary: modified={}, added={}, deleted={}, untracked={}",
            summary.modified,
            summary.added,
            summary.deleted,
            summary.untracked
        );
        Ok(summary)
    }

    /// Get the repository path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
