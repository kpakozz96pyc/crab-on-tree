use super::run_repo_job;
use crate::AppMessage;
use anyhow::Context;
use crabontree_git::GitRepository;
use std::path::PathBuf;
use tracing::instrument;

/// Execute the LoadBranchTree job.
#[instrument(skip(path))]
pub(super) async fn execute_load_branch_tree(path: PathBuf) -> anyhow::Result<AppMessage> {
    use std::collections::{HashMap, HashSet};

    let branch_tree = run_repo_job(path, |repo| {
        let local = repo
            .list_local_branches()
            .context("Failed to list local branches")?;

        let remotes = repo
            .list_remote_branches()
            .context("Failed to list remote branches")?;

        let tags = repo.list_tags().context("Failed to list tags")?;
        let head = repo.get_head().context("Failed to get HEAD")?;

        let local_branches = local
            .into_iter()
            .map(|branch| crate::state::BranchInfo {
                name: branch.name,
                commit_hash: branch.commit_hash,
                is_current: branch.is_current,
                upstream: branch.upstream,
            })
            .collect();

        let remote_branches: HashMap<String, Vec<_>> =
            remotes.into_iter().fold(HashMap::new(), |mut acc, branch| {
                acc.entry(branch.remote)
                    .or_insert_with(Vec::new)
                    .push(crate::state::BranchInfo {
                        name: branch.name,
                        commit_hash: branch.commit_hash,
                        is_current: false,
                        upstream: None,
                    });
                acc
            });

        let tag_list = tags
            .into_iter()
            .map(|tag| crate::state::TagInfo {
                name: tag.name,
                commit_hash: tag.commit_hash,
                message: tag.message,
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
    .await?;

    Ok(AppMessage::BranchTreeLoaded(branch_tree))
}

/// Execute the CheckoutBranch job.
#[instrument(skip(repo_path, branch_name))]
pub(super) async fn execute_checkout_branch(
    repo_path: PathBuf,
    branch_name: String,
) -> anyhow::Result<AppMessage> {
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

/// Execute the CheckUncommittedChanges job.
#[instrument(skip(repo_path, branch_name))]
pub(super) async fn execute_check_uncommitted_changes(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let repo_path_clone1 = repo_path.clone();

    let has_changes = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path_clone1).with_context(|| {
            format!(
                "Failed to open repository at {}",
                repo_path_clone1.display()
            )
        })?;

        repo.has_uncommitted_changes()
            .context("Failed to check for uncommitted changes")
    })
    .await
    .context("Task panicked")??;

    if has_changes {
        Ok(AppMessage::ShowCheckoutWithChangesDialog {
            branch_name,
            is_remote,
        })
    } else if is_remote {
        let local_name = if let Some((_remote, branch)) = branch_name.split_once('/') {
            branch.to_string()
        } else {
            branch_name.clone()
        };

        let local_name_clone = local_name.clone();
        let repo_path_clone2 = repo_path.clone();

        let exists = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let repo = GitRepository::open(&repo_path_clone2).with_context(|| {
                format!(
                    "Failed to open repository at {}",
                    repo_path_clone2.display()
                )
            })?;

            repo.local_branch_exists(&local_name_clone)
                .context("Failed to check if local branch exists")
        })
        .await
        .context("Task panicked")??;

        if exists {
            let repo_path_clone3 = repo_path.clone();
            let local_name_clone2 = local_name.clone();

            tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                let repo = GitRepository::open(&repo_path_clone3).with_context(|| {
                    format!(
                        "Failed to open repository at {}",
                        repo_path_clone3.display()
                    )
                })?;

                repo.checkout_branch(&local_name_clone2)
                    .context("Failed to checkout local branch")
            })
            .await
            .context("Task panicked")??;

            Ok(AppMessage::BranchCheckedOut(local_name))
        } else {
            Ok(AppMessage::CheckoutRemoteOverride {
                remote_branch: branch_name,
                local_name,
            })
        }
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

/// Execute the StashAndCheckout job.
#[instrument(skip(repo_path, branch_name, from_branch))]
pub(super) async fn execute_stash_and_checkout(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
    from_branch: String,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let from_clone = from_branch.clone();
    let repo_path_clone = repo_path.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut repo = GitRepository::open(&repo_path_clone).with_context(|| {
            format!("Failed to open repository at {}", repo_path_clone.display())
        })?;

        let now = chrono::Local::now();
        let stash_msg = format!(
            "WIP: switching from {} to {} - {}",
            from_clone,
            branch_clone,
            now.format("%Y-%m-%d %H:%M:%S")
        );

        let stash_name = repo
            .stash_changes(&stash_msg)
            .context("Failed to stash changes")?;

        tracing::info!("Stashed changes: {}", stash_name);
        Ok(stash_name)
    })
    .await
    .context("Task panicked")??;

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
pub(super) async fn execute_discard_and_checkout(
    repo_path: PathBuf,
    branch_name: String,
    is_remote: bool,
) -> anyhow::Result<AppMessage> {
    let branch_clone = branch_name.clone();
    let repo_path_clone = repo_path.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let repo = GitRepository::open(&repo_path_clone).with_context(|| {
            format!("Failed to open repository at {}", repo_path_clone.display())
        })?;

        repo.discard_all_changes()
            .context("Failed to discard changes")?;

        tracing::info!("Discarded all changes");
        Ok(())
    })
    .await
    .context("Task panicked")??;

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
pub(super) async fn execute_check_local_branch_exists(
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
        Ok(AppMessage::CheckoutRemoteOverride {
            remote_branch,
            local_name,
        })
    }
}

/// Execute the CheckoutRemoteBranch job.
#[instrument(skip(repo_path, remote_branch, local_name))]
pub(super) async fn execute_checkout_remote_branch(
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
