//! Theme utilities for applying color themes to the application.

use crabontree_ui_core::{Color, Theme};
use eframe::egui;

/// Extra theme colours that aren't expressible through egui's built-in [`Visuals`].
///
/// Populated every frame by [`apply_theme`] and stored in the egui context so any
/// widget can look them up without needing the [`Theme`] threaded through its call
/// chain. Use [`ThemeColors::get`] to retrieve them.
#[derive(Clone, Debug)]
pub struct ThemeColors {
    pub git_added: egui::Color32,
    pub git_untracked: egui::Color32,
    pub git_renamed: egui::Color32,
    pub git_conflicted: egui::Color32,
    pub git_type_changed: egui::Color32,
    pub overlay_bg: egui::Color32,
    pub overlay_fg: egui::Color32,
    pub hint_fg: egui::Color32,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            git_added: egui::Color32::from_rgb(60, 160, 60),
            git_untracked: egui::Color32::from_rgb(60, 160, 60),
            git_renamed: egui::Color32::from_rgb(80, 140, 200),
            git_conflicted: egui::Color32::from_rgb(180, 80, 180),
            git_type_changed: egui::Color32::from_rgb(130, 100, 180),
            overlay_bg: egui::Color32::from_black_alpha(128),
            overlay_fg: egui::Color32::WHITE,
            hint_fg: egui::Color32::from_gray(80),
        }
    }
}

impl ThemeColors {
    fn id() -> egui::Id {
        egui::Id::new("cot_theme_colors")
    }

    /// Retrieve the current frame's theme colours from the egui context.
    pub fn get(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp(Self::id()).unwrap_or_default())
    }

    fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(Self::id(), self.clone()));
    }
}

/// Applies a theme to the egui context.
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let bg = theme.bg_primary;
    let is_dark = (bg.r + bg.g + bg.b) / 3.0 < 0.5;

    // Start from the matching base so rounding, shadows, etc. are sensible.
    let mut visuals = if is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    let bg_primary = color_to_egui(theme.bg_primary);
    let bg_secondary = color_to_egui(theme.bg_secondary);
    let bg_tertiary = color_to_egui(theme.bg_tertiary);
    let fg_primary = color_to_egui(theme.fg_primary);
    let accent = color_to_egui(theme.accent_primary);
    let accent2 = color_to_egui(theme.accent_secondary);
    let pane_border = color_to_egui(theme.pane_border);

    // Hover background: bg_tertiary with a slight accent tint.
    let bg_hover = blend(bg_tertiary, accent, 0.08);

    // ── Backgrounds ─────────────────────────────────────────────────────────
    visuals.panel_fill = bg_primary; // TopBottomPanel, SidePanel, CentralPanel
    visuals.window_fill = bg_secondary; // egui_dock tab body, popups, windows
    visuals.extreme_bg_color = bg_secondary; // TextEdit, ScrollArea inner bg
    visuals.faint_bg_color = bg_tertiary; // subtle row highlights
    visuals.code_bg_color = bg_secondary;

    let rounding = if theme.rounded_corners {
        egui::Rounding::same(6.0)
    } else {
        egui::Rounding::ZERO
    };

    visuals.window_rounding = rounding;
    visuals.menu_rounding = rounding;

    // ── Widget visuals ───────────────────────────────────────────────────────
    // noninteractive: labels, separators, static frames
    visuals.widgets.noninteractive.bg_fill = bg_primary;
    visuals.widgets.noninteractive.weak_bg_fill = bg_secondary;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, pane_border);
    visuals.widgets.noninteractive.rounding = rounding;
    // fg_stroke is used both as the text colour and as the fallback inside text_color(),
    // so use fg_primary here (not fg_secondary) to keep static labels readable.
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_primary);

    // inactive: buttons/checkboxes at rest
    visuals.widgets.inactive.bg_fill = bg_tertiary;
    visuals.widgets.inactive.weak_bg_fill = bg_secondary;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, bg_tertiary);
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg_primary);

    // hovered
    visuals.widgets.hovered.bg_fill = bg_hover;
    visuals.widgets.hovered.weak_bg_fill = bg_tertiary;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, accent);
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg_primary);

    // active (pressed)
    visuals.widgets.active.bg_fill = accent2;
    visuals.widgets.active.weak_bg_fill = accent2;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);
    visuals.widgets.active.rounding = rounding;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg_primary);

    // open (open combo boxes, collapsing headers)
    visuals.widgets.open.bg_fill = bg_tertiary;
    visuals.widgets.open.weak_bg_fill = bg_secondary;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent);
    visuals.widgets.open.rounding = rounding;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.5, fg_primary);

    // ── Selection ────────────────────────────────────────────────────────────
    visuals.selection.bg_fill = accent2;
    visuals.selection.stroke = egui::Stroke::new(1.0, color_to_egui(theme.selection_fg));

    // ── Window chrome ────────────────────────────────────────────────────────
    visuals.window_stroke = egui::Stroke::new(1.0, pane_border);

    // ── Text ─────────────────────────────────────────────────────────────────
    visuals.override_text_color = Some(fg_primary);

    // ── Semantic ─────────────────────────────────────────────────────────────
    visuals.hyperlink_color = accent;
    visuals.error_fg_color = color_to_egui(theme.error);
    visuals.warn_fg_color = color_to_egui(theme.warning);

    ctx.set_visuals(visuals);

    // Store extra colours in context data for widgets that need them.
    ThemeColors {
        git_added: color_to_egui(theme.git_added),
        git_untracked: color_to_egui(theme.git_untracked),
        git_renamed: color_to_egui(theme.git_renamed),
        git_conflicted: color_to_egui(theme.git_conflicted),
        git_type_changed: color_to_egui(theme.git_type_changed),
        overlay_bg: color_to_egui(theme.overlay_bg),
        overlay_fg: color_to_egui(theme.overlay_fg),
        hint_fg: color_to_egui(theme.hint_fg),
    }
    .store(ctx);
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

/// Linear blend between two colours. `t = 0` → base, `t = 1` → other.
fn blend(base: egui::Color32, other: egui::Color32, t: f32) -> egui::Color32 {
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    egui::Color32::from_rgb(
        lerp(base.r(), other.r()),
        lerp(base.g(), other.g()),
        lerp(base.b(), other.b()),
    )
}
