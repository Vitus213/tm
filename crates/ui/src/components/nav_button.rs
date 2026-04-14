use eframe::egui;

/// Renders a vertical navigation button with an icon and label.
///
/// The button has a fixed width of 80px, centers its content vertically
/// and horizontally, and shows a left indicator strip when selected.
pub fn nav_button(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> bool {
    let icon_size = 20.0;
    let label_size = 12.0;
    let gap = 4.0;

    let content_height = icon_size + gap + label_size;
    let button_height = content_height + 16.0; // 8px top + 8px bottom padding
    let desired_size = egui::vec2(80.0, button_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let text_color = visuals.text_color();

        let bg_fill = if selected {
            ui.visuals().selection.bg_fill.gamma_multiply(0.3)
        } else if response.hovered() {
            ui.visuals().widgets.hovered.bg_fill.gamma_multiply(0.2)
        } else {
            egui::Color32::TRANSPARENT
        };

        if bg_fill != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 0.0, bg_fill);
        }

        if selected {
            let strip_rect =
                egui::Rect::from_min_size(rect.left_top(), egui::vec2(2.0, rect.height()));
            ui.painter()
                .rect_filled(strip_rect, 0.0, ui.visuals().selection.bg_fill);
        }

        // Use egui's layout system for text positioning
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child_ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(icon).size(icon_size).color(text_color));
            ui.add_space(gap);
            ui.label(
                egui::RichText::new(label)
                    .size(label_size)
                    .color(text_color),
            );
        });
    }

    response.clicked()
}
