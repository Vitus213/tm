use eframe::egui;
use tm_ipc::SessionsResponse;

use crate::design::{
    header_text, inner_card_frame, body_text, muted_text,
    ACCENT, ACCENT_SECONDARY, BG_ELEVATED, BG_TERTIARY, SURFACE_HOVER,
    GAP_SM, GAP_MD, GAP_LG, RADIUS_SM,
};
use crate::format::format_duration_minutes_style;

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.add_space(GAP_LG);
    ui.label(header_text("Data"));
    ui.add_space(GAP_LG);

    if payload.items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(GAP_LG * 2.0);
            ui.label(muted_text("No sessions recorded for this period."));
        });
        return;
    }

    inner_card_frame().show(ui, |ui| {
        let total_width = ui.available_width();
        let kind_col = 80.0;
        let duration_col = 90.0;
        let gap = GAP_MD;
        let flex = (total_width - kind_col - duration_col - gap * 3.0).max(100.0);
        let subject_col = flex * 0.38;
        let title_col = flex * 0.62;
        let row_height = 32.0;

        let header_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(total_width, row_height),
        );
        ui.painter().rect_filled(header_rect, RADIUS_SM, BG_ELEVATED);
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(header_rect.shrink2(egui::vec2(gap, 0.0))),
            |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_sized(
                        egui::vec2(kind_col, row_height),
                        egui::Label::new(body_text("Kind").strong()),
                    );
                    ui.add_space(gap);
                    ui.add_sized(
                        egui::vec2(subject_col, row_height),
                        egui::Label::new(body_text("Subject").strong()),
                    );
                    ui.add_space(gap);
                    ui.add_sized(
                        egui::vec2(title_col, row_height),
                        egui::Label::new(body_text("Title").strong()),
                    );
                    ui.add_space(gap);
                    ui.add_sized(
                        egui::vec2(duration_col, row_height),
                        egui::Label::new(body_text("Duration").strong()),
                    );
                });
            },
        );
        ui.add_space(GAP_SM);

        for (i, row) in payload.items.iter().enumerate() {
            let (row_rect, _) = ui.allocate_exact_size(
                egui::vec2(total_width, row_height),
                egui::Sense::empty(),
            );
            let is_hovered = ui.rect_contains_pointer(row_rect);
            let bg = if is_hovered {
                SURFACE_HOVER
            } else if i % 2 == 0 {
                BG_ELEVATED
            } else {
                BG_TERTIARY
            };
            ui.painter().rect_filled(row_rect, RADIUS_SM, bg);

            ui.allocate_new_ui(
                egui::UiBuilder::new().max_rect(row_rect.shrink2(egui::vec2(gap, 0.0))),
                |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let kind_str = format!("{:?}", row.kind);
                        let (badge_label, badge_color) = if kind_str == "App" {
                            ("App", ACCENT)
                        } else {
                            ("Web", ACCENT_SECONDARY)
                        };
                        let badge_bg = badge_color.gamma_multiply(0.15);

                        let badge_width = 36.0;
                        let badge_height = 18.0;
                        let (badge_rect, _) = ui.allocate_exact_size(
                            egui::vec2(badge_width, badge_height),
                            egui::Sense::empty(),
                        );
                        ui.painter().rect_filled(badge_rect, RADIUS_SM, badge_bg);
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            badge_label,
                            egui::FontId::proportional(10.0),
                            badge_color,
                        );

                        ui.add_space(gap);
                        ui.add_sized(
                            egui::vec2(subject_col, row_height),
                            egui::Label::new(body_text(&row.subject_id)).truncate(),
                        );
                        ui.add_space(gap);
                        ui.add_sized(
                            egui::vec2(title_col, row_height),
                            egui::Label::new(body_text(&row.title)).truncate(),
                        );
                        ui.add_space(gap);
                        ui.add_sized(
                            egui::vec2(duration_col, row_height),
                            egui::Label::new(muted_text(format_duration_minutes_style(row.duration_seconds))),
                        );
                    });
                },
            );
        }
    });
}
