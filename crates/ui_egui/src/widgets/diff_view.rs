//! Diff view widget for rendering diff hunks.
//!
//! This widget handles the display of git diffs with proper syntax
//! highlighting for additions, deletions, and context lines.

use crabontree_app::{DiffHunk, DiffLineType};
use eframe::egui;
use std::path::Path;

/// A diff view widget that displays diff hunks with line numbers and colors.
pub struct DiffView<'a> {
    /// The file path being diffed
    pub path: &'a Path,
    /// The diff hunks to display
    pub hunks: &'a [DiffHunk],
}

impl<'a> DiffView<'a> {
    /// Creates a new diff view widget.
    pub fn new(path: &'a Path, hunks: &'a [DiffHunk]) -> Self {
        Self { path, hunks }
    }

    /// Renders the diff view.
    pub fn render(self, ui: &mut egui::Ui) {
        ui.heading(format!("Diff: {}", self.path.display()));
        ui.separator();
        ui.add_space(5.0);

        if self.hunks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("Can't display diff").weak());
            });
            return;
        }

        for (hunk_idx, hunk) in self.hunks.iter().enumerate() {
            ui.push_id(format!("hunk_{}", hunk_idx), |ui| {
                self.render_hunk(ui, hunk);
                ui.add_space(10.0);
            });
        }
    }

    /// Renders a single diff hunk.
    fn render_hunk(&self, ui: &mut egui::Ui, hunk: &DiffHunk) {
        // Render hunk header
        ui.label(
            egui::RichText::new(format!(
                "@@ -{},{} +{},{} @@",
                hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
            ))
            .monospace()
            .weak(),
        );

        // Render hunk lines
        for (line_idx, line) in hunk.lines.iter().enumerate() {
            ui.push_id(format!("line_{}", line_idx), |ui| {
                let (prefix, color) = match line.line_type {
                    DiffLineType::Addition => ("+", egui::Color32::from_rgb(0, 200, 0)),
                    DiffLineType::Deletion => ("-", egui::Color32::from_rgb(200, 0, 0)),
                    DiffLineType::Context => (" ", egui::Color32::from_rgb(200, 200, 200)),
                };

                ui.horizontal(|ui| {
                    let line_num = line
                        .old_line_number
                        .or(line.new_line_number)
                        .map(|n| format!("{:4}", n))
                        .unwrap_or_else(|| "    ".to_string());
                    ui.label(egui::RichText::new(line_num).monospace().weak());
                    ui.colored_label(
                        color,
                        egui::RichText::new(format!("{}{}", prefix, line.content.trim_end()))
                            .monospace(),
                    );
                });
            });
        }
    }
}
