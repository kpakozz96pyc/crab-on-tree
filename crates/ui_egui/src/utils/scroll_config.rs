//! Scroll area configuration utilities for consistent scroll behavior across panes.
//!
//! All scroll areas in the application should use these utilities to ensure:
//! - Content takes full width (scrollbar at pane edge)
//! - No auto-shrinking to content size

use eframe::egui;

/// Creates a vertical scroll area with standard configuration.
///
/// Returns a ScrollArea that:
/// - Scrolls vertically only
/// - Does not auto-shrink (maintains full pane width)
pub fn vertical_scroll() -> egui::ScrollArea {
    egui::ScrollArea::vertical().auto_shrink([false, false])
}

/// Creates a bidirectional scroll area with standard configuration.
///
/// Returns a ScrollArea that:
/// - Scrolls both horizontally and vertically
/// - Does not auto-shrink (maintains full pane width)
pub fn both_scroll() -> egui::ScrollArea {
    egui::ScrollArea::both().auto_shrink([false, false])
}

/// Sets the UI to take full available width.
///
/// Should be called at the start of scroll area content to ensure
/// the content spans the full width of the pane, positioning the
/// scrollbar at the edge.
///
/// # Example
/// ```ignore
/// scroll_config::vertical_scroll()
///     .show(ui, |ui| {
///         scroll_config::set_full_width(ui);
///         // ... render content
///     });
/// ```
pub fn set_full_width(ui: &mut egui::Ui) {
    ui.set_min_width(ui.available_width());
}
