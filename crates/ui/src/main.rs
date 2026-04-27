fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "tm",
        options,
        Box::new(|cc| {
            tm_ui::design::apply_theme(&cc.egui_ctx);
            Ok(Box::new(tm_ui::TmApp::default()))
        }),
    )
}
