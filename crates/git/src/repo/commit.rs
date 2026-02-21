use super::*;
use tracing::instrument;

impl GitRepository {
    /// Get commit history starting from HEAD.
    #[instrument(skip(self))]
    pub fn get_commit_history(&self, limit: Option<usize>) -> Result<Vec<Commit>, GitError> {
        let start = std::time::Instant::now();

        let head = self
            .gix_repo
            .head()
            .map_err(|e| GitError::RefNotFound(format!("Cannot get HEAD: {}", e)))?;

        let head_id = head
            .id()
            .ok_or_else(|| GitError::OperationFailed("Cannot resolve HEAD".to_string()))?;

        let mut commits = Vec::new();
        let platform = self.gix_repo.rev_walk([head_id]);

        let commit_iter = platform
            .all()
            .map_err(|e| GitError::OperationFailed(format!("Failed to walk commits: {}", e)))?;

        for commit_result in commit_iter.take(limit.unwrap_or(100)) {
            let oid = commit_result
                .map_err(|e| GitError::OperationFailed(format!("Invalid commit: {}", e)))?;

            let commit_obj = self
                .gix_repo
                .find_commit(oid.id)
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

        let commit_obj = self
            .gix_repo
            .find_commit(oid)
            .map_err(|e| GitError::OperationFailed(format!("Cannot find commit: {}", e)))?;

        self.parse_commit(&commit_obj)
    }

    /// Get the diff for a specific commit compared to its first parent.
    #[instrument(skip(self))]
    pub fn get_commit_diff(&self, hash: &str) -> Result<Vec<FileDiff>, GitError> {
        let start = std::time::Instant::now();

        let oid = git2::Oid::from_str(hash)?;
        let commit = self.git2_repo.find_commit(oid)?;
        let commit_tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let diff =
            self.git2_repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;

        let mut file_diffs = Vec::new();

        for (delta_idx, delta) in diff.deltas().enumerate() {
            let path = delta
                .new_file()
                .path()
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

            let patch = git2::Patch::from_diff(&diff, delta_idx)?;
            if let Some(patch) = patch {
                for hunk_idx in 0..patch.num_hunks() {
                    let (hunk, hunk_lines_count) = patch.hunk(hunk_idx)?;
                    let mut hunk_lines = Vec::new();

                    for line_idx in 0..hunk_lines_count {
                        let line = patch.line_in_hunk(hunk_idx, line_idx)?;

                        let (line_type, old_line_num, new_line_num) = match line.origin() {
                            '+' => {
                                additions += 1;
                                (DiffLineType::Addition, None, line.new_lineno())
                            }
                            '-' => {
                                deletions += 1;
                                (DiffLineType::Deletion, line.old_lineno(), None)
                            }
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
    #[instrument(skip(self))]
    pub fn get_author_identity(&self) -> Result<(String, String), GitError> {
        let config = self.git2_repo.config()?;

        let name = match config.get_string("user.name") {
            Ok(name) => name,
            Err(_) => {
                tracing::warn!("user.name not set in git config, using fallback");
                whoami::username()
            }
        };

        let email = match config.get_string("user.email") {
            Ok(email) => email,
            Err(_) => {
                tracing::warn!("user.email not set in git config, using fallback");
                let hostname =
                    whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string());
                format!("{}@{}", whoami::username(), hostname)
            }
        };

        tracing::debug!("Author identity: {} <{}>", name, email);
        Ok((name, email))
    }

    /// Check if there are staged changes ready to commit.
    #[instrument(skip(self))]
    pub fn has_staged_changes(&self) -> Result<bool, GitError> {
        let head = self.git2_repo.head()?;
        let head_tree = head.peel_to_tree()?;

        let mut index = self.git2_repo.index()?;
        let index_tree_oid = index.write_tree()?;
        let index_tree = self.git2_repo.find_tree(index_tree_oid)?;

        let diff = self
            .git2_repo
            .diff_tree_to_tree(Some(&head_tree), Some(&index_tree), None)?;

        let has_changes = diff.deltas().len() > 0;
        tracing::debug!("Has staged changes: {}", has_changes);
        Ok(has_changes)
    }

    /// Create a commit with the current staged changes.
    #[instrument(skip(self, message))]
    pub fn create_commit(&self, message: &str) -> Result<String, GitError> {
        if message.trim().is_empty() {
            return Err(GitError::OperationFailed(
                "Commit message cannot be empty".to_string(),
            ));
        }

        if !self.has_staged_changes()? {
            return Err(GitError::OperationFailed(
                "No staged changes to commit".to_string(),
            ));
        }

        let (author_name, author_email) = self.get_author_identity()?;
        let signature = git2::Signature::now(&author_name, &author_email)?;

        let head = self.git2_repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        let mut index = self.git2_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git2_repo.find_tree(tree_oid)?;

        let commit_oid = self.git2_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        let commit_hash = commit_oid.to_string();
        tracing::info!("Created commit: {}", commit_hash);
        Ok(commit_hash)
    }

    /// Amend the last commit with a new message.
    pub fn amend_commit(&self, message: &str) -> Result<String, GitError> {
        if message.trim().is_empty() {
            return Err(GitError::OperationFailed(
                "Commit message cannot be empty".to_string(),
            ));
        }

        let (author_name, author_email) = self.get_author_identity()?;
        let signature = git2::Signature::now(&author_name, &author_email)?;

        let head = self.git2_repo.head()?;
        let commit_to_amend = head.peel_to_commit()?;
        let original_author = commit_to_amend.author();

        let mut index = self.git2_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git2_repo.find_tree(tree_oid)?;

        let parents: Vec<_> = commit_to_amend.parents().collect();
        let parent_refs: Vec<_> = parents.iter().collect();

        let commit_oid = self.git2_repo.commit(
            Some("HEAD"),
            &original_author,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        let commit_hash = commit_oid.to_string();
        tracing::info!("Amended commit: {}", commit_hash);
        Ok(commit_hash)
    }

    /// Push current branch to remote.
    pub fn push(&self) -> Result<(), GitError> {
        let head = self.git2_repo.head()?;
        if head.is_branch() {
            let branch_name = head.shorthand().ok_or_else(|| {
                GitError::OperationFailed("Could not get branch name".to_string())
            })?;

            let mut remote = self.git2_repo.find_remote("origin").map_err(|e| {
                GitError::OperationFailed(format!("Failed to find remote 'origin': {}", e))
            })?;

            let mut callbacks = git2::RemoteCallbacks::new();
            let mut tried_ssh_agent = false;
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                if allowed_types.contains(git2::CredentialType::SSH_KEY) && !tried_ssh_agent {
                    tried_ssh_agent = true;
                    let username = username_from_url.unwrap_or("git");
                    return git2::Cred::ssh_key_from_agent(username);
                }
                if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                    if let Ok(config) = git2::Config::open_default() {
                        return git2::Cred::credential_helper(&config, url, username_from_url);
                    }
                }
                if allowed_types.contains(git2::CredentialType::DEFAULT) {
                    return git2::Cred::default();
                }
                Err(git2::Error::from_str("no supported authentication methods"))
            });

            let mut push_options = git2::PushOptions::new();
            push_options.remote_callbacks(callbacks);

            let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
            remote
                .push(&[&refspec], Some(&mut push_options))
                .map_err(|e| GitError::OperationFailed(format!("Failed to push: {}", e)))?;

            tracing::info!("Pushed {} to origin", branch_name);
            Ok(())
        } else {
            Err(GitError::OperationFailed(
                "HEAD is not a branch".to_string(),
            ))
        }
    }

    /// Parse a gix commit object into our Commit struct.
    fn parse_commit(&self, commit: &gix::Commit) -> Result<Commit, GitError> {
        let hash = commit.id.to_string();
        let hash_short = hash.chars().take(7).collect();

        let message = commit
            .message_raw()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get commit message: {}", e)))?;
        let message_str = String::from_utf8_lossy(message).to_string();
        let message_summary = message_str.lines().next().unwrap_or("").to_string();

        let author = commit
            .author()
            .map_err(|e| GitError::OperationFailed(format!("Cannot get author: {}", e)))?;
        let committer = commit
            .committer()
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
