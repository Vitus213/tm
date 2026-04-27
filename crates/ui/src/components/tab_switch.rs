use crate::design::*;
use eframe::egui;

pub fn tab_switch(ui: &mut egui::Ui, (left, right): (&str, &str), is_right_selected: bool) -> bool {
    let mut new_state = is_right_selected;
    let padding = egui::vec2(GAP_LG, GAP_SM);
    let font_id = egui::FontId::proportional(TEXT_BASE);

    let left_galley = ui.painter().layout(left.to_string(), font_id.clone(), TEXT_PRIMARY, f32::INFINITY);
    let right_galley = ui.painter().layout(right.to_string(), font_id.clone(), TEXT_PRIMARY, f32::INFINITY);

    let left_size = left_galley.rect.size() + padding * 2.0;
    let right_size = right_galley.rect.size() + padding * 2.0;
    let height = left_size.y.max(right_size.y);

    let (total_rect, _) = ui.allocate_exact_size(
        egui::vec2(left_size.x + right_size.x, height),
        egui::Sense::hover(),
    );

    ui.painter().rect_filled(total_rect, RADIUS_FULL, BG_TERTIARY);

    let left_rect = egui::Rect::from_min_size(total_rect.min, egui::vec2(left_size.x, height));
    let right_rect = egui::Rect::from_min_size(
        egui::pos2(total_rect.min.x + left_size.x, total_rect.min.y),
        egui::vec2(right_size.x, height),
    );

    let left_response = ui.interact(left_rect, ui.id().with(left), egui::Sense::click());
    let right_response = ui.interact(right_rect, ui.id().with(right), egui::Sense::click());

    if left_response.hovered() && is_right_selected {
        ui.painter().rect_filled(left_rect, RADIUS_FULL, BG_ELEVATED);
    }
    if right_response.hovered() && !is_right_selected {
        ui.painter().rect_filled(right_rect, RADIUS_FULL, BG_ELEVATED);
    }

    let active_rect = if is_right_selected { right_rect } else { left_rect };
    ui.painter().rect_filled(active_rect, RADIUS_FULL, ACCENT);

    let left_color = if !is_right_selected { TEXT_ON_ACCENT } else { TEXT_SECONDARY };
    let right_color = if is_right_selected { TEXT_ON_ACCENT } else { TEXT_SECONDARY };

    ui.painter().galley(
        left_rect.center() - left_galley.rect.size() * 0.5,
        left_galley,
        left_color,
    );
    ui.painter().galley(
        right_rect.center() - right_galley.rect.size() * 0.5,
        right_galley,
        right_color,
    );

    if left_response.clicked() {
        new_state = false;
    } else if right_response.clicked() {
        new_state = true;
    }

    new_state
}
