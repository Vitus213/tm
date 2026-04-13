use eframe::egui;
use tm_ipc::OverviewResponse;

use crate::format::format_duration_minutes_style;

pub fn render(ui: &mut egui::Ui, payload: &OverviewResponse) {
    ui.heading("Overview");
    ui.label(format!("Tracked: {}", format_duration_minutes_style(payload.total_seconds)));

    ui.separator();
    ui.label("Top apps");
    for row in &payload.top_apps {
        ui.label(format!(
            "{} — {}",
            row.subject_id,
            format_duration_minutes_style(row.total_seconds)
        ));
    }

    ui.separator();
    ui.label("Top websites");
    for row in &payload.top_websites {
        ui.label(format!(
            "{} — {}",
            row.subject_id,
            format_duration_minutes_style(row.total_seconds)
        ));
    }

    ui.separator();
    ui.label("Recent activity");
    for row in &payload.recent_sessions {
        ui.label(format!(
            "{} · {}",
            row.subject_id,
            format_duration_minutes_style(row.duration_seconds)
        ));
    }
}
