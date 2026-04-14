use eframe::egui;
use tm_ipc::OverviewResponse;

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

    // A. Header row
    ui.horizontal(|ui| {
        ui.heading("Overview");
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

    ui.add_space(16.0);

    // B. "Most Frequent" section
    ui.label(
        egui::RichText::new("Most Frequent")
            .small()
            .color(ui.visuals().weak_text_color()),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let total_width = ui.available_width();
        let gap = 10.0;
        let left_width = (total_width - gap) * 0.6;
        let right_width = (total_width - gap) * 0.4;

        // Left Card — Apps
        let left_response = ui.allocate_ui_with_layout(
            egui::vec2(left_width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let frame = egui::Frame::new()
                    .corner_radius(10.0)
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(10.0);
                let inner = frame.show(ui, |ui| {
                    badge_header(
                        ui,
                        "📱",
                        "App",
                        egui::Color32::from_rgb(59, 130, 246),
                        egui::Color32::WHITE,
                    );
                    ui.add_space(8.0);
                    for row in payload.top_apps.iter().take(5) {
                        let name = if row.title.is_empty() {
                            &row.subject_id
                        } else {
                            &row.title
                        };
                        let duration = format_duration_minutes_style(row.total_seconds);
                        app_list_row(ui, 20.0, name, &duration);
                    }
                });
                let rect = inner.response.rect;
                ui.painter().text(
                    rect.max - egui::vec2(10.0, 10.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "📱",
                    egui::FontId::proportional(80.0),
                    ui.visuals().widgets.inactive.bg_fill.gamma_multiply(0.1),
                );
            },
        );

        ui.add_space(gap);

        // Right Card — Websites
        ui.allocate_ui_with_layout(
            egui::vec2(right_width, left_response.response.rect.height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let frame = egui::Frame::new()
                    .corner_radius(10.0)
                    .fill(ui.visuals().selection.bg_fill.gamma_multiply(0.08))
                    .inner_margin(10.0);
                let inner = frame.show(ui, |ui| {
                    badge_header(
                        ui,
                        "🌐",
                        "Website",
                        egui::Color32::from_rgb(139, 92, 246),
                        egui::Color32::WHITE,
                    );
                    ui.add_space(8.0);
                    for row in payload.top_websites.iter().take(5) {
                        let name = if row.title.is_empty() {
                            &row.subject_id
                        } else {
                            &row.title
                        };
                        let duration = format_duration_minutes_style(row.total_seconds);
                        app_list_row(ui, 18.0, name, &duration);
                    }
                });
                let rect = inner.response.rect;
                ui.painter().text(
                    rect.max - egui::vec2(10.0, 10.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "🌐",
                    egui::FontId::proportional(80.0),
                    ui.visuals().widgets.inactive.bg_fill.gamma_multiply(0.05),
                );
            },
        );
    });

    ui.add_space(24.0);

    // C. "More" section
    ui.horizontal(|ui| {
        ui.label("More");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let new_type =
                crate::components::tab_switch::tab_switch(ui, ("Apps", "Websites"), more_type);
            if new_type != more_type {
                event = Some(OverviewEvent::MoreTypeChanged(new_type));
            }
        });
    });
    ui.add_space(8.0);

    // Content area
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
            app_card(ui, name, &duration);
            ui.add_space(8.0);
        }
    });

    event
}

fn badge_header(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    bg_color: egui::Color32,
    text_color: egui::Color32,
) {
    let frame = egui::Frame::new()
        .corner_radius(4.0)
        .fill(bg_color)
        .inner_margin(egui::Margin::symmetric(6, 2));
    frame.show(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("{} {}", icon, label))
                .size(12.0)
                .color(text_color),
        );
    });
}

fn app_list_row(ui: &mut egui::Ui, icon_size: f32, name: &str, duration: &str) {
    ui.horizontal(|ui| {
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.inactive.bg_fill);
        ui.label(name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(duration);
        });
    });
}

fn app_card(ui: &mut egui::Ui, name: &str, duration: &str) {
    let card_width = 100.0;
    ui.allocate_ui_with_layout(
        egui::vec2(card_width, 90.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let icon_size = 32.0;
            let (rect, _response) =
                ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 6.0, ui.visuals().widgets.inactive.bg_fill);
            ui.add_space(4.0);
            ui.label(egui::RichText::new(name).size(11.0));
            ui.label(
                egui::RichText::new(duration)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        },
    );
}
