//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

mod components;
mod panes;
mod utils;
mod widgets;

use crabontree_app::{load_config, reduce, save_config, AppState, Effect, JobExecutor};
use crabontree_ui_core::Theme;
use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex};
use utils::{keyboard, theme};

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
    dock_state: DockState<panes::Pane>,
}

impl CrabOnTreeApp {
    /// Creates the default 3-pane dock layout
    fn create_default_dock_layout() -> DockState<panes::Pane> {
        let mut dock_state = DockState::new(vec![panes::Pane::CommitHistory]);

        // Split right to create: CommitHistory | DiffViewer (30% | 70%)
        let [commit_node, _diff_node] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.70, // DiffViewer takes 70% of the width
            vec![panes::Pane::DiffViewer],
        );

        // Split the left node to create: CommitHistory | ChangedFiles | DiffViewer (30% | 40% | 30%)
        let _split = dock_state.main_surface_mut().split_right(
            commit_node,
            0.57, // ChangedFiles takes ~57% of the left 70% (0.70 * 0.57 ≈ 0.40 total)
            vec![panes::Pane::ChangedFiles],
        );

        dock_state
    }

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let theme = Theme::by_name(&config.theme).unwrap_or_else(Theme::dark);

        let (executor, message_rx) = JobExecutor::new();

        let pane_widths = config.pane_widths;

        // Load or create dock layout
        let dock_state = if let Some(layout_json) = &config.dock_layout {
            match serde_json::from_str(layout_json) {
                Ok(layout) => {
                    tracing::info!("Restored dock layout from config");
                    layout
                }
                Err(e) => {
                    tracing::warn!("Failed to parse dock layout: {}, using default", e);
                    Self::create_default_dock_layout()
                }
            }
        } else {
            tracing::info!("No saved dock layout, using default");
            Self::create_default_dock_layout()
        };

        let state = AppState {
            current_repo: None,
            loading: false,
            error: None,
            config,
            staging_progress: None,
            layout_config: crabontree_app::LayoutConfig { pane_widths },
        };

        Self {
            state,
            executor,
            message_rx,
            theme,
            active_panel: keyboard::ActivePanel::BranchTree,
            show_shortcuts_help: false,
            active_pane: 0,
            dock_state,
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
            Effect::CreateCommit { repo_path, message } => {
                self.executor
                    .submit(crabontree_app::Job::CreateCommit { repo_path, message });
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
            Effect::LoadFileTree(_path) => {
                // File tree pane removed - ignore
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

        // Auto-load missing data
        let (need_branch_tree, need_changed_files) = if let Some(repo) = &self.state.current_repo {
            (repo.branch_tree.is_none(), repo.changed_files.is_none())
        } else {
            (false, false)
        };

        if need_branch_tree {
            self.handle_message(crabontree_app::AppMessage::LoadBranchTreeRequested);
        }
        if need_changed_files {
            self.handle_message(crabontree_app::AppMessage::LoadChangedFilesRequested);
        }

        // Extract data we need before entering closures to avoid borrow checker issues
        let repo_data = if let Some(repo) = &self.state.current_repo {
            (
                repo.commits.clone(),
                repo.selected_commit.clone(),
                !repo.working_dir_files.is_empty(),
                repo.changed_files.clone(),
                repo.file_view.clone(),
            )
        } else {
            return;
        };

        // Collect messages to handle after rendering
        let mut messages = Vec::new();

        // Render with DockArea
        let mut viewer = PaneViewer {
            repo_data: &repo_data,
            messages: &mut messages,
        };

        DockArea::new(&mut self.dock_state)
            .show_inside(ui, &mut viewer);

        // Handle all collected messages
        for msg in messages {
            self.handle_message(msg);
        }
    }
}

/// TabViewer implementation for rendering panes in the dock
struct PaneViewer<'a> {
    repo_data: &'a (
        Vec<crabontree_app::Commit>,
        Option<String>,
        bool,
        Option<crabontree_app::ChangedFilesState>,
        crabontree_app::FileViewState,
    ),
    messages: &'a mut Vec<crabontree_app::AppMessage>,
}

impl<'a> egui_dock::TabViewer for PaneViewer<'a> {
    type Tab = panes::Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let (commits, selected_commit, has_working_dir_changes, changed_files, file_view) =
            self.repo_data;

        // Create scroll ID outside to avoid temporary value issues
        let scroll_id = format!("{:?}_dock_scroll", tab);

        // Render pane content with appropriate scroll configuration
        let scroll_config = match tab {
            panes::Pane::DiffViewer => panes::scrollable_pane::ScrollablePaneConfig::new_both_scroll(
                tab.title(),
                &scroll_id,
            ),
            _ => panes::scrollable_pane::ScrollablePaneConfig::new(
                tab.title(),
                &scroll_id,
            ),
        };

        panes::scrollable_pane::render(ui, &scroll_config, |ui| {
            match tab {
                panes::Pane::CommitHistory => {
                    let action = panes::commit_history::render(
                        ui,
                        commits,
                        selected_commit.as_ref(),
                        *has_working_dir_changes,
                    );
                    if let Some(msg) = panes::commit_history::action_to_message(action) {
                        self.messages.push(msg);
                    }
                }
                panes::Pane::ChangedFiles => {
                    if let Some(files) = changed_files {
                        let action = panes::changed_files::render(ui, files);
                        if let Some(msg) = panes::changed_files::action_to_message(action) {
                            self.messages.push(msg);
                        }
                    } else {
                        ui.label("Loading changed files...");
                    }
                }
                panes::Pane::DiffViewer => {
                    panes::diff_viewer::render(ui, file_view);
                }
            }
        });
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false // Prevent closing essential panes
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
        let error_action =
            components::error_panel::render(ctx, self.state.error.as_ref(), self.theme.error);
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
                    let welcome_action =
                        components::welcome_view::render(ui, &self.state.config.recent_repos);
                    if let Some(msg) = components::welcome_view::action_to_message(welcome_action) {
                        self.handle_message(msg);
                    }
                }
            });
    }
}

impl Drop for CrabOnTreeApp {
    fn drop(&mut self) {
        // Save dock layout to config
        match serde_json::to_string(&self.dock_state) {
            Ok(layout_json) => {
                self.state.config.dock_layout = Some(layout_json);
                if let Err(e) = save_config(&self.state.config) {
                    tracing::warn!("Failed to save config on exit: {}", e);
                } else {
                    tracing::info!("Saved dock layout to config");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize dock layout: {}", e);
            }
        }
    }
}
