use crate::widgets::{DiffView, FileContentView};
use crabontree_app::FileViewState;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, state: &FileViewState) {
    match state {
        FileViewState::None => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("Select a file or commit to view");
            });
        }
        FileViewState::Content { path, content, .. } => {
            FileContentView::new(path, content).render(ui);
        }
        FileViewState::Diff { path, hunks, .. } => {
            DiffView::new(path, hunks).render(ui);
        }
        FileViewState::MultipleDiffs { files, .. } => {
            for (path, hunks) in files {
                ui.heading(path.display().to_string());
                ui.add_space(5.0);
                DiffView::new(path, hunks).render(ui);
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);
            }
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
