use crate::{AppMessage, AppState, Effect};

pub(super) fn handle(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
        AppMessage::LoadChangedFilesRequested => {
            state.loading = true;
            if let Some(repo) = &state.current_repo {
                Effect::LoadChangedFiles(repo.path.clone())
            } else {
                tracing::warn!("Cannot load changed files: no repository open");
                Effect::None
            }
        }

        AppMessage::ChangedFilesLoaded(mut changed_files) => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                if let Some(old_files) = &repo.changed_files {
                    changed_files.commit_summary = old_files.commit_summary.clone();
                    changed_files.commit_description = old_files.commit_description.clone();
                    changed_files.amend_last_commit = old_files.amend_last_commit;
                    changed_files.push_after_commit = old_files.push_after_commit;
                } else if !changed_files.is_commit_view {
                    let repo_key = repo.path.to_string_lossy().to_string();
                    if let Some(draft) = state.config.commit_drafts.get(&repo_key) {
                        changed_files.commit_summary = draft.summary.clone();
                        changed_files.commit_description = draft.description.clone();
                    }
                }
                repo.changed_files = Some(changed_files);
                tracing::info!("Loaded changed files");
            }
            Effect::None
        }

        AppMessage::ChangedFileSelected(path) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(files) = &mut repo.changed_files {
                    files.selected_file = Some(path.clone());
                    files.selected_files.clear();
                    files.selected_files.insert(path.clone());
                    files.last_clicked_file = Some(path.clone());
                }

                let is_viewing_commit = repo
                    .selected_commit
                    .as_ref()
                    .map(|hash| hash != crate::WORKING_DIR_HASH)
                    .unwrap_or(false);

                if is_viewing_commit {
                    if let Some(commit_diff) = &repo.commit_diff {
                        let path_str = path.to_string_lossy();
                        if let Some(file_diff) = commit_diff.iter().find(|fd| fd.path == path_str) {
                            repo.file_view = crate::state::FileViewState::Diff {
                                path: path.clone(),
                                hunks: file_diff.hunks.clone(),
                                view_mode: crate::state::DiffViewMode::Unified,
                            };
                            Effect::None
                        } else {
                            Effect::None
                        }
                    } else {
                        Effect::None
                    }
                } else {
                    let is_untracked = repo
                        .changed_files
                        .as_ref()
                        .map(|files| files.untracked.iter().any(|f| f.path == path))
                        .unwrap_or(false);

                    state.loading = true;
                    if is_untracked {
                        Effect::LoadFileContent {
                            repo_path: repo.path.clone(),
                            file_path: path,
                        }
                    } else {
                        Effect::LoadFileDiff {
                            repo_path: repo.path.clone(),
                            file_path: path,
                        }
                    }
                }
            } else {
                Effect::None
            }
        }

        AppMessage::SelectFileWithModifiers { path, ctrl, shift } => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(files) = &mut repo.changed_files {
                    if ctrl {
                        if files.selected_files.contains(&path) {
                            files.selected_files.remove(&path);
                        } else {
                            files.selected_files.insert(path.clone());
                        }
                        files.last_clicked_file = Some(path.clone());
                        files.selected_file = Some(path);
                    } else if shift {
                        if let Some(last_clicked) = &files.last_clicked_file {
                            let mut all_files = Vec::new();
                            all_files.extend(files.staged.iter().map(|f| &f.path));
                            all_files.extend(files.unstaged.iter().map(|f| &f.path));
                            all_files.extend(files.untracked.iter().map(|f| &f.path));
                            all_files.extend(files.conflicted.iter().map(|f| &f.path));

                            let last_idx = all_files.iter().position(|p| *p == last_clicked);
                            let current_idx = all_files.iter().position(|p| *p == &path);

                            if let (Some(start), Some(end)) = (last_idx, current_idx) {
                                let (start, end) = if start < end {
                                    (start, end)
                                } else {
                                    (end, start)
                                };
                                for selected in all_files.iter().take(end + 1).skip(start) {
                                    files.selected_files.insert((*selected).clone());
                                }
                            }
                        }
                        files.selected_file = Some(path);
                    }

                    if files.selected_files.len() > 1 {
                        let is_viewing_commit = repo
                            .selected_commit
                            .as_ref()
                            .map(|hash| hash != crate::WORKING_DIR_HASH)
                            .unwrap_or(false);

                        if is_viewing_commit {
                            if let Some(commit_diff) = &repo.commit_diff {
                                let selected: std::collections::HashSet<_> =
                                    files.selected_files.iter().collect();
                                let multi: Vec<_> = commit_diff
                                    .iter()
                                    .filter(|fd| {
                                        let p = std::path::PathBuf::from(&fd.path);
                                        selected.contains(&p)
                                    })
                                    .map(|fd| {
                                        (std::path::PathBuf::from(&fd.path), fd.hunks.clone())
                                    })
                                    .collect();
                                repo.file_view = crate::state::FileViewState::MultipleDiffs {
                                    files: multi,
                                    view_mode: crate::state::DiffViewMode::Unified,
                                };
                            }
                            return Effect::None;
                        } else {
                            state.loading = true;
                            let selected_paths: Vec<_> =
                                files.selected_files.iter().cloned().collect();
                            return Effect::LoadMultipleFileDiffs {
                                repo_path: repo.path.clone(),
                                file_paths: selected_paths,
                            };
                        }
                    }
                }
            }
            Effect::None
        }

        AppMessage::StageSelectedFilesRequested => {
            if let Some(repo) = &state.current_repo {
                if let Some(files) = &repo.changed_files {
                    let files_to_stage: Vec<_> = files
                        .selected_files
                        .iter()
                        .filter(|path| {
                            files.unstaged.iter().any(|f| &f.path == *path)
                                || files.untracked.iter().any(|f| &f.path == *path)
                        })
                        .cloned()
                        .collect();

                    if !files_to_stage.is_empty() {
                        state.loading = true;
                        let effects: Vec<_> = files_to_stage
                            .into_iter()
                            .map(|path| Effect::StageFile {
                                repo_path: repo.path.clone(),
                                file_path: path,
                            })
                            .collect();
                        return Effect::Batch(effects);
                    }
                }
            }
            Effect::None
        }

        AppMessage::UnstageSelectedFilesRequested => {
            if let Some(repo) = &state.current_repo {
                if let Some(files) = &repo.changed_files {
                    let files_to_unstage: Vec<_> = files
                        .selected_files
                        .iter()
                        .filter(|path| files.staged.iter().any(|f| &f.path == *path))
                        .cloned()
                        .collect();

                    if !files_to_unstage.is_empty() {
                        state.loading = true;
                        let effects: Vec<_> = files_to_unstage
                            .into_iter()
                            .map(|path| Effect::UnstageFile {
                                repo_path: repo.path.clone(),
                                file_path: path,
                            })
                            .collect();
                        return Effect::Batch(effects);
                    }
                }
            }
            Effect::None
        }

        AppMessage::FileContentRequested(path) => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::LoadFileContent {
                    repo_path: repo.path.clone(),
                    file_path: path,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::FileContentLoaded {
            path,
            content,
            language,
        } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.file_view = crate::state::FileViewState::Content {
                    path,
                    content,
                    language,
                };
            }
            Effect::None
        }

        AppMessage::FileDiffRequested(path) => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::LoadFileDiff {
                    repo_path: repo.path.clone(),
                    file_path: path,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::FileDiffLoaded { path, hunks } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.file_view = crate::state::FileViewState::Diff {
                    path,
                    hunks,
                    view_mode: crate::state::DiffViewMode::Unified,
                };
            }
            Effect::None
        }

        AppMessage::MultipleFileDiffsLoaded { files } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.file_view = crate::state::FileViewState::MultipleDiffs {
                    files,
                    view_mode: crate::state::DiffViewMode::Unified,
                };
            }
            Effect::None
        }

        AppMessage::BinaryFileDetected { path, size } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.file_view = crate::state::FileViewState::Binary { path, size };
            }
            Effect::None
        }

        AppMessage::DiffViewModeChanged(mode) => {
            if let Some(repo) = &mut state.current_repo {
                match &mut repo.file_view {
                    crate::state::FileViewState::Diff { view_mode, .. } => {
                        *view_mode = mode;
                    }
                    crate::state::FileViewState::MultipleDiffs { view_mode, .. } => {
                        *view_mode = mode;
                    }
                    _ => {}
                }
            }
            Effect::None
        }

        AppMessage::RevertFileRequested(path) => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::RevertFile {
                    repo_path: repo.path.clone(),
                    file_path: path,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::RevertFileCompleted => {
            state.loading = false;
            if let Some(repo) = &state.current_repo {
                Effect::LoadChangedFiles(repo.path.clone())
            } else {
                Effect::None
            }
        }

        AppMessage::OpenFileInEditorRequested(path) => {
            if let Some(repo) = &state.current_repo {
                Effect::OpenInEditor {
                    full_path: repo.path.join(path),
                }
            } else {
                Effect::None
            }
        }

        AppMessage::OpenFileFolderRequested(path) => {
            if let Some(repo) = &state.current_repo {
                let full = repo.path.join(&path);
                let folder = full.parent().map(|p| p.to_path_buf()).unwrap_or(full);
                Effect::OpenFolder { full_path: folder }
            } else {
                Effect::None
            }
        }

        _ => unreachable!("file_handlers received unexpected message"),
    }
}
