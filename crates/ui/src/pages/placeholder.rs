use eframe::egui;

use crate::design::{
    card_frame, header_text, muted_text,
    GAP_SM, GAP_LG, GAP_XL, TEXT_3XL, TEXT_MUTED,
};

pub fn render(ui: &mut egui::Ui, title: &str) {
    ui.add_space(GAP_XL);
    card_frame().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(GAP_XL * 2.0);
            ui.label(
                egui::RichText::new("◈")
                    .size(TEXT_3XL)
                    .color(TEXT_MUTED),
            );
            ui.add_space(GAP_LG);
            ui.label(header_text(title));
            ui.add_space(GAP_SM);
            ui.label(muted_text(
                "This section is part of the navigation shell and will gain real content in a later slice.",
            ));
            ui.add_space(GAP_XL * 2.0);
        });
    });
}
