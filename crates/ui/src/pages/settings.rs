use eframe::egui;
use tm_ipc::Settings;

use crate::design::{
    card_frame, header_text, section_title, body_text, muted_text,
    ACCENT, GAP_SM, GAP_MD, GAP_LG, RADIUS_SM,
};

pub fn render(ui: &mut egui::Ui, settings: &mut Settings, dirty: &mut bool) {
    ui.add_space(GAP_LG);
    ui.horizontal(|ui| {
        ui.label(header_text("Settings"));
        if *dirty {
            ui.add_space(GAP_SM);
            ui.label(
                egui::RichText::new("●")
                    .size(14.0)
                    .color(ACCENT),
            );
        }
    });
    ui.add_space(GAP_LG);

    card_frame().show(ui, |ui| {
        ui.label(section_title("Idle Detection"));
        ui.add_space(GAP_MD);
        ui.horizontal(|ui| {
            ui.label(body_text("Idle threshold:"));
            ui.add_space(GAP_SM);
            let mut seconds = settings.idle_threshold_seconds as i32;
            if ui
                .add(egui::DragValue::new(&mut seconds).range(0..=3600))
                .changed()
            {
                settings.idle_threshold_seconds = seconds as i64;
                *dirty = true;
            }
            ui.label(muted_text("seconds"));
        });
    });
    ui.add_space(GAP_LG);

    card_frame().show(ui, |ui| {
        ui.label(section_title("Tracking"));
        ui.add_space(GAP_MD);
        if ui
            .checkbox(
                &mut settings.website_tracking_enabled,
                body_text("Enable website tracking"),
            )
            .changed()
        {
            *dirty = true;
        }
    });
    ui.add_space(GAP_LG);

    card_frame().show(ui, |ui| {
        ui.label(section_title("Startup"));
        ui.add_space(GAP_MD);
        if ui
            .checkbox(
                &mut settings.autostart_enabled,
                body_text("Start daemon on login"),
            )
            .changed()
        {
            *dirty = true;
        }
    });
    ui.add_space(GAP_LG);

    if *dirty {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let button = egui::Button::new(body_text("Save"))
                .fill(ACCENT)
                .corner_radius(RADIUS_SM);
            if ui.add(button).clicked() {
                *dirty = false;
            }
        });
    }
}
