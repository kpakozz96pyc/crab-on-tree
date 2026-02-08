//! CrabOnTree - A Rust Git GUI using gitoxide and egui.

use crabontree_app::{
    load_config, reduce, save_config, AppState, Commit, DiffLineType, Effect, FileDiff, FileStatus,
    JobExecutor, WorkingDirFile, WorkingDirStatus,
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
    WorkingDirectory,
    CommitMessage,
    CommitHistory,
}

struct CrabOnTreeApp {
    state: AppState,
    executor: JobExecutor,
    message_rx: tokio::sync::mpsc::Receiver<crabontree_app::AppMessage>,
    theme: Theme,
    commit_message_buffer: String,
    working_dir_search: String,
    commit_search: String,
    focused_commit_index: Option<usize>,
    focused_file_index: Option<usize>,
    active_panel: ActivePanel,
    show_shortcuts_help: bool,
    vim_g_pressed: bool,  // For 'gg' vim-style navigation
    // Panel sizes (resizable)
    working_dir_height: f32,
    commit_panel_height: f32,
    commit_list_width_ratio: f32,  // Ratio of commit list width (0.0 to 1.0)
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
            staging_progress: None,
        };

        Self {
            state,
            executor,
            message_rx,
            theme,
            commit_message_buffer: String::new(),
            working_dir_search: String::new(),
            commit_search: String::new(),
            focused_commit_index: None,
            focused_file_index: None,
            active_panel: ActivePanel::WorkingDirectory,
            show_shortcuts_help: false,
            vim_g_pressed: false,
            // Default panel sizes
            working_dir_height: 300.0,
            commit_panel_height: 200.0,
            commit_list_width_ratio: 0.4,
        }
    }

    fn poll_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, message: crabontree_app::AppMessage) {
        // Handle UI-specific updates before reducer
        if let crabontree_app::AppMessage::CommitCreated { .. } = &message {
            // Clear the commit message buffer on successful commit
            self.commit_message_buffer.clear();
        }

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

        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_response = egui::TextEdit::singleline(&mut self.commit_search)
                .hint_text("Search commits...")
                .desired_width(f32::INFINITY)
                .id(egui::Id::new("commit_search_input"))
                .show(ui);

            // Clear button
            if !self.commit_search.is_empty() {
                if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                    self.commit_search.clear();
                }
            }

            // Esc to clear search when focused
            if search_response.response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.commit_search.clear();
                search_response.response.surrender_focus();
            }
        });

        ui.add_space(5.0);

        // Filter commits based on search query (search in message, author, and hash)
        let search_lower = self.commit_search.to_lowercase();
        let filtered_commits: Vec<&Commit> = if search_lower.is_empty() {
            commits.iter().collect()
        } else {
            commits.iter()
                .filter(|c| {
                    c.message.to_lowercase().contains(&search_lower) ||
                    c.author_name.to_lowercase().contains(&search_lower) ||
                    c.hash.to_lowercase().contains(&search_lower) ||
                    c.hash_short.to_lowercase().contains(&search_lower)
                })
                .collect()
        };

        // Show filtered count if search is active
        if !search_lower.is_empty() {
            ui.label(format!("Showing {} of {} commits", filtered_commits.len(), commits.len()));
            ui.add_space(3.0);
        }

        ui.separator();

        // Handle keyboard navigation for commit history (only if this panel is active)
        let search_focused = ui.memory(|mem| {
            mem.focused() == Some(egui::Id::new("commit_search_input"))
        });

        if self.active_panel == ActivePanel::CommitHistory && !search_focused {
            let num_commits = filtered_commits.len();
            if num_commits > 0 {
                // Initialize focus to first commit if not set
                if self.focused_commit_index.is_none() {
                    self.focused_commit_index = Some(0);
                }

                // Arrow keys and vim keys for navigation
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J)) {
                    if let Some(idx) = self.focused_commit_index {
                        self.focused_commit_index = Some((idx + 1).min(num_commits - 1));
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K)) {
                    if let Some(idx) = self.focused_commit_index {
                        self.focused_commit_index = Some(idx.saturating_sub(1));
                    }
                }

                // Home/End keys
                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    self.focused_commit_index = Some(0);
                }
                if ui.input(|i| i.key_pressed(egui::Key::End)) {
                    self.focused_commit_index = Some(num_commits - 1);
                }

                // Enter to select focused commit
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(idx) = self.focused_commit_index {
                        if let Some(commit) = filtered_commits.get(idx) {
                            let is_selected = selected.as_ref() == Some(&commit.hash);
                            if is_selected {
                                self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                            } else {
                                self.handle_message(crabontree_app::AppMessage::CommitSelected(commit.hash.clone()));
                            }
                        }
                    }
                }
            }
        }

        egui::ScrollArea::vertical()
            .id_source("commit_history_scroll")
            .show(ui, |ui| {
                for (idx, commit) in filtered_commits.iter().enumerate() {
                    let is_selected = selected.as_ref() == Some(&commit.hash);
                    let is_focused = self.focused_commit_index == Some(idx) &&
                                     self.active_panel == ActivePanel::CommitHistory;

                    // Add visual indicator for focused item
                    let text = format!(
                        "{}{}",
                        if is_focused { "> " } else { "  " },
                        format!("{} - {} - {}",
                            commit.hash_short,
                            commit.message_summary,
                            commit.author_name
                        )
                    );

                    // Use commit hash as unique ID for each selectable label
                    let response = ui.selectable_label(is_selected, text)
                        .on_hover_text(&commit.hash);

                    if response.clicked() {
                        self.focused_commit_index = Some(idx);
                        if is_selected {
                            self.handle_message(crabontree_app::AppMessage::CommitDeselected);
                        } else {
                            self.handle_message(crabontree_app::AppMessage::CommitSelected(commit.hash.clone()));
                        }
                    }
                }
            });
    }

    fn render_commit_panel(&mut self, ui: &mut egui::Ui, files: &[WorkingDirFile], author_name: &str, author_email: &str) {
        let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );

        // Count staged files
        let staged_count = files.iter().filter(|f| f.is_staged).count();

        ui.heading("💬 Commit Message");
        ui.add_space(5.0);

        // Multi-line text editor for commit message
        let text_edit_response = egui::TextEdit::multiline(&mut self.commit_message_buffer)
            .desired_width(f32::INFINITY)
            .desired_rows(5)
            .hint_text("Enter commit message...")
            .id(egui::Id::new("commit_message_input"))
            .show(ui);

        // Handle keyboard shortcuts
        let text_edit_focused = text_edit_response.response.has_focus();

        // Ctrl+Enter to commit (when focused and valid)
        if text_edit_focused && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
            let message_valid = !self.commit_message_buffer.trim().is_empty();
            let can_commit = message_valid && staged_count > 0;

            if can_commit {
                self.handle_message(crabontree_app::AppMessage::CreateCommitRequested);
                self.commit_message_buffer.clear();
            }
        }

        // Esc to blur/clear
        if text_edit_focused && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            text_edit_response.response.surrender_focus();
        }

        // Sync buffer with state if changed
        if text_edit_response.response.changed() {
            self.handle_message(crabontree_app::AppMessage::CommitMessageUpdated(
                self.commit_message_buffer.clone()
            ));
        }

        ui.add_space(5.0);

        // Commit statistics
        let char_count = self.commit_message_buffer.chars().count();
        let line_count = self.commit_message_buffer.lines().count();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} chars | {} lines | {} files staged",
                    char_count, line_count, staged_count
                ))
                .color(to_egui(self.theme.fg_tertiary))
                .size(12.0)
            );
        });

        ui.add_space(5.0);

        // Author identity display
        if !author_name.is_empty() {
            ui.label(
                egui::RichText::new(format!("Committing as: {} <{}>", author_name, author_email))
                .color(to_egui(self.theme.fg_tertiary))
                .size(11.0)
            );
            ui.add_space(5.0);
        }

        // Commit button - only enabled when message is not empty and files are staged
        let message_valid = !self.commit_message_buffer.trim().is_empty();
        let can_commit = message_valid && staged_count > 0;

        ui.horizontal(|ui| {
            let commit_button = egui::Button::new(
                if staged_count > 0 {
                    format!("📝 Commit ({} files)", staged_count)
                } else {
                    "📝 Commit".to_string()
                }
            );

            let button_response = ui.add_enabled(can_commit, commit_button);

            if button_response.clicked() {
                self.handle_message(crabontree_app::AppMessage::CreateCommitRequested);
                // Clear the buffer immediately (state will be cleared by reducer)
                self.commit_message_buffer.clear();
            }

            // Show tooltip explaining why button is disabled
            if !can_commit {
                button_response.on_hover_text(
                    if staged_count == 0 {
                        "No files staged for commit"
                    } else {
                        "Commit message cannot be empty"
                    }
                );
            }
        });
    }

    fn render_working_directory(&mut self, ui: &mut egui::Ui, files: &[WorkingDirFile]) {
        let to_egui = |c: Color| egui::Color32::from_rgba_premultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );

        if files.is_empty() {
            ui.label("✓ Working directory is clean");
            return;
        }

        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_response = egui::TextEdit::singleline(&mut self.working_dir_search)
                .hint_text("Search files...")
                .desired_width(200.0)
                .id(egui::Id::new("working_dir_search_input"))
                .show(ui);

            // Clear button
            if !self.working_dir_search.is_empty() {
                if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                    self.working_dir_search.clear();
                }
            }

            // Esc to clear search when focused
            if search_response.response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.working_dir_search.clear();
                search_response.response.surrender_focus();
            }
        });

        ui.add_space(5.0);

        // Filter files based on search query
        let search_lower = self.working_dir_search.to_lowercase();
        let filtered_files: Vec<&WorkingDirFile> = if search_lower.is_empty() {
            files.iter().collect()
        } else {
            files.iter()
                .filter(|f| f.path.to_string_lossy().to_lowercase().contains(&search_lower))
                .collect()
        };

        // Count staged and unstaged in filtered results (for potential future use)
        let (_staged_count, _unstaged_count) = filtered_files.iter().fold((0, 0), |(s, u), f| {
            if f.is_staged { (s + 1, u) } else { (s, u + 1) }
        });

        // Show filtered count if search is active
        if !search_lower.is_empty() {
            ui.label(
                egui::RichText::new(format!("Showing {} of {} files", filtered_files.len(), files.len()))
                .color(to_egui(self.theme.fg_tertiary))
                .size(12.0)
            );
            ui.add_space(3.0);
        }

        // Action buttons for bulk operations (only for ALL files, not filtered)
        ui.horizontal(|ui| {
            let all_unstaged = files.iter().filter(|f| !f.is_staged).count();
            let all_staged = files.iter().filter(|f| f.is_staged).count();

            if all_unstaged > 0 && ui.button(format!("➕ Stage All ({})", all_unstaged)).clicked() {
                self.handle_message(crabontree_app::AppMessage::StageAllRequested);
            }
            if all_staged > 0 && ui.button(format!("➖ Unstage All ({})", all_staged)).clicked() {
                self.handle_message(crabontree_app::AppMessage::UnstageAllRequested);
            }
        });

        ui.add_space(5.0);

        // Handle keyboard navigation for files (only if this panel is active)
        let search_focused = ui.memory(|mem| {
            mem.focused() == Some(egui::Id::new("working_dir_search_input"))
        });

        if self.active_panel == ActivePanel::WorkingDirectory && !search_focused {
            let num_files = filtered_files.len();
            if num_files > 0 {
                // Initialize focus to first file if not set
                if self.focused_file_index.is_none() {
                    self.focused_file_index = Some(0);
                }

                // Arrow keys and vim keys for navigation
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J)) {
                    if let Some(idx) = self.focused_file_index {
                        self.focused_file_index = Some((idx + 1).min(num_files - 1));
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K)) {
                    if let Some(idx) = self.focused_file_index {
                        self.focused_file_index = Some(idx.saturating_sub(1));
                    }
                }

                // Home/End keys
                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    self.focused_file_index = Some(0);
                }
                if ui.input(|i| i.key_pressed(egui::Key::End)) {
                    self.focused_file_index = Some(num_files - 1);
                }

                // Space to toggle staging for focused file
                if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                    if let Some(idx) = self.focused_file_index {
                        if let Some(file) = filtered_files.get(idx) {
                            if file.is_staged {
                                self.handle_message(crabontree_app::AppMessage::UnstageFileRequested(file.path.clone()));
                            } else {
                                self.handle_message(crabontree_app::AppMessage::StageFileRequested(file.path.clone()));
                            }
                        }
                    }
                }
            }
        }

        // Use egui's built-in virtual scrolling for performance
        let row_height = ui.spacing().interact_size.y;

        egui::ScrollArea::vertical()
            .id_source("working_dir_scroll")
            .max_height(200.0)
            .auto_shrink([false; 2])
            .show_rows(ui, row_height, filtered_files.len(), |ui, row_range| {
                // Render only visible rows
                for idx in row_range {
                    if let Some(file) = filtered_files.get(idx) {
                        let is_focused = self.focused_file_index == Some(idx) &&
                                         self.active_panel == ActivePanel::WorkingDirectory;
                        let (status_symbol, status_color, status_text) = match file.status {
                            WorkingDirStatus::Modified => ("~", self.theme.git_modified, if file.is_staged { "Modified" } else { "Modified" }),
                            WorkingDirStatus::Untracked => ("+", self.theme.git_added, if file.is_staged { "Added" } else { "Untracked" }),
                            WorkingDirStatus::Deleted => ("-", self.theme.git_deleted, "Deleted"),
                            WorkingDirStatus::Renamed => ("→", self.theme.git_modified, "Renamed"),
                            WorkingDirStatus::Conflicted => ("⚠", self.theme.warning, "Conflicted"),
                            WorkingDirStatus::TypeChanged => ("*", self.theme.git_modified, "Type Changed"),
                        };

                        ui.horizontal(|ui| {
                            // Focus indicator
                            if is_focused {
                                ui.label(">");
                            } else {
                                ui.label(" ");
                            }

                            // Stage/Unstage button
                            let button_response = if file.is_staged {
                                ui.small_button("−").on_hover_text("Unstage (Space)")
                            } else {
                                ui.small_button("+").on_hover_text("Stage (Space)")
                            };

                            if button_response.clicked() {
                                self.focused_file_index = Some(idx);
                                if file.is_staged {
                                    self.handle_message(crabontree_app::AppMessage::UnstageFileRequested(file.path.clone()));
                                } else {
                                    self.handle_message(crabontree_app::AppMessage::StageFileRequested(file.path.clone()));
                                }
                            }

                            ui.colored_label(to_egui(status_color), status_symbol);

                            // Show [S] or [U] prefix for staged/unstaged
                            let prefix = if file.is_staged { "[S]" } else { "[U]" };
                            ui.label(egui::RichText::new(prefix).color(
                                if file.is_staged {
                                    to_egui(self.theme.success)
                                } else {
                                    to_egui(self.theme.fg_tertiary)
                                }
                            ));

                            let file_label = ui.label(file.path.display().to_string())
                                .on_hover_text(status_text);

                            // Allow clicking on file name to focus it
                            if file_label.clicked() {
                                self.focused_file_index = Some(idx);
                            }
                        });
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
                for (file_idx, file) in diff.iter().enumerate() {
                    let (status_symbol, status_color) = match file.status {
                        FileStatus::Added => ("+", self.theme.git_added),
                        FileStatus::Modified => ("~", self.theme.git_modified),
                        FileStatus::Deleted => ("-", self.theme.git_deleted),
                        FileStatus::Renamed => ("→", self.theme.accent_primary),
                        FileStatus::Copied => ("©", self.theme.accent_primary),
                    };

                    // File header with collapsible diff content
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!(
                            "{} {} (+{} -{}) ",
                            status_symbol,
                            file.path,
                            file.additions,
                            file.deletions
                        ))
                        .color(to_egui(status_color))
                        .strong()
                    )
                    .id_source(format!("file_diff_{}", file_idx))
                    .default_open(false)
                    .show(ui, |ui| {
                        // Render hunks
                        if file.hunks.is_empty() {
                            ui.label(egui::RichText::new("Binary file or no diff available").italics());
                        } else {
                            for hunk in &file.hunks {
                                // Hunk header
                                ui.label(
                                    egui::RichText::new(format!(
                                        "@@ -{},{} +{},{} @@",
                                        hunk.old_start, hunk.old_lines,
                                        hunk.new_start, hunk.new_lines
                                    ))
                                    .color(to_egui(self.theme.info))
                                    .monospace()
                                );

                                // Render lines
                                for line in &hunk.lines {
                                    let (prefix, color, line_num) = match line.line_type {
                                        DiffLineType::Addition => (
                                            "+",
                                            to_egui(self.theme.git_added),
                                            line.new_line_number.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string())
                                        ),
                                        DiffLineType::Deletion => (
                                            "-",
                                            to_egui(self.theme.git_deleted),
                                            line.old_line_number.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string())
                                        ),
                                        DiffLineType::Context => (
                                            " ",
                                            to_egui(self.theme.fg_secondary),
                                            line.new_line_number.map(|n| format!("{:>4}", n)).unwrap_or_else(|| "    ".to_string())
                                        ),
                                    };

                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        ui.label(egui::RichText::new(line_num).monospace().color(to_egui(self.theme.fg_tertiary)));
                                        ui.label(
                                            egui::RichText::new(format!("{}{}", prefix, line.content.trim_end()))
                                                .monospace()
                                                .color(color)
                                        );
                                    });
                                }

                                ui.add_space(5.0);
                            }
                        }
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
        // Clone data we need at the start to avoid borrowing issues
        let repo_data = self.state.current_repo.as_ref().map(|repo| (
                repo.path.clone(),
                repo.head.clone(),
                repo.branches.clone(),
                repo.status_summary.clone(),
                repo.commits.clone(),
                repo.selected_commit.clone(),
                repo.commit_diff.clone(),
                repo.working_dir_files.clone(),
                repo.author_name.clone(),
                repo.author_email.clone(),
            ));

        if let Some((path, head, branches, status, commits, selected_commit, commit_diff, working_dir_files, author_name, author_email)) = repo_data {
            // Global keyboard shortcuts
            // 'c' key to focus commit message input
            if ui.input(|i| i.key_pressed(egui::Key::C) && !i.modifiers.ctrl && !i.modifiers.alt) {
                ui.memory_mut(|mem| {
                    mem.request_focus(egui::Id::new("commit_message_input"));
                });
            }

            // '/' key to focus working directory search
            if ui.input(|i| i.key_pressed(egui::Key::Slash) && !i.modifiers.ctrl && !i.modifiers.alt) {
                ui.memory_mut(|mem| {
                    mem.request_focus(egui::Id::new("working_dir_search_input"));
                });
            }

            // 'Ctrl+F' to focus commit history search
            if ui.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
                ui.memory_mut(|mem| {
                    mem.request_focus(egui::Id::new("commit_search_input"));
                });
            }

            // Check if any text input is focused (to avoid capturing typing)
            let any_text_focused = ui.memory(|mem| {
                mem.focused().is_some()
            });

            // Only handle global shortcuts if no text input is focused
            if !any_text_focused {
                // Panel navigation: 1, 2, 3
                if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                    self.active_panel = ActivePanel::WorkingDirectory;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
                    self.active_panel = ActivePanel::CommitMessage;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
                    self.active_panel = ActivePanel::CommitHistory;
                }

                // Tab to cycle panels
                if ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift) {
                    self.active_panel = match self.active_panel {
                        ActivePanel::WorkingDirectory => ActivePanel::CommitMessage,
                        ActivePanel::CommitMessage => ActivePanel::CommitHistory,
                        ActivePanel::CommitHistory => ActivePanel::WorkingDirectory,
                    };
                }

                // 'r' to refresh
                if ui.input(|i| i.key_pressed(egui::Key::R)) {
                    self.handle_message(crabontree_app::AppMessage::RefreshRepo);
                }

                // '?' to show help
                if ui.input(|i| i.key_pressed(egui::Key::Questionmark)) {
                    self.show_shortcuts_help = !self.show_shortcuts_help;
                }

                // 'a' to stage all files
                if ui.input(|i| i.key_pressed(egui::Key::A)) {
                    self.handle_message(crabontree_app::AppMessage::StageAllRequested);
                }

                // 'u' to unstage all files
                if ui.input(|i| i.key_pressed(egui::Key::U)) {
                    self.handle_message(crabontree_app::AppMessage::UnstageAllRequested);
                }

                // Vim-style 'g' (press twice for top)
                if ui.input(|i| i.key_pressed(egui::Key::G)) {
                    if self.vim_g_pressed {
                        // Second 'g' press - go to top
                        match self.active_panel {
                            ActivePanel::CommitHistory => self.focused_commit_index = Some(0),
                            ActivePanel::WorkingDirectory => self.focused_file_index = Some(0),
                            _ => {}
                        }
                        self.vim_g_pressed = false;
                    } else if ui.input(|i| i.modifiers.shift) {
                        // Shift+G - go to bottom
                        match self.active_panel {
                            ActivePanel::CommitHistory => {
                                if !commits.is_empty() {
                                    self.focused_commit_index = Some(commits.len() - 1);
                                }
                            }
                            ActivePanel::WorkingDirectory => {
                                if !working_dir_files.is_empty() {
                                    self.focused_file_index = Some(working_dir_files.len() - 1);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // First 'g' press
                        self.vim_g_pressed = true;
                    }
                } else {
                    // Reset vim_g_pressed if any other key is pressed
                    if ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Key { .. }))) {
                        self.vim_g_pressed = false;
                    }
                }
            }

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

            // === Working Directory Panel (Resizable) ===
            ui.heading("🔨 Working Directory");
            ui.separator();

            // Working directory content
            let wd_rect = ui.available_rect_before_wrap();
            let wd_height = self.working_dir_height.min(wd_rect.height() - 50.0);

            ui.allocate_ui_with_layout(
                egui::vec2(wd_rect.width(), wd_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.render_working_directory(ui, &working_dir_files);
                },
            );

            // Horizontal resize handle for working directory
            let resize_id = ui.id().with("wd_resize");
            let resize_rect = egui::Rect::from_min_size(
                egui::pos2(wd_rect.left(), ui.cursor().top()),
                egui::vec2(wd_rect.width(), 8.0),
            );

            let resize_response = ui.interact(resize_rect, resize_id, egui::Sense::drag());
            if resize_response.dragged() {
                self.working_dir_height = (self.working_dir_height + resize_response.drag_delta().y)
                    .max(100.0)
                    .min(800.0);
            }

            // Visual feedback for resize handle
            let resize_color = if resize_response.hovered() || resize_response.dragged() {
                egui::Color32::from_rgb(100, 150, 200)
            } else {
                egui::Color32::from_rgb(60, 60, 60)
            };
            ui.painter().rect_filled(resize_rect, 0.0, resize_color);

            if resize_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }

            ui.add_space(10.0);

            // === Commit Panel (Resizable) ===
            ui.heading("💬 Commit");
            ui.separator();

            let commit_rect = ui.available_rect_before_wrap();
            let commit_height = self.commit_panel_height.min(commit_rect.height() - 50.0);

            ui.allocate_ui_with_layout(
                egui::vec2(commit_rect.width(), commit_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.render_commit_panel(
                        ui,
                        &working_dir_files,
                        &author_name,
                        &author_email
                    );
                },
            );

            // Horizontal resize handle for commit panel
            let commit_resize_id = ui.id().with("commit_resize");
            let commit_resize_rect = egui::Rect::from_min_size(
                egui::pos2(commit_rect.left(), ui.cursor().top()),
                egui::vec2(commit_rect.width(), 8.0),
            );

            let commit_resize_response = ui.interact(commit_resize_rect, commit_resize_id, egui::Sense::drag());
            if commit_resize_response.dragged() {
                self.commit_panel_height = (self.commit_panel_height + commit_resize_response.drag_delta().y)
                    .max(100.0)
                    .min(600.0);
            }

            // Visual feedback for resize handle
            let commit_resize_color = if commit_resize_response.hovered() || commit_resize_response.dragged() {
                egui::Color32::from_rgb(100, 150, 200)
            } else {
                egui::Color32::from_rgb(60, 60, 60)
            };
            ui.painter().rect_filled(commit_resize_rect, 0.0, commit_resize_color);

            if commit_resize_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }

            ui.add_space(10.0);

            // === Commit History Section (Resizable) ===
            if ui.button("📜 Load Commit History").clicked() {
                self.handle_message(crabontree_app::AppMessage::LoadCommitHistoryRequested);
            }

            ui.add_space(10.0);
            ui.separator();

            // Get available height for the commit section
            let available_height = ui.available_height();
            let available_width = ui.available_width();

            // Two-column layout for commits - with resizable separator
            ui.horizontal(|ui| {
                // Left panel: commit list (resizable width)
                let list_width = available_width * self.commit_list_width_ratio;

                ui.allocate_ui_with_layout(
                    egui::vec2(list_width, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_commit_history(ui, &commits, &selected_commit);
                    },
                );

                // Vertical resize handle for commit list width
                let list_resize_id = ui.id().with("commit_list_resize");
                let list_resize_rect = egui::Rect::from_min_size(
                    ui.cursor().left_top(),
                    egui::vec2(8.0, available_height),
                );

                let list_resize_response = ui.interact(list_resize_rect, list_resize_id, egui::Sense::drag());
                if list_resize_response.dragged() {
                    let delta_ratio = list_resize_response.drag_delta().x / available_width;
                    self.commit_list_width_ratio = (self.commit_list_width_ratio + delta_ratio)
                        .max(0.2)
                        .min(0.8);
                }

                // Visual feedback for resize handle
                let list_resize_color = if list_resize_response.hovered() || list_resize_response.dragged() {
                    egui::Color32::from_rgb(100, 150, 200)
                } else {
                    egui::Color32::from_rgb(60, 60, 60)
                };
                ui.painter().rect_filled(list_resize_rect, 0.0, list_resize_color);

                if list_resize_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                ui.add_space(4.0);

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
