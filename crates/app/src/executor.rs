//! Async job executor.

use crate::{AppMessage, Job, JobId};
use anyhow::Context;
use crabontree_git::GitRepository;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::instrument;

/// Job executor that runs jobs in a background thread.
pub struct JobExecutor {
    job_tx: mpsc::UnboundedSender<(JobId, Job)>,
}

impl JobExecutor {
    /// Create a new job executor and return the message receiver.
    pub fn new() -> (Self, mpsc::Receiver<AppMessage>) {
        let (job_tx, job_rx) = mpsc::unbounded_channel();
        let (msg_tx, msg_rx) = mpsc::channel(100);

        // Spawn worker thread with tokio runtime
        std::thread::spawn(move || {
            worker_thread(job_rx, msg_tx);
        });

        (Self { job_tx }, msg_rx)
    }

    /// Submit a job for execution.
    #[instrument(skip(self))]
    pub fn submit(&self, job: Job) -> JobId {
        let job_id = JobId::new();
        tracing::debug!("Submitting job {:?} with id {:?}", job, job_id);

        if let Err(e) = self.job_tx.send((job_id, job)) {
            tracing::error!("Failed to submit job: {}", e);
        }

        job_id
    }
}

impl Default for JobExecutor {
    fn default() -> Self {
        Self::new().0
    }
}

/// Worker thread that processes jobs.
#[instrument(skip(job_rx, msg_tx))]
fn worker_thread(
    mut job_rx: mpsc::UnboundedReceiver<(JobId, Job)>,
    msg_tx: mpsc::Sender<AppMessage>,
) {
    tracing::info!("Worker thread starting");

    // Create tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    runtime.block_on(async {
        while let Some((job_id, job)) = job_rx.recv().await {
            tracing::debug!("Processing job {:?}", job_id);

            let result = match job {
                Job::OpenRepo(path) => execute_open_repo(path).await,
                Job::RefreshRepo(path) => execute_refresh_repo(path).await,
                Job::LoadCommitHistory(path) => execute_load_commit_history(path).await,
                Job::LoadCommitDiff { repo_path, commit_hash } =>
                    execute_load_commit_diff(repo_path, commit_hash).await,
                Job::LoadWorkingDirStatus(path) => execute_load_working_dir_status(path).await,
                Job::StageFile { repo_path, file_path } =>
                    execute_stage_file(repo_path, file_path).await,
                Job::UnstageFile { repo_path, file_path } =>
                    execute_unstage_file(repo_path, file_path).await,
                Job::StageAll(path) => execute_stage_all(path).await,
                Job::UnstageAll(path) => execute_unstage_all(path).await,
                Job::CreateCommit { repo_path, message } =>
                    execute_create_commit(repo_path, message).await,
                Job::LoadAuthorIdentity(path) => execute_load_author_identity(path).await,
                Job::LoadBranchTree(path) => execute_load_branch_tree(path).await,
                Job::CheckoutBranch { repo_path, branch_name } =>
                    execute_checkout_branch(repo_path, branch_name).await,
                Job::CheckUncommittedChanges { repo_path, branch_name, is_remote } =>
                    execute_check_uncommitted_changes(repo_path, branch_name, is_remote).await,
                Job::StashAndCheckout { repo_path, branch_name, is_remote, from_branch } =>
                    execute_stash_and_checkout(repo_path, branch_name, is_remote, from_branch).await,
                Job::DiscardAndCheckout { repo_path, branch_name, is_remote } =>
                    execute_discard_and_checkout(repo_path, branch_name, is_remote).await,
                Job::CheckLocalBranchExists { repo_path, remote_branch, local_name } =>
                    execute_check_local_branch_exists(repo_path, remote_branch, local_name).await,
                Job::CheckoutRemoteBranch { repo_path, remote_branch, local_name, override_existing } =>
                    execute_checkout_remote_branch(repo_path, remote_branch, local_name, override_existing).await,
                Job::LoadChangedFiles(path) => execute_load_changed_files(path).await,
                Job::LoadFileContent { repo_path, file_path } =>
                    execute_load_file_content(repo_path, file_path).await,
                Job::LoadFileDiff { repo_path, file_path } =>
                    execute_load_file_diff(repo_path, file_path).await,
            };

            match result {
                Ok(message) => {
                    if msg_tx.send(message).await.is_err() {
                        tracing::error!("Failed to send result message (receiver dropped)");
                        break;
                    }
                }
                Err(e) => {
                    let error_msg = format!("{:#}", e);
                    tracing::error!("Job {:?} failed: {}", job_id, error_msg);

                    if msg_tx.send(AppMessage::Error(error_msg)).await.is_err() {
                        tracing::error!("Failed to send error message (receiver dropped)");
                        break;
                    }
                }
            }
        }

        tracing::info!("Worker thread shutting down");
    });
}

async fn run_repo_job<R, F>(path: PathBuf, f: F) -> anyhow::Result<R>
where
    R: Send + 'static,
    F: FnOnce(GitRepository) -> anyhow::Result<R> + Send + 'static,
{
    tokio::task::spawn_blocking(move || -> anyhow::Result<R> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;
        f(repo)
    })
    .await
    .context("Task panicked")?
}

/// Execute the OpenRepo job.
#[instrument(skip(path))]
async fn execute_open_repo(path: PathBuf) -> anyhow::Result<AppMessage> {
    // Run blocking Git operations in a separate task
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let head = repo.get_head()
            .context("Failed to get HEAD")?;

        let branches = repo.get_branches()
            .context("Failed to get branches")?;

        let status = repo.get_status()
            .context("Failed to get status")?;

        Ok((path, head, branches, status))
    })
    .await
    .context("Task panicked")??;

    let (path, head, branches, status) = result;

    Ok(AppMessage::RepoOpened {
        path,
        head,
        branches,
        status,
    })
}

/// Execute the RefreshRepo job.
#[instrument(skip(path))]
async fn execute_refresh_repo(path: PathBuf) -> anyhow::Result<AppMessage> {
    // Run blocking Git operations in a separate task
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let head = repo.get_head()
            .context("Failed to get HEAD")?;

        let branches = repo.get_branches()
            .context("Failed to get branches")?;

        let status = repo.get_status()
            .context("Failed to get status")?;

        Ok((head, branches, status))
    })
    .await
    .context("Task panicked")??;

    let (head, branches, status) = result;

    Ok(AppMessage::RepoRefreshed {
        head,
        branches,
        status,
    })
}

/// Execute the LoadCommitHistory job.
#[instrument(skip(path))]
async fn execute_load_commit_history(path: PathBuf) -> anyhow::Result<AppMessage> {
    let commits = run_repo_job(path, |repo| {
        repo.get_commit_history(Some(100))
            .context("Failed to get commit history")
    })
    .await?;

    Ok(AppMessage::CommitHistoryLoaded(commits))
}

/// Execute the LoadCommitDiff job.
#[instrument(skip(repo_path, commit_hash))]
async fn execute_load_commit_diff(repo_path: PathBuf, commit_hash: String) -> anyhow::Result<AppMessage> {
    let hash_clone = commit_hash.clone();
    let diff = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.get_commit_diff(&hash_clone)
            .context("Failed to get commit diff")
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::CommitDiffLoaded { commit_hash, diff })
}

/// Execute the LoadWorkingDirStatus job.
#[instrument(skip(path))]
async fn execute_load_working_dir_status(path: PathBuf) -> anyhow::Result<AppMessage> {
    let files = run_repo_job(path, |repo| {
        repo.get_working_dir_status()
            .context("Failed to get working directory status")
    })
    .await?;

    Ok(AppMessage::WorkingDirStatusLoaded(files))
}

/// Execute the StageFile job.
#[instrument(skip(repo_path, file_path))]
async fn execute_stage_file(repo_path: PathBuf, file_path: PathBuf) -> anyhow::Result<AppMessage> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.stage_file(&file_path)
            .with_context(|| format!("Failed to stage file {}", file_path.display()))?;

        Ok(())
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::StagingCompleted)
}

/// Execute the UnstageFile job.
#[instrument(skip(repo_path, file_path))]
async fn execute_unstage_file(repo_path: PathBuf, file_path: PathBuf) -> anyhow::Result<AppMessage> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.unstage_file(&file_path)
            .with_context(|| format!("Failed to unstage file {}", file_path.display()))?;

        Ok(())
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::StagingCompleted)
}

/// Execute the StageAll job.
#[instrument(skip(path))]
async fn execute_stage_all(path: PathBuf) -> anyhow::Result<AppMessage> {
    run_repo_job(path, |repo| {
        repo.stage_all()
        .context("Failed to stage all changes")?;

        Ok(())
    })
    .await?;

    Ok(AppMessage::StagingCompleted)
}

/// Execute the UnstageAll job.
#[instrument(skip(path))]
async fn execute_unstage_all(path: PathBuf) -> anyhow::Result<AppMessage> {
    run_repo_job(path, |repo| {
        repo.unstage_all()
            .context("Failed to unstage all changes")?;

        Ok(())
    })
    .await?;

    Ok(AppMessage::StagingCompleted)
}

/// Execute the CreateCommit job.
#[instrument(skip(repo_path, message))]
async fn execute_create_commit(repo_path: PathBuf, message: String) -> anyhow::Result<AppMessage> {
    let message_clone = message.clone();
    let commit_hash = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.create_commit(&message_clone)
            .context("Failed to create commit")
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::CommitCreated {
        hash: commit_hash,
        message,
    })
}

/// Execute the LoadAuthorIdentity job.
#[instrument(skip(path))]
async fn execute_load_author_identity(path: PathBuf) -> anyhow::Result<AppMessage> {
    let (name, email) = run_repo_job(path, |repo| {
        repo.get_author_identity()
            .context("Failed to get author identity")
    })
    .await?;

    Ok(AppMessage::AuthorIdentityLoaded { name, email })
}

/// Execute the LoadBranchTree job.
#[instrument(skip(path))]
async fn execute_load_branch_tree(path: PathBuf) -> anyhow::Result<AppMessage> {
    use std::collections::{HashMap, HashSet};

    let branch_tree = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let local = repo.list_local_branches()
            .context("Failed to list local branches")?;

        let remotes = repo.list_remote_branches()
            .context("Failed to list remote branches")?;

        let tags = repo.list_tags()
            .context("Failed to list tags")?;

        let head = repo.get_head()
            .context("Failed to get HEAD")?;

        let local_branches = local.into_iter()
            .map(|(name, hash, is_current, upstream)| crate::state::BranchInfo {
                name,
                commit_hash: hash,
                is_current,
                upstream,
            })
            .collect();

        let remote_branches: HashMap<String, Vec<_>> = remotes.into_iter()
            .map(|(remote, branches)| {
                let branch_infos = branches.into_iter()
                    .map(|(name, hash)| crate::state::BranchInfo {
                        name,
                        commit_hash: hash,
                        is_current: false,
                        upstream: None,
                    })
                    .collect();
                (remote, branch_infos)
            })
            .collect();

        let tag_list = tags.into_iter()
            .map(|(name, hash, message)| crate::state::TagInfo {
                name,
                commit_hash: hash,
                message,
            })
            .collect();

        let mut expanded = HashSet::new();
        expanded.insert("local".to_string());

        Ok(crate::state::BranchTreeState {
            local_branches,
            remote_branches,
            tags: tag_list,
            current_branch: head,
            expanded_sections: expanded,
            selected_branch: None,
        })
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::BranchTreeLoaded(branch_tree))
}

/// Execute the CheckoutBranch job.
#[instrument(skip(repo_path, branch_name))]
async fn execute_checkout_branch(repo_path: PathBuf, branch_name: String) -> anyhow::Result<AppMessage> {
    let name_clone = branch_name.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.checkout_branch(&name_clone)
            .with_context(|| format!("Failed to checkout branch {}", name_clone))?;

        Ok(())
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::BranchCheckedOut(branch_name))
}

/// Execute the LoadChangedFiles job.
#[instrument(skip(path))]
async fn execute_load_changed_files(path: PathBuf) -> anyhow::Result<AppMessage> {
    let changed_files = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let files = repo.get_working_dir_status()
            .context("Failed to get working directory status")?;

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicted = Vec::new();

        for file in files {
            if file.status == crabontree_git::WorkingDirStatus::Conflicted {
                conflicted.push(file);
            } else if file.is_staged {
                staged.push(file);
            } else if file.status == crabontree_git::WorkingDirStatus::Untracked {
                untracked.push(file);
            } else {
                unstaged.push(file);
            }
        }

        Ok(crate::state::ChangedFilesState {
            staged,
            unstaged,
            untracked,
            conflicted,
            selected_file: None,
            commit_message: String::new(), // Empty for working directory
            is_commit_view: false, // This is working directory view
        })
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::ChangedFilesLoaded(changed_files))
}

/// Execute the LoadFileContent job.
#[instrument(skip(repo_path, file_path))]
async fn execute_load_file_content(repo_path: PathBuf, file_path: PathBuf) -> anyhow::Result<AppMessage> {
    let path_clone = file_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        // Check if binary
        let is_binary = repo.is_binary_file(&path_clone)
            .context("Failed to check if file is binary")?;

        if is_binary {
            let size = repo_path.join(&path_clone).metadata()
                .map(|m| m.len())
                .unwrap_or(0);
            return Ok((None, Some(size)));
        }

        let content = repo.get_file_content(&path_clone)
            .context("Failed to get file content")?;

        Ok((Some(content), None))
    })
    .await
    .context("Task panicked")??;

    match result {
        (Some(content), None) => {
            // Detect language from extension
            let language = file_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_string());

            Ok(AppMessage::FileContentLoaded {
                path: file_path,
                content,
                language,
            })
        }
        (None, Some(size)) => {
            Ok(AppMessage::BinaryFileDetected {
                path: file_path,
                size,
            })
        }
        _ => Err(anyhow::anyhow!("Unexpected result from file content loading")),
    }
}

/// Execute the LoadFileDiff job.
#[instrument(skip(repo_path, file_path))]
async fn execute_load_file_diff(repo_path: PathBuf, file_path: PathBuf) -> anyhow::Result<AppMessage> {
    let path_clone = file_path.clone();
    let hunks = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.get_file_diff(&path_clone)
            .context("Failed to get file diff")
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::FileDiffLoaded {
        path: file_path,
        hunks,
    })
}

/// Execute the CheckUncommittedChanges job.
#[instrument(skip(repo_path, branch_name))]
async fn execute_check_uncommitted_changes(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let repo_path_clone1 = repo_path.clone();

    let has_changes = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path_clone1)
            .with_context(|| format!("Failed to open repository at {}", repo_path_clone1.display()))?;

        repo.has_uncommitted_changes()
            .context("Failed to check for uncommitted changes")
    })
    .await
    .context("Task panicked")??;

    if has_changes {
        // Has changes - show dialog
        Ok(AppMessage::ShowCheckoutWithChangesDialog {
            branch_name,
            is_remote,
        })
    } else if is_remote {
        // No changes but remote branch - check if local name exists
        let local_name = if let Some((_remote, branch)) = branch_name.split_once('/') {
            branch.to_string()
        } else {
            branch_name.clone()
        };

        let local_name_clone = local_name.clone();
        let repo_path_clone2 = repo_path.clone();

        let exists = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let repo = GitRepository::open(&repo_path_clone2)
                .with_context(|| format!("Failed to open repository at {}", repo_path_clone2.display()))?;

            repo.local_branch_exists(&local_name_clone)
                .context("Failed to check if local branch exists")
        })
        .await
        .context("Task panicked")??;

        if exists {
            // Local branch exists - just checkout the local branch instead
            let repo_path_clone3 = repo_path.clone();
            let local_name_clone2 = local_name.clone();

            tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                let repo = GitRepository::open(&repo_path_clone3)
                    .with_context(|| format!("Failed to open repository at {}", repo_path_clone3.display()))?;

                repo.checkout_branch(&local_name_clone2)
                    .context("Failed to checkout local branch")
            })
            .await
            .context("Task panicked")??;

            Ok(AppMessage::BranchCheckedOut(local_name))
        } else {
            // No local branch - create tracking branch
            Ok(AppMessage::CheckoutRemoteOverride {
                remote_branch: branch_name,
                local_name,
            })
        }
    } else {
        // No changes and local branch - checkout directly
        tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let repo = GitRepository::open(&repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

            repo.checkout_branch(&branch_clone)
                .context("Failed to checkout branch")
        })
        .await
        .context("Task panicked")??;

        Ok(AppMessage::BranchCheckedOut(branch_name))
    }
}

/// Execute the StashAndCheckout job.
#[instrument(skip(repo_path, branch_name, from_branch))]
async fn execute_stash_and_checkout(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
    from_branch: String,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let from_clone = from_branch.clone();
    let repo_path_clone = repo_path.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut repo = GitRepository::open(&repo_path_clone)
            .with_context(|| format!("Failed to open repository at {}", repo_path_clone.display()))?;

        // Create stash message with datetime
        let now = chrono::Local::now();
        let stash_msg = format!(
            "WIP: switching from {} to {} - {}",
            from_clone,
            branch_clone,
            now.format("%Y-%m-%d %H:%M:%S")
        );

        // Stash changes
        let stash_name = repo.stash_changes(&stash_msg)
            .context("Failed to stash changes")?;

        tracing::info!("Stashed changes: {}", stash_name);

        Ok(stash_name)
    })
    .await
    .context("Task panicked")??;

    // After stashing, proceed with checkout
    if is_remote {
        let local_name = if let Some((_remote, branch)) = branch_name.split_once('/') {
            branch.to_string()
        } else {
            branch_name.clone()
        };

        Ok(AppMessage::CheckoutRemoteOverride {
            remote_branch: branch_name,
            local_name,
        })
    } else {
        let branch_name_clone = branch_name.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let repo = GitRepository::open(&repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

            repo.checkout_branch(&branch_name_clone)
                .context("Failed to checkout branch")
        })
        .await
        .context("Task panicked")??;

        Ok(AppMessage::BranchCheckedOut(branch_name))
    }
}

/// Execute the DiscardAndCheckout job.
#[instrument(skip(repo_path, branch_name))]
async fn execute_discard_and_checkout(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let repo_path_clone = repo_path.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path_clone)
            .with_context(|| format!("Failed to open repository at {}", repo_path_clone.display()))?;

        // Discard all changes
        repo.discard_all_changes()
            .context("Failed to discard changes")?;

        tracing::info!("Discarded all changes");

        Ok(())
    })
    .await
    .context("Task panicked")??;

    // After discarding, proceed with checkout
    if is_remote {
        let local_name = if let Some((_remote, branch)) = branch_name.split_once('/') {
            branch.to_string()
        } else {
            branch_name.clone()
        };

        Ok(AppMessage::CheckoutRemoteOverride {
            remote_branch: branch_name,
            local_name,
        })
    } else {
        tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let repo = GitRepository::open(&repo_path)
                .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

            repo.checkout_branch(&branch_clone)
                .context("Failed to checkout branch")
        })
        .await
        .context("Task panicked")??;

        Ok(AppMessage::BranchCheckedOut(branch_name))
    }
}

/// Execute the CheckLocalBranchExists job.
#[instrument(skip(repo_path, remote_branch, local_name))]
async fn execute_check_local_branch_exists(
    repo_path: PathBuf,
    remote_branch: String,
    local_name: String,
) -> anyhow::Result<AppMessage> {
    let local_clone = local_name.clone();

    let exists = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.local_branch_exists(&local_clone)
            .context("Failed to check if local branch exists")
    })
    .await
    .context("Task panicked")??;

    if exists {
        Ok(AppMessage::ShowRemoteBranchConflictDialog {
            remote_branch,
            local_name,
        })
    } else {
        // No conflict - create tracking branch
        Ok(AppMessage::CheckoutRemoteOverride {
            remote_branch,
            local_name,
        })
    }
}

/// Execute the CheckoutRemoteBranch job.
#[instrument(skip(repo_path, remote_branch, local_name))]
async fn execute_checkout_remote_branch(
    repo_path: PathBuf,
    remote_branch: String,
    local_name: String,
    override_existing: bool,
) -> anyhow::Result<AppMessage> {
    let remote_clone = remote_branch.clone();
    let local_clone = local_name.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        repo.create_tracking_branch(&remote_clone, &local_clone, override_existing)
            .context("Failed to create tracking branch")
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::BranchCheckedOut(local_name))
}
