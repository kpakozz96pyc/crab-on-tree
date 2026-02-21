use crate::{AppMessage, AppState, Effect};

pub(super) fn handle(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
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
                Effect::CheckUncommittedChanges {
                    repo_path: repo.path.clone(),
                    branch_name: name,
                    is_remote,
                }
            } else {
                Effect::None
            }
        }

        AppMessage::ShowCheckoutWithChangesDialog {
            branch_name,
            is_remote,
        } => {
            state.loading = false;
            state.checkout_changes_dialog = Some(crate::state::CheckoutChangesDialog {
                branch_name,
                is_remote,
            });
            Effect::None
        }

        AppMessage::CheckoutWithStash {
            branch_name,
            is_remote,
        } => {
            state.checkout_changes_dialog = None;
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

        AppMessage::CheckoutWithDiscard {
            branch_name,
            is_remote,
        } => {
            state.checkout_changes_dialog = None;
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

        AppMessage::ShowRemoteBranchConflictDialog {
            remote_branch,
            local_name,
        } => {
            state.branch_conflict_dialog = Some(crate::state::BranchConflictDialog {
                remote_branch: remote_branch.clone(),
                local_name: local_name.clone(),
                new_name_input: local_name,
            });
            Effect::None
        }

        AppMessage::CheckoutRemoteOverride {
            remote_branch,
            local_name,
        } => {
            state.branch_conflict_dialog = None;
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

        AppMessage::CheckoutRemoteRename {
            remote_branch,
            new_local_name,
        } => {
            state.branch_conflict_dialog = None;
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

        _ => unreachable!("branch_handlers received unexpected message"),
    }
}
