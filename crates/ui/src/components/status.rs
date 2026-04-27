use crate::design::*;
use eframe::egui;

pub fn loading_spinner(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let time = ui.input(|i| i.time) as f32;
        let dot_radius = 3.0;
        let dot_gap = GAP_SM;

        for i in 0..3 {
            let phase = (time * 3.0 + i as f32 * 1.3).sin();
            let alpha = (phase + 1.0) * 0.4 + 0.2;
            let color = ACCENT.gamma_multiply(alpha);

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(dot_radius * 2.0, dot_radius * 2.0),
                egui::Sense::hover(),
            );
            ui.painter().circle_filled(rect.center(), dot_radius, color);

            if i < 2 {
                ui.add_space(dot_gap);
            }
        }

        ui.add_space(GAP_MD);
        ui.label(muted_text("Loading..."));
    });
}

pub fn empty_state(ui: &mut egui::Ui, icon: &str, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(GAP_XL);
        ui.label(egui::RichText::new(icon).size(32.0).color(TEXT_MUTED));
        ui.add_space(GAP_MD);
        ui.label(muted_text(message));
        ui.add_space(GAP_XL);
    });
}

pub fn error_state(ui: &mut egui::Ui, message: &str) -> bool {
    let mut retry_clicked = false;

    ui.vertical_centered(|ui| {
        ui.add_space(GAP_XL);
        ui.label(egui::RichText::new("⚠").size(32.0).color(ERROR));
        ui.add_space(GAP_MD);
        ui.label(body_text(message));
        ui.add_space(GAP_LG);

        if ui.button(body_text("Retry")).clicked() {
            retry_clicked = true;
        }

        ui.add_space(GAP_XL);
    });

    retry_clicked
}
