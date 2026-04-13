use eframe::egui;
use tm_ipc::OverviewResponse;

pub fn render(ui: &mut egui::Ui, payload: &OverviewResponse) {
    ui.heading("Overview");
    ui.label(format!("Tracked: {} minutes", payload.total_seconds / 60));

    ui.separator();
    ui.label("Top apps");
    for row in &payload.top_apps {
        ui.label(format!(
            "{} — {} min",
            row.subject_id,
            row.total_seconds / 60
        ));
    }

    ui.separator();
    ui.label("Top websites");
    for row in &payload.top_websites {
        ui.label(format!(
            "{} — {} min",
            row.subject_id,
            row.total_seconds / 60
        ));
    }

    ui.separator();
    ui.label("Recent activity");
    for row in &payload.recent_sessions {
        ui.label(format!(
            "{} · {} min",
            row.subject_id,
            row.duration_seconds / 60
        ));
    }
}
