use crabontree_app::CommitInfo;
use crabontree_ui_core::{format_absolute_time, format_relative_time};
use eframe::egui;

pub fn render(ui: &mut egui::Ui, info: &CommitInfo, commit_message: &str) {
    // Metadata grid
    egui::Grid::new("commit_info_grid")
        .num_columns(2)
        .spacing(egui::vec2(8.0, 2.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Author:").strong());
            ui.label(format!("{} <{}>", info.author_name, info.author_email));
            ui.end_row();

            ui.label(egui::RichText::new("Date:").strong());
            let relative = format_relative_time(info.author_date);
            let absolute = format_absolute_time(info.author_date);
            ui.label(format!("{} ({})", relative, absolute));
            ui.end_row();

            ui.label(egui::RichText::new("Commit hash:").strong());
            ui.label(egui::RichText::new(&info.hash).monospace());
            ui.end_row();

            if !info.parent_hashes.is_empty() {
                ui.label(egui::RichText::new("Parent(s):").strong());
                ui.label(egui::RichText::new(info.parent_hashes.join(" ")).monospace());
                ui.end_row();
            }
        });
    ui.add_space(15.0);

    // Commit message
    if !commit_message.trim().is_empty() {
        ui.add_space(5.0);
        ui.label(commit_message.trim());
        ui.add_space(5.0);
    }

    ui.separator();

    // Branch/tag containment info
    if info.branches.is_empty() {
        ui.label(egui::RichText::new("Contained in no branch").weak());
    } else {
        ui.label(format!("Contained in branches: {}", info.branches.join(", ")));
    }
    if info.tags.is_empty() {
        ui.label(egui::RichText::new("Contained in no tag").weak());
    } else {
        ui.label(format!("Contained in tags: {}", info.tags.join(", ")));
    }
}
