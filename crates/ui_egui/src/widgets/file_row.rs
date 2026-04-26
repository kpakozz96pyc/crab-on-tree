//! File row widget for displaying files with status icons.
//!
//! This widget displays a file path with a colored status indicator,
//! eliminating the duplication of status icon logic across multiple
//! sections (staged, unstaged, untracked, conflicted).

use crate::utils::theme::ThemeColors;
use crabontree_app::WorkingDirStatus;
use eframe::egui;
use std::path::Path;

/// Interaction type for file row clicks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRowInteraction {
    None,
    SingleClick { ctrl: bool, shift: bool },
    DoubleClick,
}

/// A file row widget that displays a file with its status icon.
pub struct FileRow<'a> {
    /// The file path to display
    pub path: &'a Path,
    /// The working directory status of the file
    pub status: &'a WorkingDirStatus,
    /// Whether this file is currently selected
    pub is_selected: bool,
    /// Whether this row currently has keyboard/mouse focus within the active pane
    pub is_focused: bool,
}

impl<'a> FileRow<'a> {
    /// Creates a new file row widget.
    pub fn new(
        path: &'a Path,
        status: &'a WorkingDirStatus,
        is_selected: bool,
        is_focused: bool,
    ) -> Self {
        Self {
            path,
            status,
            is_selected,
            is_focused,
        }
    }

    /// Renders the file row and returns the interaction type and the row's egui Response.
    pub fn render(self, ui: &mut egui::Ui) -> (FileRowInteraction, egui::Response) {
        let (status_icon, status_color) = self.get_status_info(ui);
        let tc = ThemeColors::get(ui.ctx());

        let available_width = ui.available_width();
        let path_text: egui::WidgetText = self.path.display().to_string().into();
        let path_galley =
            path_text.into_galley(ui, Some(false), available_width - 8.0, egui::TextStyle::Body);
        let row_height = path_galley.size().y + 4.0;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_width, row_height),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let bg = if self.is_focused {
                tc.focused_row_bg
            } else if self.is_selected {
                tc.selected_row_bg
            } else if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };

            if bg != egui::Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(rect, ui.visuals().widgets.inactive.rounding, bg);
            }

            let text_color = if self.is_selected {
                ui.visuals().selection.stroke.color
            } else {
                ui.visuals().text_color()
            };

            // Paint status icon
            let icon_widget: egui::WidgetText =
                egui::RichText::new(status_icon).strong().color(status_color).into();
            let icon_galley =
                icon_widget.into_galley(ui, Some(false), f32::INFINITY, egui::TextStyle::Body);
            let icon_width = icon_galley.size().x;
            ui.painter()
                .galley(rect.left_top() + egui::vec2(4.0, 2.0), icon_galley, status_color);

            // Paint file path
            ui.painter().galley(
                rect.left_top() + egui::vec2(4.0 + icon_width + 6.0, 2.0),
                path_galley,
                text_color,
            );
        }

        let interaction = if response.double_clicked() {
            FileRowInteraction::DoubleClick
        } else if response.clicked() {
            let ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
            let shift = ui.input(|i| i.modifiers.shift);
            FileRowInteraction::SingleClick { ctrl, shift }
        } else {
            FileRowInteraction::None
        };

        (interaction, response)
    }

    /// Gets the status icon and color for the current file status.
    fn get_status_info(&self, ui: &egui::Ui) -> (&'static str, egui::Color32) {
        let tc = ThemeColors::get(ui.ctx());
        match self.status {
            WorkingDirStatus::Modified => ("~", ui.visuals().warn_fg_color),
            WorkingDirStatus::Deleted => ("-", ui.visuals().error_fg_color),
            WorkingDirStatus::Untracked => ("+", tc.git_untracked),
            WorkingDirStatus::Renamed => ("R", tc.git_renamed),
            WorkingDirStatus::Conflicted => ("!", tc.git_conflicted),
            WorkingDirStatus::TypeChanged => ("T", tc.git_type_changed),
        }
    }

}
