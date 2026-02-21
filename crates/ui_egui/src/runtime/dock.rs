use crate::runtime::CrabOnTreeApp;
use crate::{panes, utils::keyboard};
use eframe::egui;
use egui_dock::{DockArea, NodeIndex};

impl CrabOnTreeApp {
    pub(crate) fn render_repository_view(&mut self, ui: &mut egui::Ui) {
        self.render_dock_layout(ui);
    }

    pub(crate) fn get_visible_panes(&self) -> Vec<panes::Pane> {
        let mut visible = Vec::new();
        self.dock_state.iter_all_tabs().for_each(|(_, tab)| {
            if !visible.contains(tab) {
                visible.push(*tab);
            }
        });
        visible
    }

    pub(crate) fn toggle_pane(&mut self, pane: panes::Pane) {
        if let Some((surface, node, tab_index)) = self.dock_state.find_tab(&pane) {
            if surface == egui_dock::SurfaceIndex::main() {
                self.saved_dock_states.insert(pane, self.dock_state.clone());
                self.dock_state
                    .main_surface_mut()
                    .remove_tab((node, tab_index));
            }
        } else if let Some(saved_state) = self.saved_dock_states.remove(&pane) {
            self.dock_state = saved_state;
        } else {
            self.dock_state
                .main_surface_mut()
                .set_focused_node(NodeIndex::root());
            self.dock_state
                .main_surface_mut()
                .push_to_focused_leaf(pane);
        }
    }

    fn render_dock_layout(&mut self, ui: &mut egui::Ui) {
        let (action, new_pane) = keyboard::handle_shortcuts(ui, self.active_pane);
        self.active_pane = new_pane;

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

        let repo_data = if let Some(repo) = &self.state.current_repo {
            repo
        } else {
            return;
        };

        let mut messages = Vec::new();
        let mut viewer = PaneViewer {
            repo_data,
            messages: &mut messages,
            loading: self.state.loading,
            committing: self.state.committing,
        };

        DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);

        for msg in messages {
            self.handle_message(msg);
        }
    }
}

/// TabViewer implementation for rendering panes in the dock.
struct PaneViewer<'a> {
    repo_data: &'a crabontree_app::RepoState,
    messages: &'a mut Vec<crabontree_app::AppMessage>,
    loading: bool,
    committing: bool,
}

impl<'a> egui_dock::TabViewer for PaneViewer<'a> {
    type Tab = panes::Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let repo = self.repo_data;
        let scroll_id = format!("{:?}_dock_scroll", tab);

        let scroll_config = match tab {
            panes::Pane::DiffViewer => {
                panes::scrollable_pane::ScrollablePaneConfig::new_both_scroll(&scroll_id)
            }
            _ => panes::scrollable_pane::ScrollablePaneConfig::new(&scroll_id),
        };

        panes::scrollable_pane::render(ui, &scroll_config, |ui| match tab {
            panes::Pane::CommitHistory => {
                let action = panes::commit_history::render(
                    ui,
                    &repo.commits,
                    repo.selected_commit.as_ref(),
                    !repo.working_dir_files.is_empty(),
                );
                if let Some(msg) = panes::commit_history::action_to_message(action) {
                    self.messages.push(msg);
                }
            }
            panes::Pane::ChangedFiles => {
                if let Some(files) = &repo.changed_files {
                    let action = panes::changed_files::render(ui, files, self.committing);
                    if let Some(msg) = panes::changed_files::action_to_message(action) {
                        self.messages.push(msg);
                    }
                } else {
                    ui.label("Loading changed files...");
                }
            }
            panes::Pane::DiffViewer => {
                panes::diff_viewer::render(ui, &repo.file_view);
            }
            panes::Pane::Branches => {
                if let Some(branch_tree) = &repo.branch_tree {
                    let action = panes::branches::render(ui, branch_tree, self.loading);
                    if let Some(msg) = panes::branches::action_to_message(action) {
                        self.messages.push(msg);
                    }
                } else {
                    ui.label("Loading branches...");
                }
            }
        });
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}
