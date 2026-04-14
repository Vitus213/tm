use eframe::egui;

/// Renders a horizontal tab bar with selectable labels.
///
/// # Parameters
/// - `labels`: The labels for each tab
/// - `selected_index`: The index of the currently selected tab (must be < labels.len())
///
/// # Returns
/// The index of the newly selected tab if a different one was clicked, otherwise None.
pub fn tabbar(ui: &mut egui::Ui, labels: &[&str], selected_index: usize) -> Option<usize> {
    debug_assert!(
        selected_index < labels.len(),
        "selected_index ({}) is out of bounds (labels.len() = {})",
        selected_index,
        labels.len()
    );

    if selected_index >= labels.len() {
        return None;
    }

    let mut new_index = None;

    ui.horizontal(|ui| {
        for (i, label) in labels.iter().enumerate() {
            let is_selected = i == selected_index;
            let response = ui.selectable_label(is_selected, *label);

            if response.clicked() && !is_selected {
                new_index = Some(i);
            }

            if is_selected {
                let rect = response.rect;
                let underline_rect =
                    egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - 2.0), rect.max);
                ui.painter()
                    .rect_filled(underline_rect, 0.0, ui.visuals().selection.stroke.color);
            }

            if i < labels.len() - 1 {
                ui.add_space(16.0);
            }
        }
    });

    new_index
}
