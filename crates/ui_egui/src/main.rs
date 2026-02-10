//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

mod components;
mod panes;
mod utils;
mod widgets;

use crabontree_app::{load_config, reduce, save_config, AppState, Effect, JobExecutor};
use crabontree_ui_core::Theme;
use eframe::egui;
use utils::{keyboard, scroll_config, theme};

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
    active_panel: keyboard::ActivePanel,
    show_shortcuts_help: bool,
    active_pane: usize,
}

impl CrabOnTreeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let theme = Theme::by_name(&config.theme).unwrap_or_else(Theme::dark);

        let (executor, message_rx) = JobExecutor::new();

        let pane_widths = config.pane_widths;

        let state = AppState {
            current_repo: None,
            loading: false,
            error: None,
            config,
            staging_progress: None,
            layout_config: crabontree_app::LayoutConfig {
                pane_widths,
            },
        };

        Self {
            state,
            executor,
            message_rx,
            theme,
            active_panel: keyboard::ActivePanel::BranchTree,
            show_shortcuts_help: false,
            active_pane: 0,
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
            Effect::LoadWorkingDirStatus(path) => {
                self.executor.submit(crabontree_app::Job::LoadWorkingDirStatus(path));
            }
            Effect::StageFile { repo_path, file_path } => {
                self.executor.submit(crabontree_app::Job::StageFile { repo_path, file_path });
            }
            Effect::UnstageFile { repo_path, file_path } => {
                self.executor.submit(crabontree_app::Job::UnstageFile { repo_path, file_path });
            }
            Effect::StageAll(path) => {
                self.executor.submit(crabontree_app::Job::StageAll(path));
            }
            Effect::UnstageAll(path) => {
                self.executor.submit(crabontree_app::Job::UnstageAll(path));
            }
            Effect::CreateCommit { repo_path, message } => {
                self.executor.submit(crabontree_app::Job::CreateCommit { repo_path, message });
            }
            Effect::LoadAuthorIdentity(path) => {
                self.executor.submit(crabontree_app::Job::LoadAuthorIdentity(path));
            }
            Effect::LoadBranchTree(path) => {
                self.executor.submit(crabontree_app::Job::LoadBranchTree(path));
            }
            Effect::CheckoutBranch { repo_path, branch_name } => {
                self.executor.submit(crabontree_app::Job::CheckoutBranch { repo_path, branch_name });
            }
            Effect::LoadFileTree(_path) => {
                // File tree pane removed - ignore
            }
            Effect::LoadChangedFiles(path) => {
                self.executor.submit(crabontree_app::Job::LoadChangedFiles(path));
            }
            Effect::LoadFileContent { repo_path, file_path } => {
                self.executor.submit(crabontree_app::Job::LoadFileContent { repo_path, file_path });
            }
            Effect::LoadFileDiff { repo_path, file_path } => {
                self.executor.submit(crabontree_app::Job::LoadFileDiff { repo_path, file_path });
            }
        }
    }


    fn render_repository_view(&mut self, ui: &mut egui::Ui) {
        self.render_four_pane_layout(ui);
    }

    fn render_four_pane_layout(&mut self, ui: &mut egui::Ui) {
        // Handle keyboard shortcuts
        let (action, new_pane, new_panel) =
            keyboard::handle_shortcuts(ui, self.active_pane, self.active_panel);

        self.active_pane = new_pane;
        self.active_panel = new_panel;

        match action {
            keyboard::KeyboardAction::ToggleHelp => {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            }
            _ => {
                if let Some(msg) = keyboard::action_to_message(action) {
                    self.handle_message(msg);
                }
            }
        }

        // Load 3-pane data on first render (removed file tree)
        let (need_branch_tree, need_changed_files) = if let Some(repo) = &self.state.current_repo {
            (
                repo.branch_tree.is_none(),
                repo.changed_files.is_none(),
            )
        } else {
            (false, false)
        };

        if need_branch_tree {
            self.handle_message(crabontree_app::AppMessage::LoadBranchTreeRequested);
        }
        if need_changed_files {
            self.handle_message(crabontree_app::AppMessage::LoadChangedFilesRequested);
        }

        // Clone necessary data
        let (changed_files, file_view) = if let Some(repo) = &self.state.current_repo {
            (
                repo.changed_files.clone(),
                repo.file_view.clone(),
            )
        } else {
            return;
        };

        // Left Panel: Commit History
        egui::SidePanel::left("commit_history_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(200.0..=600.0)
            .show_inside(ui, |ui| {
                // Fixed height header - exactly 40px total including separator
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Commit History");
                });
                ui.add_space(10.0);
                ui.separator();
                scroll_config::vertical_scroll()
                    .id_source("commit_history_scroll")
                    .show(ui, |ui| {
                        scroll_config::set_full_width(ui);

                        // Get commit history data
                        let (commits, selected_commit, has_working_dir_changes) =
                            if let Some(repo) = &self.state.current_repo {
                                (
                                    repo.commits.as_slice(),
                                    repo.selected_commit.as_ref(),
                                    !repo.working_dir_files.is_empty(),
                                )
                            } else {
                                (&[][..], None, false)
                            };

                        // Render and handle action
                        let action = panes::commit_history::render(
                            ui,
                            commits,
                            selected_commit,
                            has_working_dir_changes,
                        );

                        if let Some(msg) = panes::commit_history::action_to_message(action) {
                            self.handle_message(msg);
                        }
                    });
            });

        // Right Panel: Diff Viewer
        egui::SidePanel::right("diff_viewer_panel")
            .resizable(true)
            .default_width(500.0)
            .width_range(300.0..=800.0)
            .show_inside(ui, |ui| {
                // Fixed height header - exactly 40px total including separator
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Diff Viewer");
                });
                ui.add_space(10.0);
                ui.separator();
                scroll_config::vertical_scroll()
                    .id_source("diff_viewer_scroll")
                    .show(ui, |ui| {
                        scroll_config::set_full_width(ui);
                        panes::diff_viewer::render(ui, &file_view);
                    });
            });

        // Central Panel: Changed Files
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                // Fixed height header - exactly 40px total including separator
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Changed Files");
                });
                ui.add_space(10.0);
                ui.separator();
                scroll_config::vertical_scroll()
                    .id_source("changed_files_scroll")
                    .show(ui, |ui| {
                        scroll_config::set_full_width(ui);
                        if let Some(files) = &changed_files {
                            let action = panes::changed_files::render(ui, files);
                            if let Some(msg) = panes::changed_files::action_to_message(action) {
                                self.handle_message(msg);
                            }
                        } else {
                            ui.label("Loading changed files...");
                        }
                    });
            });
    }

}

impl eframe::App for CrabOnTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        theme::apply_theme(ctx, &self.theme);

        // Request continuous repaints for message polling
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Render top panel
        let top_action = components::top_panel::render(
            ctx,
            self.state.current_repo.is_some(),
            self.state.loading,
        );
        if let Some(msg) = components::top_panel::action_to_message(top_action) {
            self.handle_message(msg);
        }

        // Render error panel
        let error_action = components::error_panel::render(
            ctx,
            self.state.error.as_ref(),
            self.theme.error,
        );
        if let Some(msg) = components::error_panel::action_to_message(error_action) {
            self.handle_message(msg);
        }

        // Render help dialog if requested
        if self.show_shortcuts_help {
            components::shortcuts_help::render(ctx, &mut self.show_shortcuts_help);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                if self.state.current_repo.is_some() {
                    self.render_repository_view(ui);
                } else {
                    let welcome_action = components::welcome_view::render(
                        ui,
                        &self.state.config.recent_repos,
                    );
                    if let Some(msg) = components::welcome_view::action_to_message(welcome_action) {
                        self.handle_message(msg);
                    }
                }
            });
    }
}
