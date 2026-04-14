use eframe::egui;

/// A horizontal tab switch with two options.
///
/// # Parameters
/// - `left`: The label for the left option
/// - `right`: The label for the right option
/// - `is_right_selected`: Whether the right option is currently selected (false = left selected)
///
/// # Returns
/// The *new* state after a click: `false` if the left option is selected,
/// or `true` if the right option is selected. Returns the original state if no click occurred.
pub fn tab_switch(ui: &mut egui::Ui, (left, right): (&str, &str), is_right_selected: bool) -> bool {
    let (label_left, label_right) = (left, right);

    ui.horizontal(|ui| {
        let response_left = ui.selectable_label(!is_right_selected, label_left);
        let response_right = ui.selectable_label(is_right_selected, label_right);

        if response_left.clicked() && is_right_selected {
            false
        } else if response_right.clicked() && !is_right_selected {
            true
        } else {
            is_right_selected
        }
    })
    .inner
}
