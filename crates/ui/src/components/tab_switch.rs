use eframe::egui;

/// A horizontal tab switch with two options.
///
/// Returns `false` if the first option (left) is selected,
/// or `true` if the second option (right) is selected.
pub fn tab_switch(ui: &mut egui::Ui, (left, right): (&str, &str), selected: bool) -> bool {
    let (label_left, label_right) = (left, right);

    ui.horizontal(|ui| {
        let response_left = ui.selectable_label(!selected, label_left);
        let response_right = ui.selectable_label(selected, label_right);

        if response_left.clicked() && selected {
            false
        } else if response_right.clicked() && !selected {
            true
        } else {
            selected
        }
    })
    .inner
}
