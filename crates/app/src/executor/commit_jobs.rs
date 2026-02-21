use super::run_repo_job;
use crate::AppMessage;
use anyhow::Context;
use crabontree_git::GitRepository;
use std::path::PathBuf;
use tracing::instrument;

/// Execute the LoadCommitHistory job.
#[instrument(skip(path))]
pub(super) async fn execute_load_commit_history(path: PathBuf) -> anyhow::Result<AppMessage> {
    let commits = run_repo_job(path, |repo| {
        repo.get_commit_history(Some(100))
            .context("Failed to get commit history")
    })
    .await?;

    Ok(AppMessage::CommitHistoryLoaded(commits))
}

/// Execute the LoadCommitDiff job.
#[instrument(skip(repo_path, commit_hash))]
pub(super) async fn execute_load_commit_diff(
    repo_path: PathBuf,
    commit_hash: String,
) -> anyhow::Result<AppMessage> {
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

/// Execute the CreateCommit job.
#[instrument(skip(repo_path, message, amend, push))]
pub(super) async fn execute_create_commit(
    repo_path: PathBuf,
    message: String,
    amend: bool,
    push: bool,
) -> anyhow::Result<AppMessage> {
    let message_clone = message.clone();
    let repo_path_clone = repo_path.clone();

    let commit_hash = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        if amend {
            repo.amend_commit(&message_clone)
                .context("Failed to amend commit")
        } else {
            repo.create_commit(&message_clone)
                .context("Failed to create commit")
        }
    })
    .await
    .context("Task panicked")??;

    let push_error = if push {
        let push_result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let repo = GitRepository::open(&repo_path_clone).with_context(|| {
                format!("Failed to open repository at {}", repo_path_clone.display())
            })?;

            repo.push().context("Failed to push to remote")
        })
        .await
        .context("Task panicked")?;

        match push_result {
            Ok(_) => None,
            Err(e) => {
                tracing::warn!("Push failed after commit: {}", e);
                Some(format!("{:#}", e))
            }
        }
    } else {
        None
    };

    Ok(AppMessage::CommitCreated {
        hash: commit_hash,
        message,
        push_error,
    })
}

/// Execute the LoadAuthorIdentity job.
#[instrument(skip(path))]
pub(super) async fn execute_load_author_identity(path: PathBuf) -> anyhow::Result<AppMessage> {
    let (name, email) = run_repo_job(path, |repo| {
        repo.get_author_identity()
            .context("Failed to get author identity")
    })
    .await?;

    Ok(AppMessage::AuthorIdentityLoaded { name, email })
}
