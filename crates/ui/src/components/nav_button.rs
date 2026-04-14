use eframe::egui;

pub fn nav_button(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> bool {
    // Width fixed at 80px, height is natural based on content
    let desired_size = egui::vec2(80.0, 48.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

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

        let icon_pos = egui::pos2(rect.center().x, rect.min.y + 16.0);
        ui.painter().text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(20.0),
            visuals.text_color(),
        );

        let label_pos = egui::pos2(rect.center().x, rect.min.y + 38.0);
        ui.painter().text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            visuals.text_color(),
        );
    }

    response.clicked()
}
