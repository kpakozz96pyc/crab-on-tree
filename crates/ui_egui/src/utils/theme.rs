/// Theme utilities for applying color themes to the application.

use crabontree_ui_core::{Color, Theme};
use eframe::egui;

/// Applies a theme to the egui context.
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = color_to_egui(theme.bg_primary);
    visuals.extreme_bg_color = color_to_egui(theme.bg_secondary);
    visuals.faint_bg_color = color_to_egui(theme.bg_tertiary);

    visuals.override_text_color = Some(color_to_egui(theme.fg_primary));
    visuals.hyperlink_color = color_to_egui(theme.accent_primary);
    visuals.error_fg_color = color_to_egui(theme.error);
    visuals.warn_fg_color = color_to_egui(theme.warning);

    ctx.set_visuals(visuals);
}

/// Converts a Color to egui::Color32.
pub fn color_to_egui(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}
