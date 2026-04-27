use crate::design::*;
use eframe::egui;

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
            let padding = egui::vec2(GAP_LG, GAP_SM);
            let font_id = egui::FontId::proportional(TEXT_BASE);

            let galley = ui.painter().layout(label.to_string(), font_id.clone(), TEXT_PRIMARY, f32::INFINITY);
            let size = galley.rect.size() + padding * 2.0;

            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

            let bg_color = if is_selected {
                ACCENT
            } else if response.hovered() {
                BG_ELEVATED
            } else {
                egui::Color32::TRANSPARENT
            };

            if bg_color != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, RADIUS_FULL, bg_color);
            }

            let text_color = if is_selected {
                TEXT_ON_ACCENT
            } else if response.hovered() {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            };

            let galley = ui.painter().layout(label.to_string(), font_id, text_color, f32::INFINITY);
            ui.painter().galley(rect.center() - galley.rect.size() * 0.5, galley, text_color);

            if response.clicked() && !is_selected {
                new_index = Some(i);
            }

            if i < labels.len() - 1 {
                ui.add_space(GAP_SM);
            }
        }
    });

    new_index
}
