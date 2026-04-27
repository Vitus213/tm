use eframe::egui;
use tm_ipc::Settings;

pub fn render(ui: &mut egui::Ui, settings: &mut Settings, dirty: &mut bool) {
    ui.heading("Settings");
    ui.add_space(16.0);

    ui.group(|ui| {
        ui.label("Idle detection");
        ui.horizontal(|ui| {
            ui.label("Idle threshold (seconds):");
            let mut seconds = settings.idle_threshold_seconds as i32;
            if ui
                .add(egui::DragValue::new(&mut seconds).range(0..=3600))
                .changed()
            {
                settings.idle_threshold_seconds = seconds as i64;
                *dirty = true;
            }
        });
    });

    ui.add_space(12.0);

    ui.group(|ui| {
        ui.label("Tracking");
        if ui
            .checkbox(
                &mut settings.website_tracking_enabled,
                "Enable website tracking",
            )
            .changed()
        {
            *dirty = true;
        }
    });

    ui.add_space(12.0);

    ui.group(|ui| {
        ui.label("Startup");
        if ui
            .checkbox(&mut settings.autostart_enabled, "Start daemon on login")
            .changed()
        {
            *dirty = true;
        }
    });

    ui.add_space(16.0);

    if *dirty && ui.button("Save").clicked() {
        *dirty = false;
    }
}
