use super::*;
use tracing::instrument;

impl GitRepository {
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
            let read_dir = fs::read_dir(path).map_err(|e| {
                GitError::OperationFailed(format!("Failed to read directory: {}", e))
            })?;

            for entry in read_dir {
                let entry = entry.map_err(|e| {
                    GitError::OperationFailed(format!("Failed to read entry: {}", e))
                })?;
                let path = entry.path();

                if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                    continue;
                }

                let metadata = entry.metadata().map_err(|e| {
                    GitError::OperationFailed(format!("Failed to get metadata: {}", e))
                })?;

                let relative = path
                    .strip_prefix(repo_root)
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
        let head = self.git2_repo.head()?;
        let head_tree = head.peel_to_tree()?;

        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.pathspec(file_path);

        let diff = self
            .git2_repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))?;

        let mut hunks = Vec::new();

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

        let mut buffer = vec![0u8; 8192];
        let bytes_read = std::fs::File::open(&full_path)
            .and_then(|mut f| std::io::Read::read(&mut f, &mut buffer))
            .map_err(|e| GitError::OperationFailed(format!("Failed to read file: {}", e)))?;

        let is_binary = buffer[..bytes_read].contains(&0);

        tracing::debug!("File is binary: {}", is_binary);
        Ok(is_binary)
    }
}
