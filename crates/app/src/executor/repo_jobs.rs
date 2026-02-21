use super::run_repo_job;
use crate::AppMessage;
use anyhow::Context;
use crabontree_git::GitRepository;
use std::path::PathBuf;
use tracing::instrument;

/// Execute the OpenRepo job.
#[instrument(skip(path))]
pub(super) async fn execute_open_repo(path: PathBuf) -> anyhow::Result<AppMessage> {
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let head = repo.get_head().context("Failed to get HEAD")?;
        let branches = repo.get_branches().context("Failed to get branches")?;
        let status = repo.get_status().context("Failed to get status")?;

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
pub(super) async fn execute_refresh_repo(path: PathBuf) -> anyhow::Result<AppMessage> {
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path.display()))?;

        let head = repo.get_head().context("Failed to get HEAD")?;
        let branches = repo.get_branches().context("Failed to get branches")?;
        let status = repo.get_status().context("Failed to get status")?;

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

/// Execute the LoadWorkingDirStatus job.
#[instrument(skip(path))]
pub(super) async fn execute_load_working_dir_status(path: PathBuf) -> anyhow::Result<AppMessage> {
    let files = run_repo_job(path, |repo| {
        repo.get_working_dir_status()
            .context("Failed to get working directory status")
    })
    .await?;

    Ok(AppMessage::WorkingDirStatusLoaded(files))
}

/// Execute the StageFile job.
#[instrument(skip(repo_path, file_path))]
pub(super) async fn execute_stage_file(
    repo_path: PathBuf,
    file_path: PathBuf,
) -> anyhow::Result<AppMessage> {
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
pub(super) async fn execute_unstage_file(
    repo_path: PathBuf,
    file_path: PathBuf,
) -> anyhow::Result<AppMessage> {
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
pub(super) async fn execute_stage_all(path: PathBuf) -> anyhow::Result<AppMessage> {
    run_repo_job(path, |repo| {
        repo.stage_all().context("Failed to stage all changes")?;
        Ok(())
    })
    .await?;

    Ok(AppMessage::StagingCompleted)
}

/// Execute the UnstageAll job.
#[instrument(skip(path))]
pub(super) async fn execute_unstage_all(path: PathBuf) -> anyhow::Result<AppMessage> {
    run_repo_job(path, |repo| {
        repo.unstage_all()
            .context("Failed to unstage all changes")?;
        Ok(())
    })
    .await?;

    Ok(AppMessage::StagingCompleted)
}
