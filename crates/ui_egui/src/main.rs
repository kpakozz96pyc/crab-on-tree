//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

mod panes;
mod utils;
mod widgets;

use crabontree_app::{
    load_config, reduce, save_config, AppState, Effect, JobExecutor,
};
use crabontree_ui_core::{Color, Theme};
use eframe::egui;
use utils::scroll_config;

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

/// Active panel for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivePanel {
    BranchTree,
    ChangedFiles,
    FileViewer,
}

struct CrabOnTreeApp {
    state: AppState,
    executor: JobExecutor,
    message_rx: tokio::sync::mpsc::Receiver<crabontree_app::AppMessage>,
    theme: Theme,
    active_panel: ActivePanel,
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
            active_panel: ActivePanel::BranchTree,
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
                ui.heading("CrabOnTree");

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

    fn render_shortcuts_help(&mut self, ctx: &egui::Context) {
        egui::Window::new("Keyboard Shortcuts")
            .open(&mut self.show_shortcuts_help)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Navigation");
                ui.separator();
                ui.label("↑ / k        Previous item");
                ui.label("↓ / j        Next item");
                ui.label("Enter       Select/View commit");
                ui.label("Space       Toggle staging (files)");
                ui.label("Home        First item");
                ui.label("End         Last item");
                ui.label("g g         Go to top (vim-style)");
                ui.label("Shift+G     Go to bottom (vim-style)");

                ui.add_space(10.0);
                ui.heading("Search");
                ui.separator();
                ui.label("/           Focus file search");
                ui.label("Ctrl+F      Focus commit search");
                ui.label("Esc         Clear search / Blur");

                ui.add_space(10.0);
                ui.heading("Panels");
                ui.separator();
                ui.label("1           Working directory");
                ui.label("2           Commit message");
                ui.label("3           Commit history");
                ui.label("Tab         Cycle panels");

                ui.add_space(10.0);
                ui.heading("Actions");
                ui.separator();
                ui.label("c           Focus commit message");
                ui.label("Ctrl+Enter  Create commit");
                ui.label("a           Stage all files");
                ui.label("u           Unstage all files");
                ui.label("r           Refresh repository");

                ui.add_space(10.0);
                ui.heading("Help");
                ui.separator();
                ui.label("?           Show/hide this help");
            });
    }

    fn render_repository_view(&mut self, ui: &mut egui::Ui) {
        self.render_four_pane_layout(ui);
    }

    fn render_four_pane_layout(&mut self, ui: &mut egui::Ui) {
        // Keyboard shortcuts for 4-pane layout
        let any_text_focused = ui.memory(|mem| mem.focused().is_some());

        if !any_text_focused {
            // Pane selection: 1, 2, 3 (removed file tree pane)
            if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.active_pane = 0;
                self.active_panel = ActivePanel::BranchTree;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.active_pane = 1;
                self.active_panel = ActivePanel::ChangedFiles;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.active_pane = 2;
                self.active_panel = ActivePanel::FileViewer;
            }

            // Tab to cycle through panes
            if ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
                self.active_pane = (self.active_pane + 1) % 3;
                self.active_panel = match self.active_pane {
                    0 => ActivePanel::BranchTree,
                    1 => ActivePanel::ChangedFiles,
                    _ => ActivePanel::FileViewer,
                };
            }

            // Shift+Tab to cycle backward
            if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift) {
                self.active_pane = if self.active_pane == 0 { 2 } else { self.active_pane - 1 };
                self.active_panel = match self.active_pane {
                    0 => ActivePanel::BranchTree,
                    1 => ActivePanel::ChangedFiles,
                    _ => ActivePanel::FileViewer,
                };
            }

            // 'r' key to refresh
            if ui.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.ctrl) {
                self.handle_message(crabontree_app::AppMessage::RefreshRepo);
            }

            // '?' to show help
            if ui.input(|i| i.key_pressed(egui::Key::Questionmark) || (i.key_pressed(egui::Key::Slash) && i.modifiers.shift)) {
                self.show_shortcuts_help = !self.show_shortcuts_help;
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
        self.apply_theme(ctx);

        // Request continuous repaints for message polling
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        self.render_top_panel(ctx);
        self.render_error_panel(ctx);

        // Render help dialog if requested
        if self.show_shortcuts_help {
            self.render_shortcuts_help(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                if self.state.current_repo.is_some() {
                    self.render_repository_view(ui);
                } else {
                    self.render_welcome_view(ui);
                }
            });
    }
}
