use crate::runtime::CrabOnTreeApp;
use crate::utils::keyboard::DiffScrollAction;
use crate::utils::theme::ThemeColors;
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
        let visible_panes = self.get_visible_panes();
        let mut visible_indices: Vec<usize> =
            visible_panes.iter().map(|p| pane_to_index(*p)).collect();
        visible_indices.sort_unstable();
        visible_indices.dedup();

        if !visible_indices.contains(&self.active_pane) {
            if let Some(first_visible) = visible_indices.first().copied() {
                self.active_pane = first_visible;
                self.focus_active_pane_tab();
            }
        }

        let prev_active_pane = self.active_pane;
        let (action, new_pane) = keyboard::handle_shortcuts(
            ui,
            self.active_pane,
            self.state.current_repo.as_ref(),
            &visible_indices,
        );
        self.active_pane = new_pane;
        if self.active_pane != prev_active_pane {
            self.focus_active_pane_tab();
        }

        let mut diff_scroll: Option<DiffScrollAction> = None;

        match action {
            keyboard::KeyboardAction::ToggleHelp => {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            }
            keyboard::KeyboardAction::ScrollDiff(command) => {
                diff_scroll = Some(command);
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
            self.handle_message(crabontree_app::AppMessage::CommitSelected(
                crabontree_app::WORKING_DIR_HASH.to_string(),
            ));
        }

        let repo_data = if let Some(repo) = &self.state.current_repo {
            repo
        } else {
            return;
        };

        let mut messages = Vec::new();
        let mut pane_focus_request: Option<usize> = None;
        let mut viewer = PaneViewer {
            repo_data,
            messages: &mut messages,
            loading: self.state.loading,
            committing: self.state.committing,
            active_pane: self.active_pane,
            diff_scroll,
            pane_focus_request: &mut pane_focus_request,
        };

        DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);

        if let Some(index) = pane_focus_request {
            if index != self.active_pane {
                self.active_pane = index;
                self.focus_active_pane_tab();
            }
        }

        for msg in messages {
            self.handle_message(msg);
        }
    }

    fn focus_active_pane_tab(&mut self) {
        let target_pane = pane_from_index(self.active_pane);
        if let Some((surface, node, tab_index)) = self.dock_state.find_tab(&target_pane) {
            self.dock_state
                .set_focused_node_and_surface((surface, node));
            self.dock_state.set_active_tab((surface, node, tab_index));
        }
    }
}

/// TabViewer implementation for rendering panes in the dock.
struct PaneViewer<'a> {
    repo_data: &'a crabontree_app::RepoState,
    messages: &'a mut Vec<crabontree_app::AppMessage>,
    loading: bool,
    committing: bool,
    active_pane: usize,
    diff_scroll: Option<DiffScrollAction>,
    pane_focus_request: &'a mut Option<usize>,
}

impl<'a> egui_dock::TabViewer for PaneViewer<'a> {
    type Tab = panes::Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let repo = self.repo_data;
        let scroll_id = format!("{:?}_dock_scroll", tab);
        let is_active = pane_to_index(*tab) == self.active_pane;
        let tc = ThemeColors::get(ui.ctx());
        let stroke = if is_active {
            egui::Stroke::new(tc.focused_pane_border_width, tc.focused_pane_border_color)
        } else {
            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
        };

        let scroll_config = match tab {
            panes::Pane::DiffViewer => {
                panes::scrollable_pane::ScrollablePaneConfig::new_both_scroll(&scroll_id)
            }
            _ => panes::scrollable_pane::ScrollablePaneConfig::new(&scroll_id),
        };

        let frame_response = egui::Frame::none()
            .stroke(stroke)
            .inner_margin(egui::Margin::same(2.0))
            .show(ui, |ui| {
                panes::scrollable_pane::render(ui, &scroll_config, |ui| match tab {
                    panes::Pane::CommitHistory => {
                        let action = panes::commit_history::render(
                            ui,
                            &repo.commits,
                            repo.selected_commit.as_ref(),
                            !repo.working_dir_files.is_empty(),
                            is_active,
                        );
                        if let Some(msg) = panes::commit_history::action_to_message(action) {
                            self.messages.push(msg);
                        }
                    }
                    panes::Pane::ChangedFiles => {
                        if let Some(files) = &repo.changed_files {
                            let action =
                                panes::changed_files::render(ui, files, self.committing, is_active);
                            if let Some(msg) = panes::changed_files::action_to_message(action) {
                                self.messages.push(msg);
                            }
                        } else {
                            ui.label("Loading changed files...");
                        }
                    }
                    panes::Pane::DiffViewer => {
                        panes::diff_viewer::render(ui, &repo.file_view);
                        if is_active {
                            apply_diff_scroll(ui, self.diff_scroll);
                        }
                    }
                    panes::Pane::Branches => {
                        if let Some(branch_tree) = &repo.branch_tree {
                            let action =
                                panes::branches::render(ui, branch_tree, self.loading, is_active);
                            if let Some(msg) = panes::branches::action_to_message(action) {
                                self.messages.push(msg);
                            }
                        } else {
                            ui.label("Loading branches...");
                        }
                    }
                });
            });

        let pane_index = pane_to_index(*tab);
        if ui.ctx().input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                if frame_response.response.rect.contains(pos) {
                    *self.pane_focus_request = Some(pane_index);
                }
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

fn pane_to_index(pane: panes::Pane) -> usize {
    match pane {
        panes::Pane::CommitHistory => 0,
        panes::Pane::Branches => 1,
        panes::Pane::ChangedFiles => 2,
        panes::Pane::DiffViewer => 3,
    }
}

fn pane_from_index(index: usize) -> panes::Pane {
    match index {
        0 => panes::Pane::CommitHistory,
        1 => panes::Pane::Branches,
        2 => panes::Pane::ChangedFiles,
        _ => panes::Pane::DiffViewer,
    }
}

fn apply_diff_scroll(ui: &egui::Ui, action: Option<DiffScrollAction>) {
    let Some(action) = action else {
        return;
    };

    let line_height = 18.0;
    let page_height = ui.clip_rect().height() * 0.9;
    match action {
        DiffScrollAction::Line(lines) => {
            ui.scroll_with_delta(egui::vec2(0.0, lines as f32 * line_height))
        }
        DiffScrollAction::Page(pages) => {
            ui.scroll_with_delta(egui::vec2(0.0, pages as f32 * page_height))
        }
        DiffScrollAction::Home => ui.scroll_to_rect(ui.min_rect(), Some(egui::Align::TOP)),
        DiffScrollAction::End => ui.scroll_to_rect(ui.min_rect(), Some(egui::Align::BOTTOM)),
    }
}
