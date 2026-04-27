use eframe::egui;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use tm_ipc::ChartsResponse;

use crate::design::{
    inner_card_frame, header_text, muted_text, CHART_BLUE, CHART_PURPLE, GAP_LG, GAP_XL,
};

pub fn render(ui: &mut egui::Ui, payload: &ChartsResponse) {
    ui.label(header_text("Charts"));
    ui.add_space(GAP_LG);

    inner_card_frame().show(ui, |ui| {
        ui.label(muted_text("Hourly Distribution"));
        ui.add_space(GAP_LG);

        let bars = payload
            .hourly_totals
            .iter()
            .enumerate()
            .map(|(index, bucket)| {
                Bar::new(index as f64, bucket.total_seconds as f64 / 60.0)
                    .name(bucket.label.clone())
                    .fill(CHART_BLUE)
                    .width(0.7)
            })
            .collect::<Vec<_>>();

        Plot::new("hourly-distribution")
            .height(200.0)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(bars).name("Hourly"));
            });
    });

    ui.add_space(GAP_XL);

    inner_card_frame().show(ui, |ui| {
        ui.label(muted_text("Daily Trend"));
        ui.add_space(GAP_LG);

        let points = payload
            .daily_totals
            .iter()
            .enumerate()
            .map(|(index, point)| [index as f64, point.total_seconds as f64 / 60.0])
            .collect::<PlotPoints<'_>>();

        Plot::new("daily-trend")
            .height(200.0)
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(points)
                        .name("Daily total")
                        .stroke(egui::Stroke::new(2.0, CHART_PURPLE)),
                );
            });
    });
}
