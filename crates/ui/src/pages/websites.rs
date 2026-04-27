use std::collections::BTreeMap;

use eframe::egui;
use tm_ipc::SessionsResponse;

struct WebsiteRow {
    subject_id: String,
    title: String,
    total_seconds: i64,
}

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.heading("Websites");
    ui.add_space(8.0);

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
        ui.label("No website activity recorded for this period.");
        return;
    }

    egui::Grid::new("websites-grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Domain");
            ui.label("Title");
            ui.label("Duration");
            ui.end_row();

            for site in &websites {
                ui.label(&site.subject_id);
                ui.label(&site.title);
                ui.label(format!("{} min", site.total_seconds / 60));
                ui.end_row();
            }
        });
}
