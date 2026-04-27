use std::collections::BTreeMap;

use eframe::egui;
use tm_ipc::SessionsResponse;

use crate::design::{
    card_frame, header_text, inner_card_frame, body_text, muted_text, label_text, stat_value,
    ACCENT_SECONDARY, ACCENT_SECONDARY_DIM, BG_ELEVATED, BG_TERTIARY, SURFACE_HOVER,
    GAP_XS, GAP_SM, GAP_MD, GAP_LG, GAP_XL, RADIUS_SM,
};
use crate::format::format_duration_minutes_style;

struct WebsiteRow {
    subject_id: String,
    title: String,
    total_seconds: i64,
}

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.add_space(GAP_LG);
    ui.label(header_text("Websites"));
    ui.add_space(GAP_LG);

    let mut grouped: BTreeMap<String, WebsiteRow> = BTreeMap::new();
    for row in &payload.items {
        let entry = grouped
            .entry(row.subject_id.clone())
            .or_insert_with(|| WebsiteRow {
                subject_id: row.subject_id.clone(),
                title: row.title.clone(),
                total_seconds: 0,
            });
        entry.total_seconds += row.duration_seconds;
        entry.title = row.title.clone();
    }

    let mut websites: Vec<_> = grouped.into_values().collect();
    websites.sort_by_key(|a| std::cmp::Reverse(a.total_seconds));

    if websites.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(GAP_XL * 2.0);
            ui.label(muted_text("No website activity recorded for this period."));
        });
        return;
    }

    let total_websites = websites.len() as i64;
    let total_seconds: i64 = websites.iter().map(|a| a.total_seconds).sum();

    ui.horizontal(|ui| {
        let stat_card = |ui: &mut egui::Ui, label: &str, value: String| {
            card_frame().show(ui, |ui| {
                ui.set_width(120.0);
                ui.vertical(|ui| {
                    ui.label(label_text(label));
                    ui.add_space(GAP_XS);
                    ui.label(stat_value(value));
                });
            });
        };
        stat_card(ui, "Sites", total_websites.to_string());
        ui.add_space(GAP_MD);
        stat_card(ui, "Total Time", format_duration_minutes_style(total_seconds));
    });
    ui.add_space(GAP_LG);

    let max_seconds = websites.iter().map(|a| a.total_seconds).max().unwrap_or(1);

    inner_card_frame().show(ui, |ui| {
        let total_width = ui.available_width();
        let bar_col = 120.0;
        let duration_col = 80.0;
        let gap = GAP_MD;
        let name_col = (total_width - bar_col - duration_col - gap * 2.0).max(100.0);
        let row_height = 36.0;
        let bar_height = 6.0;

        for (i, site) in websites.iter().enumerate() {
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
                        let dot_size = 8.0;
                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(dot_size, dot_size),
                            egui::Sense::empty(),
                        );
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            dot_size / 2.0,
                            ACCENT_SECONDARY,
                        );
                        ui.add_space(GAP_SM);

                        ui.allocate_ui_with_layout(
                            egui::vec2(name_col - dot_size - GAP_SM, row_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(body_text(&site.subject_id));
                                if !site.title.is_empty() && site.title != site.subject_id {
                                    ui.label(muted_text(&site.title));
                                }
                            },
                        );

                        let (bar_rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_col, row_height),
                            egui::Sense::empty(),
                        );
                        let fraction = if max_seconds > 0 {
                            site.total_seconds as f32 / max_seconds as f32
                        } else {
                            0.0
                        };
                        let filled_width = (bar_col * fraction).max(2.0);
                        let bar_y = bar_rect.center().y - bar_height / 2.0;
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(bar_rect.min.x, bar_y),
                                egui::vec2(filled_width, bar_height),
                            ),
                            RADIUS_SM,
                            ACCENT_SECONDARY_DIM,
                        );

                        ui.add_space(gap);
                        ui.add_sized(
                            egui::vec2(duration_col, row_height),
                            egui::Label::new(muted_text(format_duration_minutes_style(site.total_seconds))),
                        );
                    });
                },
            );
        }
    });
}
