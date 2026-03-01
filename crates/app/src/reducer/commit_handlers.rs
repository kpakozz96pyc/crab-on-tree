use super::helpers::commit_view::{
    build_commit_info, build_commit_view_changed_files, commit_message,
};
use crate::{AppMessage, AppState, Effect};

pub(super) fn handle(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
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
                repo.commit_diff = None;
                repo.file_view = crate::state::FileViewState::None;

                if hash == crate::WORKING_DIR_HASH {
                    Effect::LoadChangedFiles(repo.path.clone())
                } else {
                    state.loading = true;
                    Effect::LoadCommitDiff {
                        repo_path: repo.path.clone(),
                        commit_hash: hash,
                    }
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CommitDeselected => {
            if let Some(repo) = &mut state.current_repo {
                repo.selected_commit = None;
                repo.commit_diff = None;
                repo.changed_files = None;
            }
            Effect::None
        }

        AppMessage::CommitDiffLoaded { commit_hash, diff } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                if repo.selected_commit.as_ref() == Some(&commit_hash) {
                    repo.commit_diff = Some(diff.clone());
                    tracing::info!("Loaded diff for commit {}", commit_hash);
                    let message = commit_message(&repo.commits, &commit_hash);
                    let info =
                        build_commit_info(&repo.commits, repo.branch_tree.as_ref(), &commit_hash);
                    repo.changed_files =
                        Some(build_commit_view_changed_files(&diff, message, info));
                }
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
                Effect::Batch(vec![
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                    Effect::LoadChangedFiles(repo.path.clone()),
                    Effect::RefreshRepo(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

        AppMessage::StagingProgress {
            current,
            total,
            operation,
        } => {
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
                        amend: false,
                        push: false,
                    }
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CommitCreated {
            hash,
            message: _,
            push_error,
        } => {
            state.loading = false;
            state.committing = false;

            if let Some(err) = push_error {
                state.error = Some(format!("Commit created but push failed: {}", err));
            }

            if let Some(repo) = &mut state.current_repo {
                repo.commit_message.clear();
                repo.file_view = crate::state::FileViewState::None;

                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.commit_summary.clear();
                    changed_files.commit_description.clear();
                    changed_files.amend_last_commit = false;
                    changed_files.selected_file = None;
                    changed_files.selected_files.clear();
                }

                let repo_key = repo.path.to_string_lossy().to_string();
                state.config.commit_drafts.remove(&repo_key);

                tracing::info!("Commit created: {}", hash);

                Effect::Batch(vec![
                    Effect::SaveConfig,
                    Effect::LoadCommitHistory(repo.path.clone()),
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                    Effect::LoadChangedFiles(repo.path.clone()),
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

        AppMessage::CommitSummaryUpdated(summary) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.commit_summary = summary;
                }
            }
            Effect::None
        }

        AppMessage::CommitDescriptionUpdated(description) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.commit_description = description;
                }
            }
            Effect::None
        }

        AppMessage::AmendLastCommitToggled(amend) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.amend_last_commit = amend;
                }
            }
            Effect::None
        }

        AppMessage::PushAfterCommitToggled(push) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.push_after_commit = push;
                }
            }
            Effect::None
        }

        AppMessage::CommitChangesRequested {
            summary,
            description,
            amend,
            push,
        } => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                state.committing = true;
                let full_message = if description.is_empty() {
                    summary.clone()
                } else {
                    format!("{}\n\n{}", summary, description)
                };

                Effect::CreateCommit {
                    repo_path: repo.path.clone(),
                    message: full_message,
                    amend,
                    push,
                }
            } else {
                Effect::None
            }
        }

        _ => unreachable!("commit_handlers received unexpected message"),
    }
}
