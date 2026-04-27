use crate::design::*;
use eframe::egui;

pub fn nav_button(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> bool {
    let desired_size = egui::vec2(NAV_WIDTH, 64.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg_color = if selected {
            ACCENT_DIM
        } else if response.hovered() {
            BG_ELEVATED
        } else {
            egui::Color32::TRANSPARENT
        };

        if bg_color != egui::Color32::TRANSPARENT {
            let bg_rect = rect.shrink2(egui::vec2(GAP_XS, GAP_XS));
            ui.painter().rect_filled(bg_rect, RADIUS_SM, bg_color);
        }

        if selected {
            let indicator_rect = egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(0.0, GAP_SM),
                egui::vec2(3.0, rect.height() - GAP_SM * 2.0),
            );
            ui.painter().rect_filled(indicator_rect, RADIUS_SM, ACCENT);
        }

        let text_color = if selected || response.hovered() {
            TEXT_PRIMARY
        } else {
            TEXT_SECONDARY
        };

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child_ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(icon).size(20.0).color(text_color));
            ui.add_space(GAP_XS);
            ui.label(egui::RichText::new(label).size(TEXT_XS).color(text_color));
        });
    }

    response.clicked()
}
