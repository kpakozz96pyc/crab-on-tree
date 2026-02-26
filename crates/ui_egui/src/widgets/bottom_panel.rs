use eframe::egui;

const SEPARATOR_HEIGHT: f32 = 8.0;

/// Splits a pane into a scrollable list above and a fixed-height panel pinned to the bottom.
///
/// `render_list` receives the list area ui.
/// `render_panel` receives the panel child ui and the panel rect (useful for overlays).
pub fn render_with_bottom_panel<L, P>(
    ui: &mut egui::Ui,
    panel_height: f32,
    render_list: L,
    render_panel: P,
) where
    L: FnOnce(&mut egui::Ui),
    P: FnOnce(&mut egui::Ui, egui::Rect),
{
    let pane_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(pane_rect, egui::Sense::hover());

    let clamped_height = panel_height.min(pane_rect.height());
    let panel_top = pane_rect.bottom() - clamped_height;
    let list_bottom = (panel_top - SEPARATOR_HEIGHT).max(pane_rect.top());

    let list_rect =
        egui::Rect::from_min_max(pane_rect.min, egui::pos2(pane_rect.right(), list_bottom));
    let panel_rect = egui::Rect::from_min_max(
        egui::pos2(pane_rect.left(), panel_top),
        egui::pos2(pane_rect.right(), pane_rect.bottom()),
    );

    ui.allocate_ui_at_rect(list_rect, |ui| render_list(ui));

    if list_bottom < panel_top {
        let y = list_bottom + (SEPARATOR_HEIGHT * 0.5);
        ui.painter().line_segment(
            [
                egui::pos2(pane_rect.left(), y),
                egui::pos2(pane_rect.right(), y),
            ],
            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        );
    }

    let mut panel_ui = ui.child_ui(panel_rect, egui::Layout::top_down(egui::Align::Min));
    egui::ScrollArea::vertical()
        .id_source("bottom_panel_scroll")
        .show(&mut panel_ui, |ui| render_panel(ui, panel_rect));
}
