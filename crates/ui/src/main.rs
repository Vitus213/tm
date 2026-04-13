fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "tm",
        options,
        Box::new(|_cc| Ok(Box::new(tm_ui::TmApp::default()))),
    )
}
