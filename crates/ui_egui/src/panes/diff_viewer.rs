/// Diff viewer pane for displaying file content and diffs.
///
/// This pane handles displaying different types of file content:
/// - Text files with line numbers
/// - Diffs with syntax highlighting
/// - Binary file indicators
/// - Empty state when no file is selected

use crate::utils::scroll_config;
use crate::widgets::{DiffView, FileContentView};
use crabontree_app::FileViewState;
use eframe::egui;

/// Renders the diff viewer pane.
///
/// This pane shows different content based on the FileViewState:
/// - None: Shows a prompt to select a file
/// - Content: Shows file content with line numbers
/// - Diff: Shows diff hunks with syntax highlighting
/// - Binary: Shows a message for binary files
pub fn render(ui: &mut egui::Ui, state: &FileViewState) {
    match state {
        FileViewState::None => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("Select a file or commit to view");
            });
        }
        FileViewState::Content { path, content, .. } => {
            // Show file content with line numbers
            scroll_config::both_scroll()
                .id_source("file_content_scroll")
                .show(ui, |ui| {
                    scroll_config::set_full_width(ui);
                    FileContentView::new(path, content).render(ui);
                });
        }
        FileViewState::Diff { path, hunks, .. } => {
            // Render diff hunks
            scroll_config::both_scroll()
                .id_source("file_diff_scroll")
                .show(ui, |ui| {
                    scroll_config::set_full_width(ui);
                    DiffView::new(path, hunks).render(ui);
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
