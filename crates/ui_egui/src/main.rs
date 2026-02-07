//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

use crabontree_app::{load_config, reduce, save_config, AppState, Commit, Effect, FileDiff, FileStatus, JobExecutor};
use crabontree_ui_core::{Color, Theme};
use eframe::egui;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("crabontree=debug".parse().unwrap())
                .add_directive("crabontree_app=debug".parse().unwrap())
                .add_directive("crabontree_git=debug".parse().unwrap())
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting CrabOnTree");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("CrabOnTree"),
        ..Default::default()
    };

    eframe::run_native(
        "CrabOnTree",
        options,
        Box::new(|cc| Box::new(CrabOnTreeApp::new(cc))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
}

struct CrabOnTreeApp {
    state: AppState,
    executor: JobExecutor,
    message_rx: tokio::sync::mpsc::Receiver<crabontree_app::AppMessage>,
    theme: Theme,
}

impl CrabOnTreeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let theme = Theme::by_name(&config.theme).unwrap_or_else(Theme::dark);

        let (executor, message_rx) = JobExecutor::new();

        let state = AppState {
            current_repo: None,
            loading: false,
            error: None,
            config,
        };

        Self {
            state,
            executor,
            message_rx,
            theme,
        }
    }

    fn poll_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, message: crabontree_app::AppMessage) {
        let effect = reduce(&mut self.state, message);
        self.execute_effect(effect);
    }

    fn execute_effect(&mut self, effect: Effect) {
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
                self.executor.submit(crabontree_app::Job::LoadCommitHistory(path));
            }
            Effect::LoadCommitDiff { repo_path, commit_hash } => {
                self.executor.submit(crabontree_app::Job::LoadCommitDiff { repo_path, commit_hash });
            }
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Convert Color to egui::Color32
        let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );

        visuals.panel_fill = to_egui(self.theme.bg_primary);
        visuals.extreme_bg_color = to_egui(self.theme.bg_secondary);
        visuals.faint_bg_color = to_egui(self.theme.bg_tertiary);

        visuals.override_text_color = Some(to_egui(self.theme.fg_primary));
        visuals.hyperlink_color = to_egui(self.theme.accent_primary);
        visuals.error_fg_color = to_egui(self.theme.error);
        visuals.warn_fg_color = to_egui(self.theme.warning);

        ctx.set_visuals(visuals);
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🦀 CrabOnTree");

                ui.add_space(20.0);

                if ui.button("📂 Open Repository").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.handle_message(crabontree_app::AppMessage::OpenRepoRequested(path));
                    }
                }

                if self.state.current_repo.is_some() {
                    if ui.button("🔄 Refresh").clicked() {
                        self.handle_message(crabontree_app::AppMessage::RefreshRepo);
                    }

                    if ui.button("✖ Close").clicked() {
                        self.handle_message(crabontree_app::AppMessage::CloseRepo);
                    }
                }

                // Loading spinner
                if self.state.loading {
                    ui.add_space(10.0);
                    ui.spinner();
                }
            });
        });
    }

    fn render_error_panel(&mut self, ctx: &egui::Context) {
        if let Some(error) = self.state.error.clone() {
            egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                        (c.a * 255.0) as u8,
                    );

                    ui.colored_label(to_egui(self.theme.error), format!("❌ {}", error));

                    if ui.button("✖").clicked() {
                        self.handle_message(crabontree_app::AppMessage::ClearError);
                    }
                });
            });
        }
    }

    fn render_welcome_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Welcome to CrabOnTree");
            ui.add_space(20.0);
            ui.label("A Git GUI written in Rust with gitoxide and egui");
            ui.add_space(40.0);

            if ui.button("📂 Open a Repository").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.handle_message(crabontree_app::AppMessage::OpenRepoRequested(path));
                }
            }

            ui.add_space(40.0);

            if !self.state.config.recent_repos.is_empty() {
                ui.heading("Recent Repositories");
                ui.add_space(10.0);

                for repo_path in self.state.config.recent_repos.clone() {
                    if ui.button(repo_path.display().to_string()).clicked() {
                        self.handle_message(crabontree_app::AppMessage::OpenRepoRequested(repo_path));
                    }
                }
            }
        });
    }

    fn format_timestamp(timestamp: i64) -> String {
        use chrono::{DateTime, Utc};
        DateTime::from_timestamp(timestamp, 0)
            .map(|d| d.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Invalid date".to_string())
    }

    fn render_commit_history(&mut self, ui: &mut egui::Ui, commits: &[Commit], selected: &Option<String>) {
        if commits.is_empty() {
            ui.label("No commits loaded. Click 'Load Commit History' to view.");
            return;
        }

        ui.heading("Commit History");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_source("commit_history_scroll")
            .show(ui, |ui| {
                for commit in commits {
                    let is_selected = selected.as_ref() == Some(&commit.hash);

                    // Use commit hash as unique ID for each selectable label
                    let response = ui.selectable_label(
                        is_selected,
                        format!(
                            "{} - {} - {}",
                            commit.hash_short,
                            commit.message_summary,
                            commit.author_name
                        )
                    ).on_hover_text(&commit.hash);

                    if response.clicked() {
                        if is_selected {
                            self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                        } else {
                            self.handle_message(crabontree_app::AppMessage::CommitSelected(commit.hash.clone()));
                        }
                    }
                }
            });
    }

    fn render_commit_diff(&self, ui: &mut egui::Ui, diff: &[FileDiff]) {
        ui.add_space(10.0);
        ui.heading("Changed Files");
        ui.separator();

        if diff.is_empty() {
            ui.label("No changes");
            return;
        }

        let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );

        egui::ScrollArea::vertical()
            .id_source("commit_diff_scroll")
            .show(ui, |ui| {
                for file in diff {
                    let (status_symbol, status_color) = match file.status {
                        FileStatus::Added => ("+", self.theme.git_added),
                        FileStatus::Modified => ("~", self.theme.git_modified),
                        FileStatus::Deleted => ("-", self.theme.git_deleted),
                        FileStatus::Renamed => ("→", self.theme.accent_primary),
                        FileStatus::Copied => ("©", self.theme.accent_primary),
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(to_egui(status_color), status_symbol);
                        ui.label(&file.path);
                    });
                }
            });
    }

    fn render_commit_details(&self, ui: &mut egui::Ui, commit: &Commit, diff: Option<&Vec<FileDiff>>) {
        egui::ScrollArea::vertical()
            .id_source("commit_details_scroll")
            .show(ui, |ui| {
                ui.heading("Commit Details");
                ui.separator();

            ui.horizontal(|ui| {
                ui.label("Hash:");
                ui.monospace(&commit.hash);
            });

            ui.horizontal(|ui| {
                ui.label("Author:");
                ui.label(format!("{} <{}>", commit.author_name, commit.author_email));
            });

            ui.horizontal(|ui| {
                ui.label("Date:");
                ui.label(Self::format_timestamp(commit.author_date));
            });

            if commit.committer_name != commit.author_name {
                ui.horizontal(|ui| {
                    ui.label("Committer:");
                    ui.label(format!("{} <{}>", commit.committer_name, commit.committer_email));
                });
            }

            ui.add_space(10.0);
            ui.heading("Message");
            ui.separator();
            ui.label(&commit.message);

            if !commit.parent_hashes.is_empty() {
                ui.add_space(10.0);
                ui.heading("Parents");
                ui.separator();
                for parent in &commit.parent_hashes {
                    ui.monospace(parent);
                }
            }

            if let Some(diff) = diff {
                self.render_commit_diff(ui, diff);
            }
        });
    }

    fn render_repository_view(&mut self, ui: &mut egui::Ui) {
        // Clone data we need at the start to avoid borrowing issues
        let repo_data = self.state.current_repo.as_ref().map(|repo| (
                repo.path.clone(),
                repo.head.clone(),
                repo.branches.clone(),
                repo.status_summary.clone(),
                repo.commits.clone(),
                repo.selected_commit.clone(),
                repo.commit_diff.clone(),
            ));

        if let Some((path, head, branches, status, commits, selected_commit, commit_diff)) = repo_data {
            let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
                (c.a * 255.0) as u8,
            );

            // Collapsible repository info section
            egui::CollapsingHeader::new("📁 Repository Info")
                .id_source("repo_info_header")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.monospace(path.display().to_string());
                    });

                    ui.horizontal(|ui| {
                        ui.label("Current Branch:");
                        ui.colored_label(to_egui(self.theme.git_branch), &head);
                    });

                    ui.add_space(10.0);
                    ui.heading("Branches");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_source("branches_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for branch in &branches {
                                let is_current = branch == &head;
                                let text = if is_current {
                                    format!("● {}", branch)
                                } else {
                                    format!("  {}", branch)
                                };

                                if is_current {
                                    ui.colored_label(to_egui(self.theme.git_branch), text);
                                } else {
                                    ui.label(text);
                                }
                            }
                        });

                    ui.add_space(10.0);
                    ui.label(format!(
                        "Status: {} modified, {} added, {} deleted, {} untracked",
                        status.modified,
                        status.added,
                        status.deleted,
                        status.untracked
                    ));
                });

            ui.add_space(10.0);

            if ui.button("📜 Load Commit History").clicked() {
                self.handle_message(crabontree_app::AppMessage::LoadCommitHistoryRequested);
            }

            ui.add_space(10.0);
            ui.separator();

            // Get available height for the commit section
            let available_height = ui.available_height();

            // Two-column layout for commits - fills remaining space
            ui.horizontal(|ui| {
                // Left panel: commit list (40% of width)
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() * 0.4);
                    ui.set_height(available_height);
                    self.render_commit_history(ui, &commits, &selected_commit);
                });

                ui.separator();

                // Right panel: commit details (remaining width)
                ui.vertical(|ui| {
                    ui.set_height(available_height);
                    if let Some(hash) = &selected_commit {
                        if let Some(commit) = commits.iter().find(|c| &c.hash == hash) {
                            self.render_commit_details(ui, commit, commit_diff.as_ref());
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.label("Select a commit to view details");
                        });
                    }
                });
            });
        }
    }
}

impl eframe::App for CrabOnTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        self.apply_theme(ctx);

        // Request continuous repaints for message polling
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        self.render_top_panel(ctx);
        self.render_error_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.current_repo.is_some() {
                self.render_repository_view(ui);
            } else {
                self.render_welcome_view(ui);
            }
        });
    }
}
