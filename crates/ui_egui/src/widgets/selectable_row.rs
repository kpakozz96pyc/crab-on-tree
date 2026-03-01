/// Renders a selectable row with hover and click functionality that spans 100% width
/// Returns true if the row was clicked
pub fn selectable_row(
    ui: &mut egui::Ui,
    text: impl Into<egui::WidgetText>,
    is_selected: bool,
) -> bool {
    let text = text.into();
    let available_width = ui.available_width();

    // Calculate text size first with wrapping enabled
    let galley = text.into_galley(ui, Some(true), available_width - 8.0, egui::TextStyle::Body);
    let row_height = galley.size().y + 4.0; // Add some padding

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, row_height),
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        // Determine background color based on state
        let bg_color = if is_selected {
            ui.visuals().selection.bg_fill
        } else if response.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };

        // Paint background if needed.
        if bg_color != egui::Color32::TRANSPARENT {
            ui.painter()
                .rect_filled(rect, ui.visuals().widgets.inactive.rounding, bg_color);
        }

        // Paint text left-aligned
        let text_color = if is_selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let text_pos = rect.left_top() + egui::vec2(4.0, 2.0);
        ui.painter().galley(text_pos, galley, text_color);
    }

    response.clicked()
}
