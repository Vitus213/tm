use crate::design::*;
use eframe::egui;

pub fn card<R>(
    ui: &mut egui::Ui,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    card_frame().show(ui, content)
}
