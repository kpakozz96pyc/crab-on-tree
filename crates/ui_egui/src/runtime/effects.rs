use crate::runtime::CrabOnTreeApp;
use crabontree_app::{save_config, Effect};

impl CrabOnTreeApp {
    fn submit_job(&mut self, job: crabontree_app::Job) {
        self.pending_jobs = self.pending_jobs.saturating_add(1);
        self.state.loading = true;
        self.executor.submit(job);
    }

    pub(crate) fn execute_effect(&mut self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::OpenRepo(path) => {
                self.submit_job(crabontree_app::Job::OpenRepo(path));
            }
            Effect::RefreshRepo(path) => {
                self.submit_job(crabontree_app::Job::RefreshRepo(path));
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
                self.submit_job(crabontree_app::Job::LoadCommitHistory(path));
            }
            Effect::LoadCommitDiff {
                repo_path,
                commit_hash,
            } => {
                self.submit_job(crabontree_app::Job::LoadCommitDiff {
                    repo_path,
                    commit_hash,
                });
            }
            Effect::LoadWorkingDirStatus(path) => {
                self.submit_job(crabontree_app::Job::LoadWorkingDirStatus(path));
            }
            Effect::StageFile {
                repo_path,
                file_path,
            } => {
                self.submit_job(crabontree_app::Job::StageFile {
                    repo_path,
                    file_path,
                });
            }
            Effect::UnstageFile {
                repo_path,
                file_path,
            } => {
                self.submit_job(crabontree_app::Job::UnstageFile {
                    repo_path,
                    file_path,
                });
            }
            Effect::StageAll(path) => {
                self.submit_job(crabontree_app::Job::StageAll(path));
            }
            Effect::UnstageAll(path) => {
                self.submit_job(crabontree_app::Job::UnstageAll(path));
            }
            Effect::CreateCommit {
                repo_path,
                message,
                amend,
                push,
            } => {
                self.submit_job(crabontree_app::Job::CreateCommit {
                    repo_path,
                    message,
                    amend,
                    push,
                });
            }
            Effect::LoadAuthorIdentity(path) => {
                self.submit_job(crabontree_app::Job::LoadAuthorIdentity(path));
            }
            Effect::LoadBranchTree(path) => {
                self.submit_job(crabontree_app::Job::LoadBranchTree(path));
            }
            Effect::CheckoutBranch {
                repo_path,
                branch_name,
            } => {
                self.submit_job(crabontree_app::Job::CheckoutBranch {
                    repo_path,
                    branch_name,
                });
            }
            Effect::LoadChangedFiles(path) => {
                self.submit_job(crabontree_app::Job::LoadChangedFiles(path));
            }
            Effect::LoadFileContent {
                repo_path,
                file_path,
            } => {
                self.submit_job(crabontree_app::Job::LoadFileContent {
                    repo_path,
                    file_path,
                });
            }
            Effect::LoadFileDiff {
                repo_path,
                file_path,
            } => {
                self.submit_job(crabontree_app::Job::LoadFileDiff {
                    repo_path,
                    file_path,
                });
            }
            Effect::LoadMultipleFileDiffs {
                repo_path,
                file_paths,
            } => {
                self.submit_job(crabontree_app::Job::LoadMultipleFileDiffs {
                    repo_path,
                    file_paths,
                });
            }
            Effect::CheckUncommittedChanges {
                repo_path,
                branch_name,
                is_remote,
            } => {
                self.submit_job(crabontree_app::Job::CheckUncommittedChanges {
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
                self.submit_job(crabontree_app::Job::StashAndCheckout {
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
                self.submit_job(crabontree_app::Job::DiscardAndCheckout {
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
                self.submit_job(crabontree_app::Job::CheckLocalBranchExists {
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
                self.submit_job(crabontree_app::Job::CheckoutRemoteBranch {
                    repo_path,
                    remote_branch,
                    local_name,
                    override_existing,
                });
            }
            Effect::RevertFile {
                repo_path,
                file_path,
            } => {
                self.submit_job(crabontree_app::Job::RevertFile {
                    repo_path,
                    file_path,
                });
            }
            Effect::OpenInEditor { full_path } => {
                if let Err(e) = std::process::Command::new("xdg-open")
                    .arg(&full_path)
                    .spawn()
                {
                    tracing::warn!("Failed to open file in editor: {}", e);
                }
            }
            Effect::OpenFolder { full_path } => {
                if let Err(e) = std::process::Command::new("xdg-open")
                    .arg(&full_path)
                    .spawn()
                {
                    tracing::warn!("Failed to open folder: {}", e);
                }
            }
        }
    }
}
