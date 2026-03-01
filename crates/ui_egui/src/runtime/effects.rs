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
            effect => {
                if let Ok(job) = effect.try_into_job() {
                    self.submit_job(job);
                }
            }
        }
    }
}
