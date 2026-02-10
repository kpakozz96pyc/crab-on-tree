/// File row widget for displaying files with status icons.
///
/// This widget displays a file path with a colored status indicator,
/// eliminating the duplication of status icon logic across multiple
/// sections (staged, unstaged, untracked, conflicted).

use crabontree_app::WorkingDirStatus;
use eframe::egui;
use std::path::Path;

/// A file row widget that displays a file with its status icon.
pub struct FileRow<'a> {
    /// The file path to display
    pub path: &'a Path,
    /// The working directory status of the file
    pub status: &'a WorkingDirStatus,
    /// Whether this file is currently selected
    pub is_selected: bool,
}

impl<'a> FileRow<'a> {
    /// Creates a new file row widget.
    pub fn new(path: &'a Path, status: &'a WorkingDirStatus, is_selected: bool) -> Self {
        Self {
            path,
            status,
            is_selected,
        }
    }

    /// Renders the file row and returns true if it was clicked.
    pub fn render(self, ui: &mut egui::Ui) -> bool {
        let (status_icon, status_color) = self.get_status_info();

        ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width());
            ui.colored_label(
                status_color,
                egui::RichText::new(status_icon).strong(),
            );
            ui.selectable_label(self.is_selected, self.path.display().to_string())
        })
        .inner
        .clicked()
    }

    /// Gets the status icon and color for the current file status.
    fn get_status_info(&self) -> (&'static str, egui::Color32) {
        match self.status {
            WorkingDirStatus::Modified => ("~", egui::Color32::from_rgb(200, 150, 0)),
            WorkingDirStatus::Deleted => ("-", egui::Color32::from_rgb(200, 0, 0)),
            WorkingDirStatus::Untracked => ("+", egui::Color32::from_rgb(0, 200, 0)),
            WorkingDirStatus::Renamed => ("R", egui::Color32::from_rgb(100, 150, 200)),
            WorkingDirStatus::Conflicted => ("!", egui::Color32::from_rgb(200, 0, 200)),
            WorkingDirStatus::TypeChanged => ("T", egui::Color32::from_rgb(150, 100, 200)),
        }
    }
}
