use super::*;
use tracing::instrument;

impl GitRepository {
    /// List all local branches with metadata.
    #[instrument(skip(self))]
    pub fn list_local_branches(&self) -> Result<Vec<LocalBranch>, GitError> {
        let mut branches = Vec::new();

        let head = self
            .gix_repo
            .head()
            .map_err(|e| GitError::RefNotFound(format!("Failed to get HEAD: {}", e)))?;

        let current_branch = if !head.is_detached() {
            head.referent_name()
                .and_then(|name| name.shorten().to_string().into())
        } else {
            None
        };

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
                let commit_hash = match r.try_id() {
                    Some(id) => id.to_string(),
                    None => match r.clone().into_fully_peeled_id() {
                        Ok(id) => id.to_string(),
                        Err(_) => {
                            tracing::warn!("Could not resolve branch {}", name);
                            continue;
                        }
                    },
                };
                let is_current = current_branch.as_deref() == Some(&name);
                let upstream = None; // TODO: Implement upstream tracking if needed

                branches.push(LocalBranch {
                    name,
                    commit_hash,
                    is_current,
                    upstream,
                });
            }
        }

        tracing::debug!("Found {} local branches", branches.len());
        Ok(branches)
    }

    /// List all remote branches grouped by remote name.
    #[instrument(skip(self))]
    pub fn list_remote_branches(&self) -> Result<Vec<RemoteBranch>, GitError> {
        let mut remotes = Vec::new();

        let references = self
            .gix_repo
            .references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references.remote_branches().map_err(|e| {
            GitError::OperationFailed(format!("Failed to iterate remote branches: {}", e))
        })?)
        .flatten()
        {
            if let Some(full_name) = r.name().shorten().to_string().into() {
                let parts: Vec<&str> = full_name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let remote = parts[0].to_string();
                    let branch = parts[1].to_string();

                    let commit_hash = match r.try_id() {
                        Some(id) => id.to_string(),
                        None => match r.clone().into_fully_peeled_id() {
                            Ok(id) => id.to_string(),
                            Err(_) => {
                                tracing::warn!(
                                    "Could not resolve remote branch {}/{}",
                                    remote,
                                    branch
                                );
                                continue;
                            }
                        },
                    };

                    remotes.push(RemoteBranch {
                        remote,
                        name: branch,
                        commit_hash,
                    });
                }
            }
        }

        tracing::debug!("Found {} remotes with branches", remotes.len());
        Ok(remotes)
    }

    /// List all tags (annotated and lightweight).
    #[instrument(skip(self))]
    pub fn list_tags(&self) -> Result<Vec<Tag>, GitError> {
        let mut tags = Vec::new();

        let references = self
            .gix_repo
            .references()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get references: {}", e)))?;

        for r in (references
            .tags()
            .map_err(|e| GitError::OperationFailed(format!("Failed to iterate tags: {}", e)))?)
        .flatten()
        {
            if let Some(name) = r.name().shorten().to_string().into() {
                let commit_hash = match r.try_id() {
                    Some(id) => id.to_string(),
                    None => match r.clone().into_fully_peeled_id() {
                        Ok(id) => id.to_string(),
                        Err(_) => {
                            tracing::warn!("Could not resolve tag {}", name);
                            continue;
                        }
                    },
                };

                let message = None; // TODO: Parse tag object for message if needed

                tags.push(Tag {
                    name,
                    commit_hash,
                    message,
                });
            }
        }

        tracing::debug!("Found {} tags", tags.len());
        Ok(tags)
    }

    /// Checkout a branch.
    #[instrument(skip(self))]
    pub fn checkout_branch(&self, branch_name: &str) -> Result<(), GitError> {
        let obj = self
            .git2_repo
            .revparse_single(&format!("refs/heads/{}", branch_name))?;

        self.git2_repo.checkout_tree(&obj, None)?;
        self.git2_repo
            .set_head(&format!("refs/heads/{}", branch_name))?;

        tracing::info!("Checked out branch: {}", branch_name);
        Ok(())
    }

    /// Check if repository has uncommitted changes (staged or unstaged).
    #[instrument(skip(self))]
    pub fn has_uncommitted_changes(&self) -> Result<bool, GitError> {
        let files = self.get_working_dir_status()?;
        let has_changes = !files.is_empty();
        tracing::debug!("Has uncommitted changes: {}", has_changes);
        Ok(has_changes)
    }

    /// Stash all changes (staged and unstaged) with a message.
    #[instrument(skip(self))]
    pub fn stash_changes(&mut self, message: &str) -> Result<String, GitError> {
        let (author_name, author_email) = self.get_author_identity()?;
        let signature = git2::Signature::now(&author_name, &author_email)?;

        let stash_id = self.git2_repo.stash_save(
            &signature,
            message,
            Some(git2::StashFlags::INCLUDE_UNTRACKED),
        )?;

        let stash_name = format!("stash@{{0}} - {}", message);
        tracing::info!("Created stash: {} ({})", stash_name, stash_id);
        Ok(stash_name)
    }

    /// Discard all changes in working directory and index (hard reset to HEAD).
    #[instrument(skip(self))]
    pub fn discard_all_changes(&self) -> Result<(), GitError> {
        let head = self.git2_repo.head()?;
        let obj = head.peel(git2::ObjectType::Commit)?;

        self.git2_repo.reset(&obj, git2::ResetType::Hard, None)?;

        tracing::info!("Discarded all changes (reset --hard HEAD)");
        Ok(())
    }

    /// Check if a local branch exists.
    #[instrument(skip(self))]
    pub fn local_branch_exists(&self, branch_name: &str) -> Result<bool, GitError> {
        let result = self
            .git2_repo
            .find_branch(branch_name, git2::BranchType::Local);
        match result {
            Ok(_) => Ok(true),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Create a local tracking branch from a remote branch.
    /// If force is true, will override existing local branch.
    #[instrument(skip(self))]
    pub fn create_tracking_branch(
        &self,
        remote_branch: &str,
        local_name: &str,
        force: bool,
    ) -> Result<(), GitError> {
        let remote_ref = format!("refs/remotes/{}", remote_branch);
        let remote_commit = self.git2_repo.revparse_single(&remote_ref)?;
        let commit = remote_commit.peel_to_commit()?;

        if force {
            if let Ok(mut existing) = self
                .git2_repo
                .find_branch(local_name, git2::BranchType::Local)
            {
                tracing::info!("Deleting existing branch: {}", local_name);
                existing.delete()?;
            }
        }

        let mut branch = self.git2_repo.branch(local_name, &commit, false)?;
        branch.set_upstream(Some(remote_branch))?;

        let obj = self
            .git2_repo
            .revparse_single(&format!("refs/heads/{}", local_name))?;
        self.git2_repo.checkout_tree(&obj, None)?;
        self.git2_repo
            .set_head(&format!("refs/heads/{}", local_name))?;

        tracing::info!(
            "Created and checked out tracking branch: {} -> {}",
            local_name,
            remote_branch
        );
        Ok(())
    }
}
