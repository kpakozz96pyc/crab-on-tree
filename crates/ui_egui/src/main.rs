//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

mod widgets;

use crabontree_app::{
    load_config, reduce, save_config, AppState, ChangedFilesState, Effect,
    FileViewState, JobExecutor,
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
                egui::ScrollArea::vertical()
                    .id_source("commit_history_scroll")
                    .show(ui, |ui| {
                        self.render_commit_history_pane(ui);
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
                egui::ScrollArea::vertical()
                    .id_source("diff_viewer_scroll")
                    .show(ui, |ui| {
                        self.render_file_viewer_pane(ui, &file_view);
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
                egui::ScrollArea::vertical()
                    .id_source("changed_files_scroll")
                    .show(ui, |ui| {
                        if let Some(files) = &changed_files {
                            self.render_changed_files_pane(ui, files);
                        } else {
                            ui.label("Loading changed files...");
                        }
                    });
            });
    }

    fn render_commit_history_pane(&mut self, ui: &mut egui::Ui) {
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

        let has_working_dir_changes = if let Some(repo) = &self.state.current_repo {
            !repo.working_dir_files.is_empty()
        } else {
            false
        };

        if commits.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                if ui.button("Load Commit History").clicked() {
                    self.handle_message(crabontree_app::AppMessage::LoadCommitHistoryRequested);
                }
            });
        } else {
            // Show Working Directory as first commit (0000000)
            ui.push_id("working_directory", |ui| {
                let is_selected = selected_commit.as_ref() == Some(&crabontree_app::WORKING_DIR_HASH.to_string());
                let status_indicator = if has_working_dir_changes { " *" } else { "" };
                let text = format!("0000000 - Working Directory{}", status_indicator);

                if widgets::selectable_row(ui, text, is_selected) {
                    if is_selected {
                        self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                    } else {
                        self.handle_message(crabontree_app::AppMessage::CommitSelected(crabontree_app::WORKING_DIR_HASH.to_string()));
                    }
                }
            });

            // Show regular commits
            for (idx, commit) in commits.iter().enumerate() {
                ui.push_id(format!("commit_{}", idx), |ui| {
                    let is_selected = selected_commit.as_ref() == Some(&commit.hash);
                    let text = format!("{} - {}", &commit.hash[..7], commit.message_summary);

                    if widgets::selectable_row(ui, text, is_selected) {
                        if is_selected {
                            self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                        } else {
                            self.handle_message(crabontree_app::AppMessage::CommitSelected(commit.hash.clone()));
                        }
                    }
                });
            }
        }
    }


    fn render_changed_files_pane(&mut self, ui: &mut egui::Ui, files: &ChangedFilesState) {
        // Helper function to get status icon and color
        let get_status_info = |status: &crabontree_app::WorkingDirStatus| {
            match status {
                crabontree_app::WorkingDirStatus::Modified => ("~", egui::Color32::from_rgb(200, 150, 0)),
                crabontree_app::WorkingDirStatus::Deleted => ("-", egui::Color32::from_rgb(200, 0, 0)),
                crabontree_app::WorkingDirStatus::Untracked => ("+", egui::Color32::from_rgb(0, 200, 0)),
                crabontree_app::WorkingDirStatus::Renamed => ("R", egui::Color32::from_rgb(100, 150, 200)),
                crabontree_app::WorkingDirStatus::Conflicted => ("!", egui::Color32::from_rgb(200, 0, 200)),
                crabontree_app::WorkingDirStatus::TypeChanged => ("T", egui::Color32::from_rgb(150, 100, 200)),
            }
        };

        // Staged files
        if !files.staged.is_empty() {
            egui::CollapsingHeader::new(format!("Staged ({})", files.staged.len()))
                .id_source("changed_files_staged")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.staged.iter().enumerate() {
                        ui.push_id(format!("staged_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            let (status_icon, status_color) = get_status_info(&file.status);

                            if ui.horizontal(|ui| {
                                ui.colored_label(status_color, egui::RichText::new(status_icon).strong());
                                ui.selectable_label(is_selected, file.path.display().to_string())
                            }).inner.clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Unstaged files
        if !files.unstaged.is_empty() {
            egui::CollapsingHeader::new(format!("Unstaged ({})", files.unstaged.len()))
                .id_source("changed_files_unstaged")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.unstaged.iter().enumerate() {
                        ui.push_id(format!("unstaged_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            let (status_icon, status_color) = get_status_info(&file.status);

                            if ui.horizontal(|ui| {
                                ui.colored_label(status_color, egui::RichText::new(status_icon).strong());
                                ui.selectable_label(is_selected, file.path.display().to_string())
                            }).inner.clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Untracked files
        if !files.untracked.is_empty() {
            egui::CollapsingHeader::new(format!("Untracked ({})", files.untracked.len()))
                .id_source("changed_files_untracked")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.untracked.iter().enumerate() {
                        ui.push_id(format!("untracked_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            let (status_icon, status_color) = get_status_info(&file.status);

                            if ui.horizontal(|ui| {
                                ui.colored_label(status_color, egui::RichText::new(status_icon).strong());
                                ui.selectable_label(is_selected, file.path.display().to_string())
                            }).inner.clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
            ui.add_space(5.0);
        }

        // Conflicted files
        if !files.conflicted.is_empty() {
            egui::CollapsingHeader::new(format!("Conflicted ({})", files.conflicted.len()))
                .id_source("changed_files_conflicted")
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, file) in files.conflicted.iter().enumerate() {
                        ui.push_id(format!("conflicted_{}", idx), |ui| {
                            let is_selected = files.selected_file.as_ref() == Some(&file.path);
                            let (status_icon, status_color) = get_status_info(&file.status);

                            if ui.horizontal(|ui| {
                                ui.colored_label(status_color, egui::RichText::new(status_icon).strong());
                                ui.selectable_label(is_selected, file.path.display().to_string())
                            }).inner.clicked() {
                                self.handle_message(crabontree_app::AppMessage::ChangedFileSelected(file.path.clone()));
                            }
                        });
                    }
                });
        }
    }

    fn render_file_viewer_pane(&mut self, ui: &mut egui::Ui, state: &FileViewState) {
        // Show file view based on state
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
