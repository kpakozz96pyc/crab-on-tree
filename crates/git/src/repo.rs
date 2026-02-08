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

    /// Get working directory status (changed files).
    ///
    /// Returns a list of all files that differ from HEAD, including:
    /// - Modified files (staged or unstaged)
    /// - New files (tracked or untracked)
    /// - Deleted files
    /// - Renamed files
    /// - Conflicted files
    ///
    /// Uses git2 for this operation as it provides excellent status API.
    #[instrument(skip(self))]
    pub fn get_working_dir_status(&self) -> Result<Vec<WorkingDirFile>, GitError> {
        let start = std::time::Instant::now();

        let mut options = git2::StatusOptions::new();
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self.git2_repo.statuses(Some(&mut options))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path()
                .ok_or_else(|| GitError::OperationFailed("Invalid UTF-8 in file path".to_string()))?;
            let status = entry.status();

            // Check if file is staged (in index)
            let is_staged = status.intersects(
                git2::Status::INDEX_NEW |
                git2::Status::INDEX_MODIFIED |
                git2::Status::INDEX_DELETED |
                git2::Status::INDEX_RENAMED |
                git2::Status::INDEX_TYPECHANGE
            );

            // Determine the file status
            // Prioritize working tree changes over index changes for display
            let file_status = if status.contains(git2::Status::CONFLICTED) {
                Some(WorkingDirStatus::Conflicted)
            } else if status.contains(git2::Status::WT_NEW) {
                Some(WorkingDirStatus::Untracked)
            } else if status.contains(git2::Status::WT_MODIFIED) || status.contains(git2::Status::INDEX_MODIFIED) {
                Some(WorkingDirStatus::Modified)
            } else if status.contains(git2::Status::WT_DELETED) || status.contains(git2::Status::INDEX_DELETED) {
                Some(WorkingDirStatus::Deleted)
            } else if status.contains(git2::Status::WT_RENAMED) || status.contains(git2::Status::INDEX_RENAMED) {
                Some(WorkingDirStatus::Renamed)
            } else if status.contains(git2::Status::WT_TYPECHANGE) || status.contains(git2::Status::INDEX_TYPECHANGE) {
                Some(WorkingDirStatus::TypeChanged)
            } else if status.contains(git2::Status::INDEX_NEW) {
                // New file in index (staged)
                Some(WorkingDirStatus::Modified) // Treat as modified since it's new
            } else {
                None // Ignore other statuses (like ignored files)
            };

            if let Some(status) = file_status {
                files.push(WorkingDirFile {
                    path: PathBuf::from(path),
                    status,
                    is_staged,
                });
            }
        }

        let elapsed = start.elapsed();
        tracing::info!(
            "get_working_dir_status completed: {} files in {:?}",
            files.len(),
            elapsed
        );
        Ok(files)
    }

    /// Stage a file (add it to the index).
    #[instrument(skip(self))]
    pub fn stage_file(&self, path: &std::path::Path) -> Result<(), GitError> {
        let mut index = self.git2_repo.index()?;
        index.add_path(path)?;
        index.write()?;
        tracing::debug!("Staged file: {}", path.display());
        Ok(())
    }

    /// Unstage a file (remove it from the index, but keep working directory changes).
    #[instrument(skip(self))]
    pub fn unstage_file(&self, path: &std::path::Path) -> Result<(), GitError> {
        // Use git reset HEAD <path> to unstage
        let obj = self.git2_repo.revparse_single("HEAD")?;
        self.git2_repo.reset_default(Some(&obj), &[path])?;

        tracing::debug!("Unstaged file: {}", path.display());
        Ok(())
    }

    /// Stage all changes in the working directory.
    /// Optimized for large numbers of files by using chunked operations.
    #[instrument(skip(self))]
    pub fn stage_all(&self) -> Result<(), GitError> {
        tracing::debug!("Starting stage_all operation");

        // Get list of unstaged files
        let files = self.get_working_dir_status()?;
        let unstaged_paths: Vec<_> = files
            .iter()
            .filter(|f| !f.is_staged)
            .map(|f| f.path.clone())
            .collect();

        if unstaged_paths.is_empty() {
            tracing::debug!("No unstaged files to stage");
            return Ok(());
        }

        tracing::debug!("Found {} unstaged files", unstaged_paths.len());

        // Use batch staging for efficiency
        self.stage_files_batch(&unstaged_paths)?;

        tracing::debug!("Staged all changes successfully");
        Ok(())
    }

    /// Stage multiple files in optimized batches.
    /// This is more efficient than staging files one by one.
    #[instrument(skip(self, paths))]
    pub fn stage_files_batch(&self, paths: &[std::path::PathBuf]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }

        tracing::debug!("Staging {} files in batch", paths.len());

        let mut index = self.git2_repo.index()?;

        // Process files in chunks to avoid memory issues with very large batches
        const CHUNK_SIZE: usize = 500;

        for (chunk_idx, chunk) in paths.chunks(CHUNK_SIZE).enumerate() {
            tracing::debug!("Processing chunk {} ({} files)", chunk_idx + 1, chunk.len());

            for path in chunk {
                index.add_path(path)?;
            }

            // Write index after each chunk to commit progress
            index.write()?;

            tracing::debug!("Completed chunk {} of {}", chunk_idx + 1, (paths.len() + CHUNK_SIZE - 1) / CHUNK_SIZE);
        }

        tracing::debug!("Staged {} files successfully", paths.len());
        Ok(())
    }

    /// Unstage multiple files in optimized batches.
    #[instrument(skip(self, paths))]
    pub fn unstage_files_batch(&self, paths: &[std::path::PathBuf]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }

        tracing::debug!("Unstaging {} files in batch", paths.len());

        // Process in chunks
        const CHUNK_SIZE: usize = 500;

        for (chunk_idx, chunk) in paths.chunks(CHUNK_SIZE).enumerate() {
            tracing::debug!("Processing chunk {} ({} files)", chunk_idx + 1, chunk.len());

            let obj = self.git2_repo.revparse_single("HEAD")?;
            self.git2_repo.reset_default(Some(&obj), chunk)?;

            tracing::debug!("Completed chunk {} of {}", chunk_idx + 1, (paths.len() + CHUNK_SIZE - 1) / CHUNK_SIZE);
        }

        tracing::debug!("Unstaged {} files successfully", paths.len());
        Ok(())
    }

    /// Unstage all changes (reset index to match HEAD).
    /// Optimized for large numbers of files by using chunked operations.
    #[instrument(skip(self))]
    pub fn unstage_all(&self) -> Result<(), GitError> {
        tracing::debug!("Starting unstage_all operation");

        // Get list of staged files
        let files = self.get_working_dir_status()?;
        let staged_paths: Vec<_> = files
            .iter()
            .filter(|f| f.is_staged)
            .map(|f| f.path.clone())
            .collect();

        if staged_paths.is_empty() {
            tracing::debug!("No staged files to unstage");
            return Ok(());
        }

        tracing::debug!("Found {} staged files", staged_paths.len());

        // Use batch unstaging for efficiency
        self.unstage_files_batch(&staged_paths)?;

        tracing::debug!("Unstaged all changes successfully");
        Ok(())
    }

    /// Get commit history starting from HEAD.
    #[instrument(skip(self))]
    pub fn get_commit_history(&self, limit: Option<usize>) -> Result<Vec<Commit>, GitError> {
        let start = std::time::Instant::now();

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

        let elapsed = start.elapsed();
        tracing::info!(
            "get_commit_history completed: {} commits in {:?}",
            commits.len(),
            elapsed
        );
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
        let start = std::time::Instant::now();

        // Use git2 for diff generation (better patch support than gix)
        let oid = git2::Oid::from_str(hash)?;
        let commit = self.git2_repo.find_commit(oid)?;

        let commit_tree = commit.tree()?;

        // Get parent tree (or None for root commit)
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        // Create diff between parent and current commit
        let diff = self.git2_repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            None,
        )?;

        let mut file_diffs = Vec::new();

        // Process each delta (changed file)
        for (delta_idx, delta) in diff.deltas().enumerate() {
            let path = delta.new_file().path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());

            let status = match delta.status() {
                git2::Delta::Added => FileStatus::Added,
                git2::Delta::Deleted => FileStatus::Deleted,
                git2::Delta::Modified => FileStatus::Modified,
                git2::Delta::Renamed => FileStatus::Renamed,
                git2::Delta::Copied => FileStatus::Copied,
                _ => FileStatus::Modified,
            };

            let mut hunks = Vec::new();
            let mut additions = 0;
            let mut deletions = 0;

            // Generate patch for this file
            let patch = git2::Patch::from_diff(&diff, delta_idx)?;

            if let Some(patch) = patch {
                // Process each hunk in the patch
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk, hunk_lines_count) = patch.hunk(hunk_idx)?;

                    let mut hunk_lines = Vec::new();

                    // Process each line in the hunk
                    for line_idx in 0..hunk_lines_count {
                        let line = patch.line_in_hunk(hunk_idx, line_idx)?;

                        let (line_type, old_line_num, new_line_num) = match line.origin() {
                            '+' => {
                                additions += 1;
                                (DiffLineType::Addition, None, line.new_lineno())
                            },
                            '-' => {
                                deletions += 1;
                                (DiffLineType::Deletion, line.old_lineno(), None)
                            },
                            ' ' => {
                                (DiffLineType::Context, line.old_lineno(), line.new_lineno())
                            },
                            _ => continue, // Skip other line types (headers, etc.)
                        };

                        let content = String::from_utf8_lossy(line.content()).to_string();

                        hunk_lines.push(DiffLine {
                            line_type,
                            content,
                            old_line_number: old_line_num.map(|n| n as usize),
                            new_line_number: new_line_num.map(|n| n as usize),
                        });
                    }

                    hunks.push(DiffHunk {
                        old_start: hunk.old_start() as usize,
                        old_lines: hunk.old_lines() as usize,
                        new_start: hunk.new_start() as usize,
                        new_lines: hunk.new_lines() as usize,
                        lines: hunk_lines,
                    });
                }
            }

            file_diffs.push(FileDiff {
                path,
                status,
                additions,
                deletions,
                hunks,
            });
        }

        let elapsed = start.elapsed();
        tracing::info!(
            "get_commit_diff completed: {} files in {:?}",
            file_diffs.len(),
            elapsed
        );
        Ok(file_diffs)
    }

    /// Get author identity from git config.
    ///
    /// Returns (name, email) tuple from user.name and user.email config.
    /// Falls back to a default if config is missing.
    #[instrument(skip(self))]
    pub fn get_author_identity(&self) -> Result<(String, String), GitError> {
        let config = self.git2_repo.config()?;

        // Try to get user.name
        let name = match config.get_string("user.name") {
            Ok(name) => name,
            Err(_) => {
                tracing::warn!("user.name not set in git config, using fallback");
                whoami::username()
            }
        };

        // Try to get user.email
        let email = match config.get_string("user.email") {
            Ok(email) => email,
            Err(_) => {
                tracing::warn!("user.email not set in git config, using fallback");
                let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string());
                format!("{}@{}", whoami::username(), hostname)
            }
        };

        tracing::debug!("Author identity: {} <{}>", name, email);
        Ok((name, email))
    }

    /// Check if there are staged changes ready to commit.
    ///
    /// Returns true if the index differs from HEAD.
    #[instrument(skip(self))]
    pub fn has_staged_changes(&self) -> Result<bool, GitError> {
        // Get HEAD tree
        let head = self.git2_repo.head()?;
        let head_tree = head.peel_to_tree()?;

        // Get index
        let mut index = self.git2_repo.index()?;
        let index_tree_oid = index.write_tree()?;
        let index_tree = self.git2_repo.find_tree(index_tree_oid)?;

        // Compare trees
        let diff = self.git2_repo.diff_tree_to_tree(
            Some(&head_tree),
            Some(&index_tree),
            None,
        )?;

        let has_changes = diff.deltas().len() > 0;
        tracing::debug!("Has staged changes: {}", has_changes);
        Ok(has_changes)
    }

    /// Create a commit with the current staged changes.
    ///
    /// Returns the commit hash as a string.
    #[instrument(skip(self, message))]
    pub fn create_commit(&self, message: &str) -> Result<String, GitError> {
        // Validate message
        if message.trim().is_empty() {
            return Err(GitError::OperationFailed(
                "Commit message cannot be empty".to_string()
            ));
        }

        // Check for staged changes
        if !self.has_staged_changes()? {
            return Err(GitError::OperationFailed(
                "No staged changes to commit".to_string()
            ));
        }

        // Get author identity
        let (author_name, author_email) = self.get_author_identity()?;

        // Create signature for author and committer (same in this case)
        let signature = git2::Signature::now(&author_name, &author_email)?;

        // Get HEAD reference and commit
        let head = self.git2_repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        // Get tree from index
        let mut index = self.git2_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git2_repo.find_tree(tree_oid)?;

        // Create the commit
        let commit_oid = self.git2_repo.commit(
            Some("HEAD"),           // Update HEAD
            &signature,             // Author
            &signature,             // Committer
            message,                // Commit message
            &tree,                  // Tree
            &[&parent_commit],      // Parents
        )?;

        let commit_hash = commit_oid.to_string();
        tracing::info!("Created commit: {}", commit_hash);
        Ok(commit_hash)
    }

    /// List all local branches with metadata.
    #[instrument(skip(self))]
    pub fn list_local_branches(&self) -> Result<Vec<(String, String, bool, Option<String>)>, GitError> {
        let mut branches = Vec::new();

        let head = self.gix_repo.head()
            .map_err(|e| GitError::RefNotFound(format!("Failed to get HEAD: {}", e)))?;

        let current_branch = if !head.is_detached() {
            head.referent_name()
                .and_then(|name| name.shorten().to_string().into())
        } else {
            None
        };

        let references = self.gix_repo.references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.local_branches()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate branches: {}", e)))?).flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                let commit_hash = r.id().to_string();
                let is_current = current_branch.as_deref() == Some(&name);

                // Try to get upstream (tracking branch)
                let upstream = None; // TODO: Implement upstream tracking if needed

                branches.push((name, commit_hash, is_current, upstream));
            }
        }

        tracing::debug!("Found {} local branches", branches.len());
        Ok(branches)
    }

    /// List all remote branches grouped by remote name.
    #[instrument(skip(self))]
    pub fn list_remote_branches(&self) -> Result<std::collections::HashMap<String, Vec<(String, String)>>, GitError> {
        use std::collections::HashMap;

        let mut remotes: HashMap<String, Vec<(String, String)>> = HashMap::new();

        let references = self.gix_repo.references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.remote_branches()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate remote branches: {}", e)))?).flatten()
        {
            if let Some(full_name) = r.name().shorten().to_string().into() {
                // full_name is like "origin/main"
                let parts: Vec<&str> = full_name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let remote = parts[0].to_string();
                    let branch = parts[1].to_string();
                    let commit_hash = r.id().to_string();

                    remotes.entry(remote)
                        .or_insert_with(Vec::new)
                        .push((branch, commit_hash));
                }
            }
        }

        tracing::debug!("Found {} remotes with branches", remotes.len());
        Ok(remotes)
    }

    /// List all tags (annotated and lightweight).
    #[instrument(skip(self))]
    pub fn list_tags(&self) -> Result<Vec<(String, String, Option<String>)>, GitError> {
        let mut tags = Vec::new();

        let references = self.gix_repo.references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.tags()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate tags: {}", e)))?).flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                let commit_hash = r.id().to_string();

                // Try to get tag message (for annotated tags)
                let message = None; // TODO: Parse tag object for message if needed

                tags.push((name, commit_hash, message));
            }
        }

        tracing::debug!("Found {} tags", tags.len());
        Ok(tags)
    }

    /// Checkout a branch.
    #[instrument(skip(self))]
    pub fn checkout_branch(&self, branch_name: &str) -> Result<(), GitError> {
        // Use git2 for checkout (write operation)
        let obj = self.git2_repo.revparse_single(&format!("refs/heads/{}", branch_name))?;

        self.git2_repo.checkout_tree(&obj, None)?;
        self.git2_repo.set_head(&format!("refs/heads/{}", branch_name))?;

        tracing::info!("Checked out branch: {}", branch_name);
        Ok(())
    }

    /// Build repository tree from working directory.
    #[instrument(skip(self))]
    pub fn get_repository_tree(&self) -> Result<Vec<(PathBuf, bool, u64)>, GitError> {
        use std::fs;

        let mut entries = Vec::new();
        let repo_path = self.path();

        fn walk_dir(
            path: &Path,
            repo_root: &Path,
            entries: &mut Vec<(PathBuf, bool, u64)>,
        ) -> Result<(), GitError> {
            let read_dir = fs::read_dir(path)
                .map_err(|e| GitError::OperationFailed(format!("Failed to read directory: {}", e)))?;

            for entry in read_dir {
                let entry = entry
                    .map_err(|e| GitError::OperationFailed(format!("Failed to read entry: {}", e)))?;
                let path = entry.path();

                // Skip .git directory
                if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                    continue;
                }

                let metadata = entry.metadata()
                    .map_err(|e| GitError::OperationFailed(format!("Failed to get metadata: {}", e)))?;

                let relative = path.strip_prefix(repo_root)
                    .map_err(|e| GitError::OperationFailed(format!("Path error: {}", e)))?
                    .to_path_buf();

                let is_dir = metadata.is_dir();
                let size = if is_dir { 0 } else { metadata.len() };

                entries.push((relative, is_dir, size));

                if is_dir {
                    walk_dir(&path, repo_root, entries)?;
                }
            }

            Ok(())
        }

        walk_dir(repo_path, repo_path, &mut entries)?;

        tracing::debug!("Built repository tree with {} entries", entries.len());
        Ok(entries)
    }

    /// Get file content from filesystem.
    #[instrument(skip(self))]
    pub fn get_file_content(&self, file_path: &Path) -> Result<String, GitError> {
        let full_path = self.path().join(file_path);

        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| GitError::OperationFailed(format!("Failed to read file: {}", e)))?;

        tracing::debug!("Read file content: {} bytes", content.len());
        Ok(content)
    }

    /// Get diff hunks for a changed file.
    #[instrument(skip(self))]
    pub fn get_file_diff(&self, file_path: &Path) -> Result<Vec<DiffHunk>, GitError> {
        // Get HEAD tree
        let head = self.git2_repo.head()?;
        let head_tree = head.peel_to_tree()?;

        // Create diff options
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.pathspec(file_path);

        // Create diff between HEAD and working directory
        let diff = self.git2_repo.diff_tree_to_workdir_with_index(
            Some(&head_tree),
            Some(&mut diff_opts),
        )?;

        let mut hunks = Vec::new();

        // Process each delta
        for (delta_idx, _delta) in diff.deltas().enumerate() {
            let patch = git2::Patch::from_diff(&diff, delta_idx)?;

            if let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk, hunk_lines_count) = patch.hunk(hunk_idx)?;
                    let mut hunk_lines = Vec::new();

                    for line_idx in 0..hunk_lines_count {
                        let line = patch.line_in_hunk(hunk_idx, line_idx)?;

                        let (line_type, old_line_num, new_line_num) = match line.origin() {
                            '+' => (DiffLineType::Addition, None, line.new_lineno()),
                            '-' => (DiffLineType::Deletion, line.old_lineno(), None),
                            ' ' => (DiffLineType::Context, line.old_lineno(), line.new_lineno()),
                            _ => continue,
                        };

                        let content = String::from_utf8_lossy(line.content()).to_string();

                        hunk_lines.push(DiffLine {
                            line_type,
                            content,
                            old_line_number: old_line_num.map(|n| n as usize),
                            new_line_number: new_line_num.map(|n| n as usize),
                        });
                    }

                    hunks.push(DiffHunk {
                        old_start: hunk.old_start() as usize,
                        old_lines: hunk.old_lines() as usize,
                        new_start: hunk.new_start() as usize,
                        new_lines: hunk.new_lines() as usize,
                        lines: hunk_lines,
                    });
                }
            }
        }

        tracing::debug!("Generated {} diff hunks for file", hunks.len());
        Ok(hunks)
    }

    /// Check if a file is binary (contains null bytes).
    #[instrument(skip(self))]
    pub fn is_binary_file(&self, file_path: &Path) -> Result<bool, GitError> {
        let full_path = self.path().join(file_path);

        // Read first 8KB to check for null bytes
        let mut buffer = vec![0u8; 8192];
        let bytes_read = std::fs::File::open(&full_path)
            .and_then(|mut f| std::io::Read::read(&mut f, &mut buffer))
            .map_err(|e| GitError::OperationFailed(format!("Failed to read file: {}", e)))?;

        let is_binary = buffer[..bytes_read].contains(&0);

        tracing::debug!("File is binary: {}", is_binary);
        Ok(is_binary)
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
