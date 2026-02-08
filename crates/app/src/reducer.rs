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
                working_dir_files: Vec::new(),
                commit_message: String::new(),
                author_name: String::new(),
                author_email: String::new(),
            });

            // Add to recent repos and save config
            if !state.config.recent_repos.contains(&path) {
                state.config.recent_repos.insert(0, path.clone());
                state.config.recent_repos.truncate(state.config.max_recent);
            }

            // Auto-load commits, working dir status, and author identity after opening repo
            Effect::Batch(vec![
                Effect::SaveConfig,
                Effect::LoadCommitHistory(path.clone()),
                Effect::LoadWorkingDirStatus(path.clone()),
                Effect::LoadAuthorIdentity(path),
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

        AppMessage::LoadWorkingDirStatusRequested => {
            state.loading = true;
            if let Some(repo) = &state.current_repo {
                Effect::LoadWorkingDirStatus(repo.path.clone())
            } else {
                tracing::warn!("Cannot load working dir status: no repository open");
                Effect::None
            }
        }

        AppMessage::WorkingDirStatusLoaded(files) => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.working_dir_files = files;
                tracing::info!("Loaded {} changed files in working directory", repo.working_dir_files.len());
            }
            Effect::None
        }

        AppMessage::StageFileRequested(file_path) => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::StageFile {
                    repo_path: repo.path.clone(),
                    file_path,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::UnstageFileRequested(file_path) => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::UnstageFile {
                    repo_path: repo.path.clone(),
                    file_path,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::StageAllRequested => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::StageAll(repo.path.clone())
            } else {
                Effect::None
            }
        }

        AppMessage::UnstageAllRequested => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::UnstageAll(repo.path.clone())
            } else {
                Effect::None
            }
        }

        AppMessage::StagingCompleted => {
            state.loading = false;
            state.staging_progress = None;
            if let Some(repo) = &state.current_repo {
                // Refresh working directory status and repository status after staging
                Effect::Batch(vec![
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                    Effect::RefreshRepo(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

        AppMessage::StagingProgress { current, total, operation } => {
            state.staging_progress = Some(crate::state::StagingProgress {
                current,
                total,
                operation,
            });
            Effect::None
        }

        AppMessage::CommitMessageUpdated(message) => {
            if let Some(repo) = &mut state.current_repo {
                repo.commit_message = message;
            }
            Effect::None
        }

        AppMessage::CreateCommitRequested => {
            if let Some(repo) = &state.current_repo {
                let message = repo.commit_message.trim();
                if message.is_empty() {
                    state.error = Some("Commit message cannot be empty".to_string());
                    Effect::None
                } else {
                    state.loading = true;
                    Effect::CreateCommit {
                        repo_path: repo.path.clone(),
                        message: repo.commit_message.clone(),
                    }
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CommitCreated { hash, message: _ } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                // Clear commit message
                repo.commit_message.clear();

                // Show success message temporarily
                tracing::info!("Commit created: {}", hash);

                // Refresh repo data and working directory
                Effect::Batch(vec![
                    Effect::LoadCommitHistory(repo.path.clone()),
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                    Effect::RefreshRepo(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

        AppMessage::AuthorIdentityLoaded { name, email } => {
            if let Some(repo) = &mut state.current_repo {
                repo.author_name = name;
                repo.author_email = email;
            }
            Effect::None
        }
    }
}
