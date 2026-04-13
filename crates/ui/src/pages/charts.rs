use eframe::egui;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use tm_ipc::ChartsResponse;

pub fn render(ui: &mut egui::Ui, payload: &ChartsResponse) {
    ui.heading("Charts");

    let bars = payload
        .hourly_totals
        .iter()
        .enumerate()
        .map(|(index, bucket)| {
            Bar::new(index as f64, bucket.total_seconds as f64 / 60.0).name(bucket.label.clone())
        })
        .collect::<Vec<_>>();

    Plot::new("hourly-distribution").show(ui, |plot_ui| {
        plot_ui.bar_chart(BarChart::new(bars).name("Hourly"));
    });

    let points = payload
        .daily_totals
        .iter()
        .enumerate()
        .map(|(index, point)| [index as f64, point.total_seconds as f64 / 60.0])
        .collect::<PlotPoints<'_>>();

    Plot::new("daily-trend").show(ui, |plot_ui| {
        plot_ui.line(Line::new(points).name("Daily total"));
    });
}
