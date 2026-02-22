//! Async job executor.

mod branch_jobs;
mod commit_jobs;
mod file_jobs;
mod repo_jobs;

use crate::{AppMessage, Job, JobId};
use anyhow::Context;
use crabontree_git::GitRepository;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::instrument;

use branch_jobs::*;
use commit_jobs::*;
use file_jobs::*;
use repo_jobs::*;

/// Job executor that runs jobs in a background thread.
pub struct JobExecutor {
    job_tx: mpsc::UnboundedSender<(JobId, Job)>,
}

impl JobExecutor {
    /// Create a new job executor and return the message receiver.
    pub fn new() -> (Self, mpsc::Receiver<AppMessage>) {
        let (job_tx, job_rx) = mpsc::unbounded_channel();
        let (msg_tx, msg_rx) = mpsc::channel(100);

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
                Job::LoadCommitDiff {
                    repo_path,
                    commit_hash,
                } => execute_load_commit_diff(repo_path, commit_hash).await,
                Job::LoadWorkingDirStatus(path) => execute_load_working_dir_status(path).await,
                Job::StageFile {
                    repo_path,
                    file_path,
                } => execute_stage_file(repo_path, file_path).await,
                Job::UnstageFile {
                    repo_path,
                    file_path,
                } => execute_unstage_file(repo_path, file_path).await,
                Job::StageAll(path) => execute_stage_all(path).await,
                Job::UnstageAll(path) => execute_unstage_all(path).await,
                Job::CreateCommit {
                    repo_path,
                    message,
                    amend,
                    push,
                } => execute_create_commit(repo_path, message, amend, push).await,
                Job::LoadAuthorIdentity(path) => execute_load_author_identity(path).await,
                Job::LoadBranchTree(path) => execute_load_branch_tree(path).await,
                Job::CheckoutBranch {
                    repo_path,
                    branch_name,
                } => execute_checkout_branch(repo_path, branch_name).await,
                Job::CheckUncommittedChanges {
                    repo_path,
                    branch_name,
                    is_remote,
                } => execute_check_uncommitted_changes(repo_path, branch_name, is_remote).await,
                Job::StashAndCheckout {
                    repo_path,
                    branch_name,
                    is_remote,
                    from_branch,
                } => {
                    execute_stash_and_checkout(repo_path, branch_name, is_remote, from_branch).await
                }
                Job::DiscardAndCheckout {
                    repo_path,
                    branch_name,
                    is_remote,
                } => execute_discard_and_checkout(repo_path, branch_name, is_remote).await,
                Job::CheckLocalBranchExists {
                    repo_path,
                    remote_branch,
                    local_name,
                } => execute_check_local_branch_exists(repo_path, remote_branch, local_name).await,
                Job::CheckoutRemoteBranch {
                    repo_path,
                    remote_branch,
                    local_name,
                    override_existing,
                } => {
                    execute_checkout_remote_branch(
                        repo_path,
                        remote_branch,
                        local_name,
                        override_existing,
                    )
                    .await
                }
                Job::LoadChangedFiles(path) => execute_load_changed_files(path).await,
                Job::LoadFileContent {
                    repo_path,
                    file_path,
                } => execute_load_file_content(repo_path, file_path).await,
                Job::LoadFileDiff {
                    repo_path,
                    file_path,
                } => execute_load_file_diff(repo_path, file_path).await,
                Job::LoadMultipleFileDiffs {
                    repo_path,
                    file_paths,
                } => execute_load_multiple_file_diffs(repo_path, file_paths).await,
                Job::RevertFile {
                    repo_path,
                    file_path,
                } => execute_revert_file(repo_path, file_path).await,
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

pub(super) async fn run_repo_job<R, F>(path: PathBuf, f: F) -> anyhow::Result<R>
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
