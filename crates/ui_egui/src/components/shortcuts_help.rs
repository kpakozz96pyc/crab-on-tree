//! Keyboard shortcuts help dialog component.
//!
//! Displays a window with all keyboard shortcuts organized by category.

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
            ui.label("↑ / ↓       Previous / next item");
            ui.label("Home        Scroll diff to top");
            ui.label("End         Scroll diff to bottom");
            ui.label("Page Up/Dn  Scroll diff by page");

            ui.add_space(10.0);
            ui.heading("Panels");
            ui.separator();
            ui.label("← / →       Switch active panel");
            ui.label("Tab         Cycle panels forward");
            ui.label("Shift+Tab   Cycle panels backward");
            ui.label("1 / 2 / 3 / 4   Focus panel by number");

            ui.add_space(10.0);
            ui.heading("Actions");
            ui.separator();
            ui.label("r           Refresh repository");

            ui.add_space(10.0);
            ui.heading("Help");
            ui.separator();
            ui.label("?           Show/hide this help");
        });
}
