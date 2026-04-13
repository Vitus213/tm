use eframe::egui;

pub fn render(ui: &mut egui::Ui, title: &str) {
    ui.heading(title);
    ui.label("This section is part of the Tai-like navigation shell and will gain real content in a later slice.");
}
