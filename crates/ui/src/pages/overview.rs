use eframe::egui;
use tm_ipc::OverviewResponse;

use crate::design::{
    card_frame, inner_card_frame, header_text, section_title, body_text, muted_text, stat_value,
    ACCENT, ACCENT_SECONDARY, BG_ELEVATED, TEXT_MUTED, RADIUS_SM, GAP_XS, GAP_SM, GAP_MD, GAP_LG,
    GAP_XL,
};
use crate::format::format_duration_minutes_style;
use crate::state::TimeTab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewEvent {
    TimeTabChanged(TimeTab),
    MoreTypeChanged(bool),
}

pub fn render(
    ui: &mut egui::Ui,
    time_tab: TimeTab,
    more_type: bool,
    payload: &OverviewResponse,
) -> Option<OverviewEvent> {
    let mut event = None;

    ui.horizontal(|ui| {
        ui.label(header_text("Overview"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let labels = &["Today", "Week", "Month", "Year"];
            let selected_index = time_tab as usize;
            if let Some(new_index) = crate::components::tabbar::tabbar(ui, labels, selected_index) {
                let new_tab = match new_index {
                    0 => TimeTab::Today,
                    1 => TimeTab::Week,
                    2 => TimeTab::Month,
                    3 => TimeTab::Year,
                    _ => time_tab,
                };
                event = Some(OverviewEvent::TimeTabChanged(new_tab));
            }
        });
    });

    ui.add_space(GAP_LG);

    let total_duration = format_duration_minutes_style(payload.total_seconds);
    let (top_app_name, top_app_duration) = payload
        .top_apps
        .first()
        .map(|a| {
            let name = if a.title.is_empty() {
                a.subject_id.clone()
            } else {
                a.title.clone()
            };
            let dur = format_duration_minutes_style(a.total_seconds);
            (name, dur)
        })
        .unwrap_or_else(|| ("-".to_string(), "-".to_string()));
    let (top_website_name, top_website_duration) = payload
        .top_websites
        .first()
        .map(|w| {
            let name = if w.title.is_empty() {
                w.subject_id.clone()
            } else {
                w.title.clone()
            };
            let dur = format_duration_minutes_style(w.total_seconds);
            (name, dur)
        })
        .unwrap_or_else(|| ("-".to_string(), "-".to_string()));
    let session_count = payload.recent_sessions.len().to_string();

    ui.horizontal(|ui| {
        let gap = GAP_MD;
        let count = 4.0;
        let width = (ui.available_width() - gap * (count - 1.0)) / count;

        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.label(muted_text("Total Time"));
                    ui.add_space(GAP_XS);
                    ui.label(stat_value(total_duration));
                });
            },
        );
        ui.add_space(gap);

        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.label(muted_text("Top App"));
                    ui.add_space(GAP_XS);
                    ui.label(body_text(top_app_name));
                    ui.label(muted_text(top_app_duration));
                });
            },
        );
        ui.add_space(gap);

        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.label(muted_text("Top Website"));
                    ui.add_space(GAP_XS);
                    ui.label(body_text(top_website_name));
                    ui.label(muted_text(top_website_duration));
                });
            },
        );
        ui.add_space(gap);

        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                card_frame().show(ui, |ui| {
                    ui.label(muted_text("Sessions"));
                    ui.add_space(GAP_XS);
                    ui.label(stat_value(session_count));
                });
            },
        );
    });

    ui.add_space(GAP_XL);

    ui.label(section_title("Most Frequent"));
    ui.add_space(GAP_MD);

    ui.horizontal(|ui| {
        let total_width = ui.available_width();
        let gap = GAP_LG;
        let left_width = (total_width - gap) * 0.6;
        let right_width = (total_width - gap) * 0.4;

        let left_response = ui.allocate_ui_with_layout(
            egui::vec2(left_width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let response = inner_card_frame().show(ui, |ui| {
                    badge_header(ui, "App", ACCENT);
                    ui.add_space(GAP_SM);
                    for row in payload.top_apps.iter().take(5) {
                        let name = if row.title.is_empty() {
                            &row.subject_id
                        } else {
                            &row.title
                        };
                        let duration = format_duration_minutes_style(row.total_seconds);
                        list_row(ui, name, &duration);
                    }
                });
                let rect = response.response.rect;
                ui.painter().text(
                    rect.max - egui::vec2(GAP_MD, GAP_MD),
                    egui::Align2::RIGHT_BOTTOM,
                    "📱",
                    egui::FontId::proportional(64.0),
                    TEXT_MUTED.gamma_multiply(0.12),
                );
            },
        );

        ui.add_space(gap);

        ui.allocate_ui_with_layout(
            egui::vec2(right_width, left_response.response.rect.height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let response = inner_card_frame().show(ui, |ui| {
                    badge_header(ui, "Website", ACCENT_SECONDARY);
                    ui.add_space(GAP_SM);
                    for row in payload.top_websites.iter().take(5) {
                        let name = if row.title.is_empty() {
                            &row.subject_id
                        } else {
                            &row.title
                        };
                        let duration = format_duration_minutes_style(row.total_seconds);
                        list_row(ui, name, &duration);
                    }
                });
                let rect = response.response.rect;
                ui.painter().text(
                    rect.max - egui::vec2(GAP_MD, GAP_MD),
                    egui::Align2::RIGHT_BOTTOM,
                    "🌐",
                    egui::FontId::proportional(64.0),
                    TEXT_MUTED.gamma_multiply(0.08),
                );
            },
        );
    });

    ui.add_space(GAP_XL);

    ui.horizontal(|ui| {
        ui.label(section_title("More"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let new_type =
                crate::components::tab_switch::tab_switch(ui, ("Apps", "Websites"), more_type);
            if new_type != more_type {
                event = Some(OverviewEvent::MoreTypeChanged(new_type));
            }
        });
    });
    ui.add_space(GAP_MD);

    let items = if !more_type {
        &payload.more_apps
    } else {
        &payload.more_websites
    };

    ui.horizontal_wrapped(|ui| {
        ui.set_width(ui.available_width());
        for row in items {
            let name = if row.title.is_empty() {
                &row.subject_id
            } else {
                &row.title
            };
            let duration = format_duration_minutes_style(row.total_seconds);
            more_card(ui, name, &duration);
            ui.add_space(GAP_SM);
        }
    });

    event
}

fn badge_header(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    let frame = egui::Frame::new()
        .corner_radius(RADIUS_SM)
        .fill(color.gamma_multiply(0.2))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.5)))
        .inner_margin(egui::Margin::symmetric(GAP_SM as i8, GAP_XS as i8));
    frame.show(ui, |ui| {
        ui.label(
            egui::RichText::new(label)
                .size(11.0)
                .color(color)
                .strong(),
        );
    });
}

fn list_row(ui: &mut egui::Ui, name: &str, duration: &str) {
    ui.horizontal(|ui| {
        let icon_size = 20.0;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
        ui.painter().rect_filled(rect, RADIUS_SM, BG_ELEVATED);
        ui.label(body_text(name));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(muted_text(duration));
        });
    });
    ui.add_space(GAP_XS);
}

fn more_card(ui: &mut egui::Ui, name: &str, duration: &str) {
    let card_width = 120.0;
    ui.allocate_ui_with_layout(
        egui::vec2(card_width, 100.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            inner_card_frame().show(ui, |ui| {
                let icon_size = 36.0;
                let (rect, _response) =
                    ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
                ui.painter().rect_filled(rect, RADIUS_SM, BG_ELEVATED);
                ui.add_space(GAP_SM);
                ui.label(body_text(name));
                ui.label(muted_text(duration));
            });
        },
    );
}
