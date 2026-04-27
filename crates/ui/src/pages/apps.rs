use std::collections::BTreeMap;

use eframe::egui;
use tm_ipc::SessionsResponse;

struct AppRow {
    subject_id: String,
    title: String,
    total_seconds: i64,
}

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.heading("Apps");
    ui.add_space(8.0);

    let mut grouped: BTreeMap<String, AppRow> = BTreeMap::new();
    for row in &payload.items {
        let entry = grouped
            .entry(row.subject_id.clone())
            .or_insert_with(|| AppRow {
                subject_id: row.subject_id.clone(),
                title: row.title.clone(),
                total_seconds: 0,
            });
        entry.total_seconds += row.duration_seconds;
        entry.title = row.title.clone();
    }

    let mut apps: Vec<_> = grouped.into_values().collect();
    apps.sort_by_key(|a| std::cmp::Reverse(a.total_seconds));

    if apps.is_empty() {
        ui.label("No app activity recorded for this period.");
        return;
    }

    egui::Grid::new("apps-grid").striped(true).show(ui, |ui| {
        ui.label("App");
        ui.label("Title");
        ui.label("Duration");
        ui.end_row();

        for app in &apps {
            ui.label(&app.subject_id);
            ui.label(&app.title);
            ui.label(format!("{} min", app.total_seconds / 60));
            ui.end_row();
        }
    });
}
