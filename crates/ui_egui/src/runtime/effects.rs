use crate::runtime::CrabOnTreeApp;
use crabontree_app::{save_config, Effect};

impl CrabOnTreeApp {
    pub(crate) fn execute_effect(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::OpenRepo(path) => {
                self.executor.submit(crabontree_app::Job::OpenRepo(path));
            }
            Effect::RefreshRepo(path) => {
                self.executor.submit(crabontree_app::Job::RefreshRepo(path));
            }
            Effect::SaveConfig => {
                if let Err(e) = save_config(&self.state.config) {
                    tracing::warn!("Failed to save config: {}", e);
                }
            }
            Effect::Batch(effects) => {
                for effect in effects {
                    self.execute_effect(effect);
                }
            }
            Effect::LoadCommitHistory(path) => {
                self.executor
                    .submit(crabontree_app::Job::LoadCommitHistory(path));
            }
            Effect::LoadCommitDiff {
                repo_path,
                commit_hash,
            } => {
                self.executor.submit(crabontree_app::Job::LoadCommitDiff {
                    repo_path,
                    commit_hash,
                });
            }
            Effect::LoadWorkingDirStatus(path) => {
                self.executor
                    .submit(crabontree_app::Job::LoadWorkingDirStatus(path));
            }
            Effect::StageFile {
                repo_path,
                file_path,
            } => {
                self.executor.submit(crabontree_app::Job::StageFile {
                    repo_path,
                    file_path,
                });
            }
            Effect::UnstageFile {
                repo_path,
                file_path,
            } => {
                self.executor.submit(crabontree_app::Job::UnstageFile {
                    repo_path,
                    file_path,
                });
            }
            Effect::StageAll(path) => {
                self.executor.submit(crabontree_app::Job::StageAll(path));
            }
            Effect::UnstageAll(path) => {
                self.executor.submit(crabontree_app::Job::UnstageAll(path));
            }
            Effect::CreateCommit {
                repo_path,
                message,
                amend,
                push,
            } => {
                self.executor.submit(crabontree_app::Job::CreateCommit {
                    repo_path,
                    message,
                    amend,
                    push,
                });
            }
            Effect::LoadAuthorIdentity(path) => {
                self.executor
                    .submit(crabontree_app::Job::LoadAuthorIdentity(path));
            }
            Effect::LoadBranchTree(path) => {
                self.executor
                    .submit(crabontree_app::Job::LoadBranchTree(path));
            }
            Effect::CheckoutBranch {
                repo_path,
                branch_name,
            } => {
                self.executor.submit(crabontree_app::Job::CheckoutBranch {
                    repo_path,
                    branch_name,
                });
            }
            Effect::LoadChangedFiles(path) => {
                self.executor
                    .submit(crabontree_app::Job::LoadChangedFiles(path));
            }
            Effect::LoadFileContent {
                repo_path,
                file_path,
            } => {
                self.executor.submit(crabontree_app::Job::LoadFileContent {
                    repo_path,
                    file_path,
                });
            }
            Effect::LoadFileDiff {
                repo_path,
                file_path,
            } => {
                self.executor.submit(crabontree_app::Job::LoadFileDiff {
                    repo_path,
                    file_path,
                });
            }
            Effect::LoadMultipleFileDiffs {
                repo_path,
                file_paths,
            } => {
                self.executor
                    .submit(crabontree_app::Job::LoadMultipleFileDiffs {
                        repo_path,
                        file_paths,
                    });
            }
            Effect::CheckUncommittedChanges {
                repo_path,
                branch_name,
                is_remote,
            } => {
                self.executor
                    .submit(crabontree_app::Job::CheckUncommittedChanges {
                        repo_path,
                        branch_name,
                        is_remote,
                    });
            }
            Effect::StashAndCheckout {
                repo_path,
                branch_name,
                is_remote,
                from_branch,
            } => {
                self.executor.submit(crabontree_app::Job::StashAndCheckout {
                    repo_path,
                    branch_name,
                    is_remote,
                    from_branch,
                });
            }
            Effect::DiscardAndCheckout {
                repo_path,
                branch_name,
                is_remote,
            } => {
                self.executor
                    .submit(crabontree_app::Job::DiscardAndCheckout {
                        repo_path,
                        branch_name,
                        is_remote,
                    });
            }
            Effect::CheckLocalBranchExists {
                repo_path,
                remote_branch,
                local_name,
            } => {
                self.executor
                    .submit(crabontree_app::Job::CheckLocalBranchExists {
                        repo_path,
                        remote_branch,
                        local_name,
                    });
            }
            Effect::CheckoutRemoteBranch {
                repo_path,
                remote_branch,
                local_name,
                override_existing,
            } => {
                self.executor
                    .submit(crabontree_app::Job::CheckoutRemoteBranch {
                        repo_path,
                        remote_branch,
                        local_name,
                        override_existing,
                    });
            }
        }
    }
}
