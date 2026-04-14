use eframe::egui;

/// Renders content inside a rounded card with a faint background and border.
pub fn card<R>(
    ui: &mut egui::Ui,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = egui::Frame::new()
        .corner_radius(8.0)
        .fill(ui.visuals().faint_bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .inner_margin(20.0);

    frame.show(ui, content)
}
