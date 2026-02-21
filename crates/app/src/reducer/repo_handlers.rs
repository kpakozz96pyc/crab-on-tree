use crate::{AppMessage, AppState, Effect, RepoState};

pub(super) fn handle(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
        AppMessage::OpenRepoRequested(path) => {
            state.loading = true;
            state.error = None;
            Effect::OpenRepo(path)
        }

        AppMessage::RepoOpened {
            path,
            head,
            branches,
            status,
        } => {
            state.loading = false;
            state.current_repo = Some(RepoState::new(path.clone(), head, branches, status));

            if !state.config.recent_repos.contains(&path) {
                state.config.recent_repos.insert(0, path.clone());
                state.config.recent_repos.truncate(state.config.max_recent);
            }

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

        AppMessage::RepoRefreshed {
            head,
            branches,
            status,
        } => {
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
            state.committing = false;
            state.error = Some(err);
            Effect::None
        }

        AppMessage::ClearError => {
            state.error = None;
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
                tracing::info!(
                    "Loaded {} changed files in working directory",
                    repo.working_dir_files.len()
                );
            }
            Effect::None
        }

        _ => unreachable!("repo_handlers received unexpected message"),
    }
}
