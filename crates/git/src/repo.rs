//! Git repository operations.

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
    pub hash: String,           // Full SHA-1 hash
    pub hash_short: String,     // Short hash (7 chars)
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64,       // Unix timestamp
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: i64,
    pub message: String,        // Full commit message
    pub message_summary: String, // First line only
    pub parent_hashes: Vec<String>,
}

/// Represents a file change in a commit.
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
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

/// High-level Git repository wrapper.
///
/// Uses a hybrid approach:
/// - gix for read operations (fast, safe, pure Rust)
/// - git2 for write operations (mature, proven, well-documented)
pub struct GitRepository {
    path: PathBuf,
    gix_repo: gix::Repository,    // For reading
    git2_repo: git2::Repository,  // For writing
}

impl GitRepository {
    /// Opens a Git repository at the specified path.
    ///
    /// Opens both gix (for reading) and git2 (for writing) repositories.
    #[instrument(skip(path))]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let path = path.as_ref();

        // Open with gix (for reading)
        let gix_repo = gix::discover(path)
            .map_err(|e| {
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
        let head = self.gix_repo.head()
            .map_err(|e| GitError::RefNotFound(format!("Failed to get HEAD: {}", e)))?;

        let head_name = if head.is_detached() {
            // Detached HEAD - show short commit hash
            if let Some(id) = head.id() {
                format!("detached at {}", &id.to_string()[..7])
            } else {
                "detached HEAD".to_string()
            }
        } else {
            // Normal branch
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

        let references = self.gix_repo.references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        // Get local branches
        for r in (references.local_branches()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate branches: {}", e)))?).flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                branches.push(name);
            }
        }

        // Get remote branches
        let references = self.gix_repo.references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.remote_branches()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate remote branches: {}", e)))?).flatten()
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
        // For Phase 0, we'll use a simplified status implementation
        // Full status tracking will be implemented in later phases

        let summary = StatusSummary::default();

        // Try to get basic status using gix-status
        match self.gix_repo.index() {
            Ok(_index) => {
                // For now, we'll just return zeros
                // Full implementation requires walking the working tree
                tracing::debug!("Status check completed");
            }
            Err(e) => {
                tracing::warn!("Failed to get index: {}", e);
            }
        }

        Ok(summary)
    }

    /// Get the repository path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get commit history starting from HEAD.
    #[instrument(skip(self))]
    pub fn get_commit_history(&self, limit: Option<usize>) -> Result<Vec<Commit>, GitError> {
        let head = self.gix_repo.head()
            .map_err(|e| GitError::RefNotFound(format!("Cannot get HEAD: {}", e)))?;

        let head_id = head.id()
            .ok_or_else(|| GitError::OperationFailed("Cannot resolve HEAD".to_string()))?;

        let mut commits = Vec::new();
        let platform = self.gix_repo.rev_walk([head_id]);

        let commit_iter = platform.all()
            .map_err(|e| GitError::OperationFailed(format!("Failed to walk commits: {}", e)))?;

        for commit_result in commit_iter.take(limit.unwrap_or(100)) {
            let oid = commit_result
                .map_err(|e| GitError::OperationFailed(format!("Invalid commit: {}", e)))?;

            let commit_obj = self.gix_repo.find_commit(oid.id)
                .map_err(|e| GitError::OperationFailed(format!("Cannot find commit: {}", e)))?;

            commits.push(self.parse_commit(&commit_obj)?);
        }

        tracing::debug!("Retrieved {} commits", commits.len());
        Ok(commits)
    }

    /// Get details of a specific commit by hash.
    #[instrument(skip(self))]
    pub fn get_commit_details(&self, hash: &str) -> Result<Commit, GitError> {
        let oid = gix::ObjectId::from_hex(hash.as_bytes())
            .map_err(|e| GitError::OperationFailed(format!("Invalid hash: {}", e)))?;

        let commit_obj = self.gix_repo.find_commit(oid)
            .map_err(|e| GitError::OperationFailed(format!("Cannot find commit: {}", e)))?;

        self.parse_commit(&commit_obj)
    }

    /// Get the diff for a specific commit compared to its first parent.
    #[instrument(skip(self))]
    pub fn get_commit_diff(&self, hash: &str) -> Result<Vec<FileDiff>, GitError> {
        use gix::object::tree::diff::Action;

        let oid = gix::ObjectId::from_hex(hash.as_bytes())
            .map_err(|e| GitError::OperationFailed(format!("Invalid hash: {}", e)))?;

        let commit = self.gix_repo.find_commit(oid)
            .map_err(|e| GitError::OperationFailed(format!("Cannot find commit: {}", e)))?;

        let commit_tree = commit.tree()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get commit tree: {}", e)))?;

        // Get parent tree (or empty tree if this is the root commit)
        let parent_tree = if let Some(parent_id) = commit.parent_ids().next() {
            let parent = self.gix_repo.find_commit(parent_id)
                .map_err(|e| GitError::OperationFailed(format!("Cannot find parent: {}", e)))?;
            Some(parent.tree()
                .map_err(|e| GitError::OperationFailed(format!("Cannot get parent tree: {}", e)))?)
        } else {
            None
        };

        let mut diffs = Vec::new();

        // For now, we'll provide a simplified diff that shows changed files
        // Full line-by-line diff will be added in future phases

        // Use gix to get the tree differences
        if let Some(parent_tree) = parent_tree {
            parent_tree.changes()
                .map_err(|e| GitError::OperationFailed(format!("Cannot compute changes: {}", e)))?
                .for_each_to_obtain_tree(&commit_tree, |change| {
                    let path = String::from_utf8_lossy(change.location).to_string();

                    let status = match change.event {
                        gix::object::tree::diff::change::Event::Addition { .. } => FileStatus::Added,
                        gix::object::tree::diff::change::Event::Deletion { .. } => FileStatus::Deleted,
                        gix::object::tree::diff::change::Event::Modification { .. } => FileStatus::Modified,
                        gix::object::tree::diff::change::Event::Rewrite { .. } => FileStatus::Modified,
                    };

                    diffs.push(FileDiff {
                        path,
                        status,
                        additions: 0,  // Will be implemented with full diff
                        deletions: 0,  // Will be implemented with full diff
                    });

                    Ok::<_, std::convert::Infallible>(Action::Continue)
                })
                .map_err(|e| GitError::OperationFailed(format!("Cannot compute diff: {}", e)))?;
        } else {
            // Root commit - all files are additions
            // For now, we'll skip enumerating all files in root commit
            tracing::debug!("Root commit - skipping file enumeration");
        }

        tracing::debug!("Found {} changed files", diffs.len());
        Ok(diffs)
    }

    /// Parse a gix commit object into our Commit struct.
    fn parse_commit(&self, commit: &gix::Commit) -> Result<Commit, GitError> {
        let hash = commit.id.to_string();
        let hash_short = hash.chars().take(7).collect();

        let message = commit.message_raw()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get commit message: {}", e)))?;
        let message_str = String::from_utf8_lossy(message).to_string();
        let message_summary = message_str.lines().next().unwrap_or("").to_string();

        let author = commit.author()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get author: {}", e)))?;
        let committer = commit.committer()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get committer: {}", e)))?;

        Ok(Commit {
            hash,
            hash_short,
            author_name: String::from_utf8_lossy(author.name).to_string(),
            author_email: String::from_utf8_lossy(author.email).to_string(),
            author_date: author.time.seconds,
            committer_name: String::from_utf8_lossy(committer.name).to_string(),
            committer_email: String::from_utf8_lossy(committer.email).to_string(),
            committer_date: committer.time.seconds,
            message: message_str,
            message_summary,
            parent_hashes: commit.parent_ids().map(|id| id.to_string()).collect(),
        })
    }
}
