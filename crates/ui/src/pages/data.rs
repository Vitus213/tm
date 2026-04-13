use eframe::egui;
use tm_ipc::SessionsResponse;

use crate::format::format_duration_minutes_style;

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.heading("Data");
    egui::Grid::new("session-grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Kind");
            ui.label("Subject");
            ui.label("Title");
            ui.label("Duration");
            ui.end_row();

            for row in &payload.items {
                ui.label(format!("{:?}", row.kind));
                ui.label(&row.subject_id);
                ui.label(&row.title);
                ui.label(format_duration_minutes_style(row.duration_seconds));
                ui.end_row();
            }
        });
}
