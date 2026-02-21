use super::*;
use tracing::instrument;

impl GitRepository {
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
            let path = entry.path().ok_or_else(|| {
                GitError::OperationFailed("Invalid UTF-8 in file path".to_string())
            })?;
            let status = entry.status();

            let is_staged = status.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED
                    | git2::Status::INDEX_TYPECHANGE,
            );

            let file_status = if status.contains(git2::Status::CONFLICTED) {
                Some(WorkingDirStatus::Conflicted)
            } else if status.contains(git2::Status::WT_NEW) {
                Some(WorkingDirStatus::Untracked)
            } else if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::INDEX_MODIFIED)
            {
                Some(WorkingDirStatus::Modified)
            } else if status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::INDEX_DELETED)
            {
                Some(WorkingDirStatus::Deleted)
            } else if status.contains(git2::Status::WT_RENAMED)
                || status.contains(git2::Status::INDEX_RENAMED)
            {
                Some(WorkingDirStatus::Renamed)
            } else if status.contains(git2::Status::WT_TYPECHANGE)
                || status.contains(git2::Status::INDEX_TYPECHANGE)
            {
                Some(WorkingDirStatus::TypeChanged)
            } else if status.contains(git2::Status::INDEX_NEW) {
                Some(WorkingDirStatus::Modified)
            } else {
                None
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
        let obj = self.git2_repo.revparse_single("HEAD")?;
        self.git2_repo.reset_default(Some(&obj), [path])?;

        tracing::debug!("Unstaged file: {}", path.display());
        Ok(())
    }

    /// Stage all changes in the working directory.
    /// Optimized for large numbers of files by using chunked operations.
    #[instrument(skip(self))]
    pub fn stage_all(&self) -> Result<(), GitError> {
        tracing::debug!("Starting stage_all operation");

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
        self.stage_files_batch(&unstaged_paths)?;

        tracing::debug!("Staged all changes successfully");
        Ok(())
    }

    /// Stage multiple files in optimized batches.
    #[instrument(skip(self, paths))]
    pub fn stage_files_batch(&self, paths: &[std::path::PathBuf]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }

        tracing::debug!("Staging {} files in batch", paths.len());
        let mut index = self.git2_repo.index()?;
        const CHUNK_SIZE: usize = 500;

        for (chunk_idx, chunk) in paths.chunks(CHUNK_SIZE).enumerate() {
            tracing::debug!("Processing chunk {} ({} files)", chunk_idx + 1, chunk.len());
            for path in chunk {
                index.add_path(path)?;
            }
            index.write()?;
            tracing::debug!(
                "Completed chunk {} of {}",
                chunk_idx + 1,
                paths.len().div_ceil(CHUNK_SIZE)
            );
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
        const CHUNK_SIZE: usize = 500;

        for (chunk_idx, chunk) in paths.chunks(CHUNK_SIZE).enumerate() {
            tracing::debug!("Processing chunk {} ({} files)", chunk_idx + 1, chunk.len());

            let obj = self.git2_repo.revparse_single("HEAD")?;
            self.git2_repo.reset_default(Some(&obj), chunk)?;

            tracing::debug!(
                "Completed chunk {} of {}",
                chunk_idx + 1,
                paths.len().div_ceil(CHUNK_SIZE)
            );
        }

        tracing::debug!("Unstaged {} files successfully", paths.len());
        Ok(())
    }

    /// Unstage all changes (reset index to match HEAD).
    #[instrument(skip(self))]
    pub fn unstage_all(&self) -> Result<(), GitError> {
        tracing::debug!("Starting unstage_all operation");

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
        self.unstage_files_batch(&staged_paths)?;

        tracing::debug!("Unstaged all changes successfully");
        Ok(())
    }
}
