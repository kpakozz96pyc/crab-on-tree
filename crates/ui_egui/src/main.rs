//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

use crabontree_app::{
    load_config, reduce, save_config, AppState, BranchTreeState, ChangedFilesState, Commit,
    DiffLineType, Effect, FileDiff, FileStatus, FileTreeState, FileViewState, JobExecutor,
    WorkingDirFile, WorkingDirStatus,
};
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

/// Active panel for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivePanel {
    BranchTree,
    FileTree,
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
    vim_g_pressed: bool,  // For 'gg' vim-style navigation
    // 4-pane layout state
    pane_widths: [f32; 4],
    active_pane: usize,
    branch_tree_search: String,
    file_tree_search: String,
    focused_branch_index: Option<usize>,
    focused_tree_node_index: Option<usize>,
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
            vim_g_pressed: false,
            pane_widths,
            active_pane: 0,
            branch_tree_search: String::new(),
            file_tree_search: String::new(),
            focused_branch_index: None,
            focused_tree_node_index: None,
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
            Effect::LoadFileTree(path) => {
                self.executor.submit(crabontree_app::Job::LoadFileTree(path));
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

    fn render_shortcuts_help(&mut self, ctx: &egui::Context) {
        egui::Window::new("⌨️ Keyboard Shortcuts")
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
            // Pane selection: 1, 2, 3, 4
            if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.active_pane = 0;
                self.active_panel = ActivePanel::BranchTree;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.active_pane = 1;
                self.active_panel = ActivePanel::FileTree;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.active_pane = 2;
                self.active_panel = ActivePanel::ChangedFiles;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num4)) {
                self.active_pane = 3;
                self.active_panel = ActivePanel::FileViewer;
            }

            // Tab to cycle through panes
            if ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
                self.active_pane = (self.active_pane + 1) % 4;
                self.active_panel = match self.active_pane {
                    0 => ActivePanel::BranchTree,
                    1 => ActivePanel::FileTree,
                    2 => ActivePanel::ChangedFiles,
                    _ => ActivePanel::FileViewer,
                };
            }

            // Shift+Tab to cycle backward
            if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift) {
                self.active_pane = if self.active_pane == 0 { 3 } else { self.active_pane - 1 };
                self.active_panel = match self.active_pane {
                    0 => ActivePanel::BranchTree,
                    1 => ActivePanel::FileTree,
                    2 => ActivePanel::ChangedFiles,
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

        // Load 4-pane data on first render
        let (need_branch_tree, need_file_tree, need_changed_files) = if let Some(repo) = &self.state.current_repo {
            (
                repo.branch_tree.is_none(),
                repo.file_tree.is_none(),
                repo.changed_files.is_none(),
            )
        } else {
            (false, false, false)
        };

        if need_branch_tree {
            self.handle_message(crabontree_app::AppMessage::LoadBranchTreeRequested);
        }
        if need_file_tree {
            self.handle_message(crabontree_app::AppMessage::LoadFileTreeRequested);
        }
        if need_changed_files {
            self.handle_message(crabontree_app::AppMessage::LoadChangedFilesRequested);
        }

        // Clone necessary data
        let (branch_tree, file_tree, changed_files, file_view) = if let Some(repo) = &self.state.current_repo {
            (
                repo.branch_tree.clone(),
                repo.file_tree.clone(),
                repo.changed_files.clone(),
                repo.file_view.clone(),
            )
        } else {
            return;
        };

        // Horizontal 4-pane layout - capture full height first
        let full_height = ui.available_height();
        ui.horizontal(|ui| {
            let available_width = ui.available_width();

            // Pane 1: Branch Tree
            let pane1_width = available_width * self.pane_widths[0];
            ui.allocate_ui(egui::vec2(pane1_width, full_height), |ui| {
                ui.set_min_height(full_height);
                ui.set_max_height(full_height);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().id_source("pane1_branch_tree_scroll").show(ui, |ui| {
                            ui.heading("Branches & Tags");
                            ui.separator();
                            if let Some(tree) = &branch_tree {
                                self.render_branch_tree_pane(ui, tree);
                            } else {
                                ui.label("Loading branches...");
                            }
                        });
                    });
            });

            // Separator 1
            self.render_vertical_separator(ui, 0, available_width, full_height);

            // Pane 2: File Tree
            let pane2_width = available_width * self.pane_widths[1];
            ui.allocate_ui(egui::vec2(pane2_width, full_height), |ui| {
                ui.set_min_height(full_height);
                ui.set_max_height(full_height);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().id_source("pane2_file_tree_scroll").show(ui, |ui| {
                            ui.heading("File Tree");
                            ui.separator();
                            if let Some(tree) = &file_tree {
                                self.render_file_tree_pane(ui, tree);
                            } else {
                                ui.label("Loading file tree...");
                            }
                        });
                    });
            });

            // Separator 2
            self.render_vertical_separator(ui, 1, available_width, full_height);

            // Pane 3: Changed Files
            let pane3_width = available_width * self.pane_widths[2];
            ui.allocate_ui(egui::vec2(pane3_width, full_height), |ui| {
                ui.set_min_height(full_height);
                ui.set_max_height(full_height);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().id_source("pane3_changed_files_scroll").show(ui, |ui| {
                            ui.heading("Changed Files");
                            ui.separator();
                            if let Some(files) = &changed_files {
                                self.render_changed_files_pane(ui, files);
                            } else {
                                ui.label("Loading changed files...");
                            }
                        });
                    });
            });

            // Separator 3
            self.render_vertical_separator(ui, 2, available_width, full_height);

            // Pane 4: File Viewer
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), full_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_min_height(full_height);
                    ui.set_max_height(full_height);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(25, 25, 30))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().id_source("pane4_file_viewer_scroll").show(ui, |ui| {
                                self.render_file_viewer_pane(ui, &file_view);
                            });
                        });
                },
            );
        });
    }

    fn render_vertical_separator(&mut self, ui: &mut egui::Ui, sep_idx: usize, total_width: f32, height: f32) {
        let sep_width = 8.0; // Wider hit area for easier dragging
        let visual_width = 2.0; // Thinner visual appearance

        let sep_rect = egui::Rect::from_min_size(
            ui.cursor().min - egui::vec2(sep_width / 2.0, 0.0), // Center the hit area
            egui::vec2(sep_width, height),
        );

        let sep_id = egui::Id::new(format!("pane_separator_{}", sep_idx));
        let response = ui.interact(sep_rect, sep_id, egui::Sense::drag());

        // Change cursor to indicate draggable
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }

        if response.dragged() {
            let delta = response.drag_delta().x / total_width;
            // Adjust pane widths
            if sep_idx < 3 {
                self.pane_widths[sep_idx] = (self.pane_widths[sep_idx] + delta).max(0.05).min(0.5);
                self.pane_widths[sep_idx + 1] = (self.pane_widths[sep_idx + 1] - delta).max(0.05).min(0.5);
                // Normalize
                let sum: f32 = self.pane_widths.iter().sum();
                for w in &mut self.pane_widths {
                    *w /= sum;
                }
                self.handle_message(crabontree_app::AppMessage::PaneWidthsUpdated(self.pane_widths));
            }
        }

        // Visual separator (thinner than hit area)
        let visual_rect = egui::Rect::from_min_size(
            ui.cursor().min - egui::vec2(visual_width / 2.0, 0.0),
            egui::vec2(visual_width, height),
        );

        let color = if response.hovered() || response.dragged() {
            egui::Color32::from_rgb(100, 150, 200)
        } else {
            egui::Color32::from_rgb(60, 60, 65)
        };

        ui.painter().rect_filled(visual_rect, 1.0, color);
        ui.allocate_space(egui::vec2(sep_width, 0.0));
    }

    fn render_branch_tree_pane(&mut self, ui: &mut egui::Ui, tree: &BranchTreeState) {
        // Commit History section
        let commits = if let Some(repo) = &self.state.current_repo {
            repo.commits.clone()
        } else {
            Vec::new()
        };

        let selected_commit = if let Some(repo) = &self.state.current_repo {
            repo.selected_commit.clone()
        } else {
            None
        };

        egui::CollapsingHeader::new(format!("📜 Commit History ({})", commits.len()))
            .id_source("commit_history_section")
            .default_open(true)
            .show(ui, |ui| {
                if commits.is_empty() {
                    if ui.button("Load Commit History").clicked() {
                        self.handle_message(crabontree_app::AppMessage::LoadCommitHistoryRequested);
                    }
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("commit_history_list")
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (idx, commit) in commits.iter().enumerate() {
                                ui.push_id(format!("commit_{}", idx), |ui| {
                                    let is_selected = selected_commit.as_ref() == Some(&commit.hash);
                                    let label = format!("{} - {}",
                                        &commit.hash[..7],
                                        commit.message_summary
                                    );

                                    if ui.selectable_label(is_selected, label)
                                        .on_hover_text(&commit.hash)
                                        .clicked()
                                    {
                                        if is_selected {
                                            self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                                        } else {
                                            self.handle_message(crabontree_app::AppMessage::CommitSelected(commit.hash.clone()));
                                        }
                                    }
                                });
                            }
                        });
                }
            });

        ui.add_space(10.0);

        // Local branches section
        let local_header = egui::CollapsingHeader::new("📂 Local Branches")
            .id_source("branch_tree_local")
            .show(ui, |ui| {
                for (idx, branch) in tree.local_branches.iter().enumerate() {
                    ui.push_id(format!("local_branch_{}", idx), |ui| {
                        ui.horizontal(|ui| {
                            let label = if branch.is_current {
                                format!("➤ {}", branch.name)
                            } else {
                                format!("  {}", branch.name)
                            };

                            if ui.selectable_label(branch.is_current, label).clicked() && !branch.is_current {
                                self.handle_message(crabontree_app::AppMessage::BranchCheckoutRequested(branch.name.clone()));
                            }
                        });
                    });
                }
            });
        if local_header.header_response.clicked() {
            self.handle_message(crabontree_app::AppMessage::BranchSectionToggled("local".to_string()));
        }

        ui.add_space(10.0);

        // Remote branches section
        for (remote_idx, (remote, branches)) in tree.remote_branches.iter().enumerate() {
            let remote_header = egui::CollapsingHeader::new(format!("📡 {}", remote))
                .id_source(format!("branch_tree_remote_{}", remote_idx))
                .show(ui, |ui| {
                    for (branch_idx, branch) in branches.iter().enumerate() {
                        ui.push_id(format!("remote_{}_{}", remote_idx, branch_idx), |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {}", branch.name));
                            });
                        });
                    }
                });
            if remote_header.header_response.clicked() {
                self.handle_message(crabontree_app::AppMessage::BranchSectionToggled(remote.clone()));
            }
        }

        ui.add_space(10.0);

        // Tags section
        let tags_header = egui::CollapsingHeader::new(format!("🏷  Tags ({})", tree.tags.len()))
            .id_source("branch_tree_tags")
            .show(ui, |ui| {
                for (idx, tag) in tree.tags.iter().enumerate() {
                    ui.push_id(format!("tag_{}", idx), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("  {}", tag.name));
                        });
                    });
                }
            });
        if tags_header.header_response.clicked() {
            self.handle_message(crabontree_app::AppMessage::BranchSectionToggled("tags".to_string()));
        }
    }

    fn render_file_tree_pane(&mut self, ui: &mut egui::Ui, tree: &FileTreeState) {
        use crabontree_app::FileTreeNode;

        fn render_node(
            app: &mut CrabOnTreeApp,
            ui: &mut egui::Ui,
            node: &FileTreeNode,
            depth: usize,
            expanded_paths: &std::collections::HashSet<std::path::PathBuf>,
            selected_path: &Option<std::path::PathBuf>,
        ) {
            let indent = "  ".repeat(depth);

            match node {
                FileTreeNode::Directory { path, name, children, .. } => {
                    let is_expanded = expanded_paths.contains(path);
                    let icon = if is_expanded { "📂" } else { "📁" };

                    // Use path as unique ID
                    ui.push_id(format!("dir_{}", path.display()), |ui| {
                        if ui.selectable_label(false, format!("{}{} {}", indent, icon, name)).clicked() {
                            app.handle_message(crabontree_app::AppMessage::FileTreeNodeToggled(path.clone()));
                        }
                    });

                    if is_expanded {
                        for child in children {
                            render_node(app, ui, child, depth + 1, expanded_paths, selected_path);
                        }
                    }
                }
                FileTreeNode::File { path, name, status, .. } => {
                    let icon = if status.is_some() { "📝" } else { "📄" };
                    let is_selected = selected_path.as_ref() == Some(path);

                    // Use path as unique ID
                    ui.push_id(format!("file_{}", path.display()), |ui| {
                        if ui.selectable_label(is_selected, format!("{}{} {}", indent, icon, name)).clicked() {
                            app.handle_message(crabontree_app::AppMessage::FileTreeNodeSelected(path.clone()));
                        }
                    });
                }
            }
        }

        render_node(self, ui, &tree.root, 0, &tree.expanded_paths, &tree.selected_path);
    }

    fn render_changed_files_pane(&mut self, ui: &mut egui::Ui, files: &ChangedFilesState) {
        // Staged files
        if !files.staged.is_empty() {
            egui::CollapsingHeader::new(format!("✅ Staged ({})", files.staged.len()))
                .id_source("changed_files_staged")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.staged.iter().enumerate() {
                        ui.push_id(format!("staged_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            if ui.selectable_label(is_selected, format!("  {}", file.path.display())).clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Unstaged files
        if !files.unstaged.is_empty() {
            egui::CollapsingHeader::new(format!("📝 Unstaged ({})", files.unstaged.len()))
                .id_source("changed_files_unstaged")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.unstaged.iter().enumerate() {
                        ui.push_id(format!("unstaged_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            if ui.selectable_label(is_selected, format!("  {}", file.path.display())).clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Untracked files
        if !files.untracked.is_empty() {
            egui::CollapsingHeader::new(format!("❓ Untracked ({})", files.untracked.len()))
                .id_source("changed_files_untracked")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.untracked.iter().enumerate() {
                        ui.push_id(format!("untracked_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            if ui.selectable_label(is_selected, format!("  {}", file.path.display())).clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Conflicted files
        if !files.conflicted.is_empty() {
            egui::CollapsingHeader::new(format!("⚠ Conflicted ({})", files.conflicted.len()))
                .id_source("changed_files_conflicted")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.conflicted.iter().enumerate() {
                        ui.push_id(format!("conflicted_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            if ui.selectable_label(is_selected, format!("  {}", file.path.display())).clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
        }
    }

    fn render_file_viewer_pane(&mut self, ui: &mut egui::Ui, state: &FileViewState) {
        // Check if we have a selected commit to show
        let (selected_commit, commit_diff) = if let Some(repo) = &self.state.current_repo {
            (repo.selected_commit.clone(), repo.commit_diff.clone())
        } else {
            (None, None)
        };

        // Show commit diff if a commit is selected
        if let (Some(commit_hash), Some(diff)) = (selected_commit, commit_diff) {
            ui.heading(format!("Commit: {}", &commit_hash[..7]));
            ui.separator();
            ui.add_space(5.0);

            if diff.is_empty() {
                ui.label("No changes in this commit");
            } else {
                ui.label(format!("{} file(s) changed", diff.len()));
                ui.add_space(10.0);

                egui::ScrollArea::vertical().id_source("commit_diff_files_scroll").show(ui, |ui| {
                    for (idx, file_diff) in diff.iter().enumerate() {
                        ui.push_id(format!("commit_file_{}", idx), |ui| {
                            let (status_symbol, status_color) = match file_diff.status {
                                crabontree_app::FileStatus::Added => ("+", egui::Color32::from_rgb(0, 200, 0)),
                                crabontree_app::FileStatus::Modified => ("~", egui::Color32::from_rgb(200, 150, 0)),
                                crabontree_app::FileStatus::Deleted => ("-", egui::Color32::from_rgb(200, 0, 0)),
                                _ => ("•", egui::Color32::from_rgb(150, 150, 150)),
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(status_color, egui::RichText::new(status_symbol).strong());
                                ui.label(&file_diff.path);
                            });
                        });
                    }
                });
            }
            return;
        }

        // Otherwise show file view
        match state {
            FileViewState::None => {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label("Select a file or commit to view");
                });
            }
            FileViewState::Content { path, content, .. } => {
                ui.heading(path.display().to_string());
                ui.separator();
                ui.add_space(5.0);

                // Show file content with line numbers
                egui::ScrollArea::both().id_source("file_content_scroll").show(ui, |ui| {
                    for (i, line) in content.lines().enumerate() {
                        ui.push_id(format!("content_line_{}", i), |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{:4} ", i + 1)).monospace().weak());
                                ui.label(egui::RichText::new(line).monospace());
                            });
                        });
                    }
                });
            }
            FileViewState::Diff { path, hunks, .. } => {
                ui.heading(format!("Diff: {}", path.display()));
                ui.separator();
                ui.add_space(5.0);

                // Render diff hunks
                egui::ScrollArea::both().id_source("file_diff_scroll").show(ui, |ui| {
                    for (hunk_idx, hunk) in hunks.iter().enumerate() {
                        ui.push_id(format!("hunk_{}", hunk_idx), |ui| {
                            ui.label(egui::RichText::new(format!(
                                "@@ -{},{} +{},{} @@",
                                hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
                            )).monospace().weak());

                            for (line_idx, line) in hunk.lines.iter().enumerate() {
                                ui.push_id(format!("line_{}", line_idx), |ui| {
                                    let (prefix, color) = match line.line_type {
                                        crabontree_app::DiffLineType::Addition => ("+", egui::Color32::from_rgb(0, 200, 0)),
                                        crabontree_app::DiffLineType::Deletion => ("-", egui::Color32::from_rgb(200, 0, 0)),
                                        crabontree_app::DiffLineType::Context => (" ", egui::Color32::from_rgb(200, 200, 200)),
                                    };

                                    ui.horizontal(|ui| {
                                        let line_num = line.old_line_number.or(line.new_line_number)
                                            .map(|n| format!("{:4}", n))
                                            .unwrap_or_else(|| "    ".to_string());
                                        ui.label(egui::RichText::new(line_num).monospace().weak());
                                        ui.colored_label(color, egui::RichText::new(format!("{}{}", prefix, line.content.trim_end())).monospace());
                                    });
                                });
                            }

                            ui.add_space(10.0);
                        });
                    }
                });
            }
            FileViewState::Binary { path, size } => {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading(path.display().to_string());
                    ui.add_space(20.0);
                    ui.label(format!("Binary file ({} bytes)", size));
                    ui.label("Cannot display binary content");
                });
            }
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

        // Render help dialog if requested
        if self.show_shortcuts_help {
            self.render_shortcuts_help(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.current_repo.is_some() {
                self.render_repository_view(ui);
            } else {
                self.render_welcome_view(ui);
            }
        });
    }
}
