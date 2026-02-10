/// Error panel component for displaying error messages.
///
/// Shows a dismissible error banner at the top of the application
/// when an error occurs.

use crabontree_app::AppMessage;
use crabontree_ui_core::Color;
use eframe::egui;

/// Action to be taken after rendering the error panel.
pub enum ErrorPanelAction {
    None,
    ClearError,
}

/// Renders the error panel if an error is present.
///
/// Returns an action that the caller should handle.
pub fn render(
    ctx: &egui::Context,
    error: Option<&String>,
    error_color: Color,
) -> ErrorPanelAction {
    let mut action = ErrorPanelAction::None;

    if let Some(error_msg) = error {
        egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let color = color_to_egui(error_color);
                ui.colored_label(color, format!("❌ {}", error_msg));

                if ui.button("✖").clicked() {
                    action = ErrorPanelAction::ClearError;
                }
            });
        });
    }

    action
}

/// Converts a Color to egui::Color32.
fn color_to_egui(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

/// Converts an ErrorPanelAction to an AppMessage.
pub fn action_to_message(action: ErrorPanelAction) -> Option<AppMessage> {
    match action {
        ErrorPanelAction::None => None,
        ErrorPanelAction::ClearError => Some(AppMessage::ClearError),
    }
}
