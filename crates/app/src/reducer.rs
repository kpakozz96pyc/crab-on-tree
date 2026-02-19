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
            state.current_repo = Some(RepoState::new(path.clone(), head, branches, status));

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
            state.committing = false;
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

                // Check if selecting working directory or a real commit
                if hash == crate::WORKING_DIR_HASH {
                    // For working directory, update changed files to show working dir files
                    Effect::LoadChangedFiles(repo.path.clone())
                } else {
                    // For regular commit, load commit diff
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
                repo.changed_files = None; // Clear changed files view
            }
            Effect::None
        }

        AppMessage::CommitDiffLoaded { commit_hash, diff } => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                // Only update if this is still the selected commit
                if repo.selected_commit.as_ref() == Some(&commit_hash) {
                    repo.commit_diff = Some(diff.clone());
                    tracing::info!("Loaded diff for commit {}", commit_hash);

                    // Convert commit diff to changed files format
                    use std::path::PathBuf;
                    use crate::state::ChangedFilesState;
                    use crate::WorkingDirFile;
                    use crate::WorkingDirStatus;

                    // Get the commit message from the selected commit
                    let commit_message = repo.commits.iter()
                        .find(|c| c.hash == commit_hash)
                        .map(|c| c.message.clone())
                        .unwrap_or_default();

                    let mut changed_files = ChangedFilesState {
                        staged: Vec::new(),
                        unstaged: Vec::new(),
                        untracked: Vec::new(),
                        conflicted: Vec::new(),
                        selected_file: None,
                        selected_files: std::collections::HashSet::new(),
                        last_clicked_file: None,
                        commit_message,
                        is_commit_view: true, // This is a commit view, not working directory
                        commit_summary: String::new(),
                        commit_description: String::new(),
                        amend_last_commit: false,
                        push_after_commit: false,
                    };

                    // Convert FileDiff entries to WorkingDirFile entries
                    // All files in a commit are "staged" (committed)
                    for file_diff in &diff {
                        let status = match file_diff.status {
                            crate::FileStatus::Modified => WorkingDirStatus::Modified,
                            crate::FileStatus::Added => WorkingDirStatus::Untracked, // Show as added
                            crate::FileStatus::Deleted => WorkingDirStatus::Deleted,
                            crate::FileStatus::Renamed => WorkingDirStatus::Renamed,
                            crate::FileStatus::Copied => WorkingDirStatus::Modified,
                        };

                        let working_dir_file = WorkingDirFile {
                            path: PathBuf::from(&file_diff.path),
                            status,
                            is_staged: true, // All files in commit are "committed"
                        };

                        changed_files.staged.push(working_dir_file);
                    }

                    repo.changed_files = Some(changed_files);
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
                // Refresh working directory status, changed files, and repository status after staging
                Effect::Batch(vec![
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                    Effect::LoadChangedFiles(repo.path.clone()),
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
                        amend: false,
                        push: false,
                    }
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CommitCreated { hash, message: _ } => {
            state.loading = false;
            state.committing = false;
            if let Some(repo) = &mut state.current_repo {
                // Clear commit message
                repo.commit_message.clear();

                // Clear commit panel fields
                if let Some(changed_files) = &mut repo.changed_files {
                    changed_files.commit_summary.clear();
                    changed_files.commit_description.clear();
                    changed_files.amend_last_commit = false;
                }

                // Show success message temporarily
                tracing::info!("Commit created: {}", hash);

                // Refresh repo data, working directory, and changed files
                Effect::Batch(vec![
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

        // ===== 4-Pane Layout Handlers =====

        AppMessage::LoadBranchTreeRequested => {
            state.loading = true;
            if let Some(repo) = &state.current_repo {
                Effect::LoadBranchTree(repo.path.clone())
            } else {
                tracing::warn!("Cannot load branch tree: no repository open");
                Effect::None
            }
        }

        AppMessage::BranchTreeLoaded(branch_tree) => {
            state.loading = false;
            if let Some(repo) = &mut state.current_repo {
                repo.branch_tree = Some(branch_tree);
                tracing::info!("Loaded branch tree");
            }
            Effect::None
        }

        AppMessage::BranchSectionToggled(section) => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(tree) = &mut repo.branch_tree {
                    if tree.expanded_sections.contains(&section) {
                        tree.expanded_sections.remove(&section);
                    } else {
                        tree.expanded_sections.insert(section);
                    }
                }
            }
            Effect::None
        }

        AppMessage::BranchSelected { name, is_remote: _ } => {
            if let Some(repo) = &mut state.current_repo {
                if let Some(tree) = &mut repo.branch_tree {
                    tree.selected_branch = Some(name);
                }
            }
            Effect::None
        }

        AppMessage::BranchCheckoutRequested { name, is_remote } => {
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                // Check for uncommitted changes first
                Effect::CheckUncommittedChanges {
                    repo_path: repo.path.clone(),
                    branch_name: name,
                    is_remote,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::ShowCheckoutWithChangesDialog { branch_name, is_remote } => {
            state.loading = false; // Stop loading to show dialog
            state.checkout_changes_dialog = Some(crate::state::CheckoutChangesDialog {
                branch_name,
                is_remote,
            });
            Effect::None
        }

        AppMessage::CheckoutWithStash { branch_name, is_remote } => {
            state.checkout_changes_dialog = None; // Close dialog
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::StashAndCheckout {
                    repo_path: repo.path.clone(),
                    branch_name,
                    is_remote,
                    from_branch: repo.head.clone(),
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CheckoutWithDiscard { branch_name, is_remote } => {
            state.checkout_changes_dialog = None; // Close dialog
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::DiscardAndCheckout {
                    repo_path: repo.path.clone(),
                    branch_name,
                    is_remote,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::ShowRemoteBranchConflictDialog { remote_branch, local_name } => {
            state.branch_conflict_dialog = Some(crate::state::BranchConflictDialog {
                remote_branch: remote_branch.clone(),
                local_name: local_name.clone(),
                new_name_input: local_name, // Default to the suggested name
            });
            Effect::None
        }

        AppMessage::CheckoutRemoteOverride { remote_branch, local_name } => {
            state.branch_conflict_dialog = None; // Close dialog
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::CheckoutRemoteBranch {
                    repo_path: repo.path.clone(),
                    remote_branch,
                    local_name,
                    override_existing: true,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::CheckoutRemoteRename { remote_branch, new_local_name } => {
            state.branch_conflict_dialog = None; // Close dialog
            if let Some(repo) = &state.current_repo {
                state.loading = true;
                Effect::CheckoutRemoteBranch {
                    repo_path: repo.path.clone(),
                    remote_branch,
                    local_name: new_local_name,
                    override_existing: false,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::ChangesStashed { stash_name } => {
            tracing::info!("Changes stashed: {}", stash_name);
            // Refresh repository state after stashing
            if let Some(repo) = &state.current_repo {
                Effect::Batch(vec![
                    Effect::LoadBranchTree(repo.path.clone()),
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

        AppMessage::ChangesDiscarded => {
            tracing::info!("Changes discarded");
            // Refresh repository state after discarding
            if let Some(repo) = &state.current_repo {
                Effect::Batch(vec![
                    Effect::LoadBranchTree(repo.path.clone()),
                    Effect::LoadWorkingDirStatus(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

        AppMessage::BranchCheckedOut(branch_name) => {
            state.loading = false;
            tracing::info!("Checked out branch: {}", branch_name);

            if let Some(repo) = &state.current_repo {
                // Refresh all data after checkout
                Effect::Batch(vec![
                    Effect::RefreshRepo(repo.path.clone()),
                    Effect::LoadBranchTree(repo.path.clone()),
                    Effect::LoadChangedFiles(repo.path.clone()),
                    Effect::LoadCommitHistory(repo.path.clone()),
                ])
            } else {
                Effect::None
            }
        }

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
                // Preserve commit panel fields from previous state
                if let Some(old_files) = &repo.changed_files {
                    changed_files.commit_summary = old_files.commit_summary.clone();
                    changed_files.commit_description = old_files.commit_description.clone();
                    changed_files.amend_last_commit = old_files.amend_last_commit;
                    changed_files.push_after_commit = old_files.push_after_commit;
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
                    // Clear multi-selection and select only this file
                    files.selected_files.clear();
                    files.selected_files.insert(path.clone());
                    files.last_clicked_file = Some(path.clone());
                }

                // Check if we're viewing a commit or working directory
                let is_viewing_commit = repo.selected_commit.as_ref()
                    .map(|hash| hash != crate::WORKING_DIR_HASH)
                    .unwrap_or(false);

                if is_viewing_commit {
                    // For commits, use the already-loaded commit diff
                    if let Some(commit_diff) = &repo.commit_diff {
                        // Find the file in the commit diff
                        let path_str = path.to_string_lossy();
                        if let Some(file_diff) = commit_diff.iter()
                            .find(|fd| fd.path == path_str)
                        {
                            // Set the file view to show the diff
                            repo.file_view = crate::state::FileViewState::Diff {
                                path: path.clone(),
                                hunks: file_diff.hunks.clone(),
                                view_mode: crate::state::DiffViewMode::Unified,
                            };
                            Effect::None
                        } else {
                            // File not found in commit diff
                            Effect::None
                        }
                    } else {
                        // No commit diff loaded yet
                        Effect::None
                    }
                } else {
                    // For working directory, check if file is untracked (new file)
                    let is_untracked = repo.changed_files.as_ref()
                        .map(|files| files.untracked.iter().any(|f| f.path == path))
                        .unwrap_or(false);

                    state.loading = true;
                    if is_untracked {
                        // For new/untracked files, show content instead of diff
                        Effect::LoadFileContent {
                            repo_path: repo.path.clone(),
                            file_path: path,
                        }
                    } else {
                        // For modified/deleted files, show diff
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
                        // Ctrl+Click: Toggle file in selection
                        if files.selected_files.contains(&path) {
                            files.selected_files.remove(&path);
                        } else {
                            files.selected_files.insert(path.clone());
                        }
                        files.last_clicked_file = Some(path.clone());
                        files.selected_file = Some(path);
                    } else if shift {
                        // Shift+Click: Range selection
                        if let Some(last_clicked) = &files.last_clicked_file {
                            // Get all files in order
                            let mut all_files = Vec::new();
                            all_files.extend(files.staged.iter().map(|f| &f.path));
                            all_files.extend(files.unstaged.iter().map(|f| &f.path));
                            all_files.extend(files.untracked.iter().map(|f| &f.path));
                            all_files.extend(files.conflicted.iter().map(|f| &f.path));

                            // Find indices of last clicked and current
                            let last_idx = all_files.iter().position(|p| *p == last_clicked);
                            let current_idx = all_files.iter().position(|p| *p == &path);

                            if let (Some(start), Some(end)) = (last_idx, current_idx) {
                                let (start, end) = if start < end { (start, end) } else { (end, start) };
                                // Select all files in range
                                for i in start..=end {
                                    files.selected_files.insert(all_files[i].clone());
                                }
                            }
                        }
                        files.selected_file = Some(path);
                    }

                    // Load diffs for all selected files
                    if files.selected_files.len() > 1 {
                        state.loading = true;
                        let selected_paths: Vec<_> = files.selected_files.iter().cloned().collect();
                        return Effect::LoadMultipleFileDiffs {
                            repo_path: repo.path.clone(),
                            file_paths: selected_paths,
                        };
                    }
                }
            }
            Effect::None
        }

        AppMessage::StageSelectedFilesRequested => {
            if let Some(repo) = &state.current_repo {
                if let Some(files) = &repo.changed_files {
                    // Collect files to stage (from unstaged and untracked)
                    let files_to_stage: Vec<_> = files.selected_files.iter()
                        .filter(|path| {
                            files.unstaged.iter().any(|f| &f.path == *path)
                                || files.untracked.iter().any(|f| &f.path == *path)
                        })
                        .cloned()
                        .collect();

                    if !files_to_stage.is_empty() {
                        state.loading = true;
                        let effects: Vec<_> = files_to_stage.into_iter()
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
                    // Collect files to unstage (from staged)
                    let files_to_unstage: Vec<_> = files.selected_files.iter()
                        .filter(|path| files.staged.iter().any(|f| &f.path == *path))
                        .cloned()
                        .collect();

                    if !files_to_unstage.is_empty() {
                        state.loading = true;
                        let effects: Vec<_> = files_to_unstage.into_iter()
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

        AppMessage::FileContentLoaded { path, content, language } => {
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
                repo.file_view = crate::state::FileViewState::Binary {
                    path,
                    size,
                };
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

        AppMessage::CommitChangesRequested { summary, description, amend, push } => {
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

    }
}
