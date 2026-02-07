//! Pure state reducer function.

use crate::{AppMessage, AppState, Effect, RepoState};

/// Pure reducer function that updates state based on messages.
///
/// This function is deterministic and performs no I/O.
pub fn reduce(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
        AppMessage::OpenRepoRequested(path) => {
            state.loading = true;
            state.error = None;
            Effect::OpenRepo(path)
        }

        AppMessage::RepoOpened { path, head, branches, status } => {
            state.loading = false;
            state.current_repo = Some(RepoState {
                path: path.clone(),
                head,
                branches,
                status_summary: status,
                commits: Vec::new(),
                selected_commit: None,
                commit_diff: None,
            });

            // Add to recent repos and save config
            if !state.config.recent_repos.contains(&path) {
                state.config.recent_repos.insert(0, path.clone());
                state.config.recent_repos.truncate(state.config.max_recent);
            }

            // Auto-load commits after opening repo
            Effect::Batch(vec![
                Effect::SaveConfig,
                Effect::LoadCommitHistory(path),
            ])
        }

        AppMessage::CloseRepo => {
            state.current_repo = None;
            Effect::None
        }

        AppMessage::RefreshRepo => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                state.error = None;
                Effect::RefreshRepo(repo.path.clone())
            } else {
                Effect::None
            }
        }

        AppMessage::RepoRefreshed { head, branches, status } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.head = head;
                repo.branches = branches;
                repo.status_summary = status;
            }
            Effect::None
        }

        AppMessage::Error(err) => {
            state.loading = false;
            state.error = Some(err);
            Effect::None
        }

        AppMessage::ClearError => {
            state.error = None;
            Effect::None
        }

        AppMessage::LoadCommitHistoryRequested => {
            state.loading = true;
            if let Some(repo) = &state.current_repo {
                Effect::LoadCommitHistory(repo.path.clone())
            } else {
                tracing::warn!("Cannot load commits: no repository open");
                Effect::None
            }
        }

        AppMessage::CommitHistoryLoaded(commits) => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.commits = commits;
                tracing::info!("Loaded {} commits", repo.commits.len());
            }
            Effect::None
        }

        AppMessage::CommitSelected(hash) => {
            if let Some(repo) = &mut state.current_repo {
                repo.selected_commit = Some(hash.clone());
                repo.commit_diff = None; // Clear previous diff
                state.loading = true;
                Effect::LoadCommitDiff {
                    repo_path: repo.path.clone(),
                    commit_hash: hash,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CommitDeselected => {
            if let Some(repo) = &mut state.current_repo {
                repo.selected_commit = None;
                repo.commit_diff = None;
            }
            Effect::None
        }

        AppMessage::CommitDiffLoaded { commit_hash, diff } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                // Only update if this is still the selected commit
                if repo.selected_commit.as_ref() == Some(&commit_hash) {
                    repo.commit_diff = Some(diff);
                    tracing::info!("Loaded diff for commit {}", commit_hash);
                }
            }
            Effect::None
        }
    }
}
