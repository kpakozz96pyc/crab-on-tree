//! File content view widget for displaying files with line numbers.
//!
//! This widget handles the display of file content with line numbers
//! in a monospace font.

use eframe::egui;
use std::path::Path;

/// A file content view widget that displays file content with line numbers.
pub struct FileContentView<'a> {
    /// The file path being displayed
    pub path: &'a Path,
    /// The file content to display
    pub content: &'a str,
}

impl<'a> FileContentView<'a> {
    /// Creates a new file content view widget.
    pub fn new(path: &'a Path, content: &'a str) -> Self {
        Self { path, content }
    }

    /// Renders the file content view.
    pub fn render(self, ui: &mut egui::Ui) {
        ui.heading(self.path.display().to_string());
        ui.separator();
        ui.add_space(5.0);

        if self.content.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("Can't display content").weak());
            });
            return;
        }

        for (i, line) in self.content.lines().enumerate() {
            ui.push_id(format!("content_line_{}", i), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{:4} ", i + 1))
                            .monospace()
                            .weak(),
                    );
                    ui.label(egui::RichText::new(line).monospace());
                });
            });
        }
    }
}
