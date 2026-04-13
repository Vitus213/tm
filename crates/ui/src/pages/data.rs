use eframe::egui;
use tm_ipc::SessionsResponse;

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
                ui.label(format!("{} min", row.duration_seconds / 60));
                ui.end_row();
            }
        });
}
