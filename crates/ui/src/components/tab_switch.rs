use eframe::egui;

pub fn tab_switch(ui: &mut egui::Ui, labels: (&str, &str), selected: bool) -> bool {
    let (label_left, label_right) = labels;

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
