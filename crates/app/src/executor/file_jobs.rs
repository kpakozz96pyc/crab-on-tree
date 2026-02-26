use super::run_repo_job;
use crate::AppMessage;
use anyhow::Context;
use crabontree_git::{GitRepository, WorkingDirStatus};
use std::path::PathBuf;
use tracing::instrument;

/// Execute the LoadChangedFiles job.
#[instrument(skip(path))]
pub(super) async fn execute_load_changed_files(path: PathBuf) -> anyhow::Result<AppMessage> {
    let changed_files = run_repo_job(path, |repo| {
        let files = repo
            .get_working_dir_status()
            .context("Failed to get working directory status")?;

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicted = Vec::new();

        for file in files {
            if file.status == WorkingDirStatus::Conflicted {
                conflicted.push(file);
            } else if file.is_staged {
                staged.push(file);
            } else if file.status == WorkingDirStatus::Untracked {
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
            selected_files: std::collections::HashSet::new(),
            last_clicked_file: None,
            commit_message: String::new(),
            is_commit_view: false,
            commit_info: None,
            commit_summary: String::new(),
            commit_description: String::new(),
            amend_last_commit: false,
            push_after_commit: false,
        })
    })
    .await?;

    Ok(AppMessage::ChangedFilesLoaded(changed_files))
}

/// Execute the LoadFileContent job.
#[instrument(skip(repo_path, file_path))]
pub(super) async fn execute_load_file_content(
    repo_path: PathBuf,
    file_path: PathBuf,
) -> anyhow::Result<AppMessage> {
    let path_clone = file_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        let is_binary = repo
            .is_binary_file(&path_clone)
            .context("Failed to check if file is binary")?;

        if is_binary {
            let size = repo_path
                .join(&path_clone)
                .metadata()
                .map(|m| m.len())
                .unwrap_or(0);
            return Ok((None, Some(size)));
        }

        let content = repo
            .get_file_content(&path_clone)
            .context("Failed to get file content")?;

        Ok((Some(content), None))
    })
    .await
    .context("Task panicked")??;

    match result {
        (Some(content), None) => {
            let language = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_string());

            Ok(AppMessage::FileContentLoaded {
                path: file_path,
                content,
                language,
            })
        }
        (None, Some(size)) => Ok(AppMessage::BinaryFileDetected {
            path: file_path,
            size,
        }),
        _ => Err(anyhow::anyhow!(
            "Unexpected result from file content loading"
        )),
    }
}

/// Execute the LoadFileDiff job.
#[instrument(skip(repo_path, file_path))]
pub(super) async fn execute_load_file_diff(
    repo_path: PathBuf,
    file_path: PathBuf,
) -> anyhow::Result<AppMessage> {
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

/// Execute the RevertFile job.
#[instrument(skip(repo_path, file_path))]
pub(super) async fn execute_revert_file(
    repo_path: PathBuf,
    file_path: PathBuf,
) -> anyhow::Result<AppMessage> {
    run_repo_job(repo_path, move |repo| {
        repo.revert_file(&file_path)
            .context("Failed to revert file")?;
        Ok(AppMessage::RevertFileCompleted)
    })
    .await
}

/// Execute the LoadMultipleFileDiffs job.
#[instrument(skip(repo_path, file_paths))]
pub(super) async fn execute_load_multiple_file_diffs(
    repo_path: PathBuf,
    file_paths: Vec<PathBuf>,
) -> anyhow::Result<AppMessage> {
    let paths_clone = file_paths.clone();
    let files_with_diffs = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path)
            .with_context(|| format!("Failed to open repository at {}", repo_path.display()))?;

        let mut results = Vec::new();
        for path in paths_clone {
            match repo.get_file_diff(&path) {
                Ok(hunks) => results.push((path, hunks)),
                Err(e) => {
                    tracing::warn!("Failed to get diff for {}: {}", path.display(), e);
                }
            }
        }
        Ok(results)
    })
    .await
    .context("Task panicked")??;

    Ok(AppMessage::MultipleFileDiffsLoaded {
        files: files_with_diffs,
    })
}
