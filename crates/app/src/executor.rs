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
    let commits = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        repo.get_commit_history(Some(100))
            .context("Failed to get commit history")
    })
    .await
    .context("Task panicked")??;

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
