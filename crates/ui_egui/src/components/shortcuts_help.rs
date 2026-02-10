/// Keyboard shortcuts help dialog component.
///
/// Displays a window with all keyboard shortcuts organized by category.

use eframe::egui;

/// Renders the keyboard shortcuts help dialog.
///
/// The dialog visibility is controlled by the `open` parameter.
pub fn render(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Keyboard Shortcuts")
        .open(open)
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
