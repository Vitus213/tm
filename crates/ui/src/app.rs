use std::sync::mpsc::{self, Receiver};
use std::thread;

use chrono::Utc;
use eframe::egui;
use tm_ipc::{ActivityFilter, ChartsQuery, DaemonRequest, OverviewQuery, SessionsQuery};

use crate::{
    client::IpcClient,
    components,
    pages::{charts, data, overview, placeholder},
    state::{AppState, ConnectionState, LoadState, Page, TimeTab},
};

pub struct TmApp {
    client: IpcClient,
    state: AppState,
    pending: Option<Receiver<Result<tm_ipc::DaemonResponse, String>>>,
}

impl Default for TmApp {
    fn default() -> Self {
        let time_tab = TimeTab::Today;
        let range = time_tab.to_range(Utc::now());
        Self {
            client: IpcClient::from_default_socket()
                .unwrap_or_else(|_| IpcClient::new(std::path::PathBuf::from("/tmp/tm.sock"))),
            state: AppState::new(range, time_tab),
            pending: None,
        }
    }
}

impl TmApp {
    fn request_current_page(&mut self) {
        let request = match self.state.page {
            Page::Overview => DaemonRequest::GetOverview(OverviewQuery {
                range: self.state.range.clone(),
            }),
            Page::Charts => DaemonRequest::GetCharts(ChartsQuery {
                range: self.state.range.clone(),
            }),
            Page::Data => DaemonRequest::GetSessions(SessionsQuery {
                range: self.state.range.clone(),
                activity_filter: ActivityFilter::All,
                subject_query: None,
            }),
            _ => return,
        };

        let client = IpcClient::new(self.client.socket_path().to_path_buf());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(client.send(request));
        });
        self.pending = Some(rx);
    }
}

impl eframe::App for TmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.pending
            && let Ok(result) = receiver.try_recv()
        {
            match result {
                Ok(response) => self.state.apply_response(response),
                Err(message) => self.state.apply_client_error(message),
            }
            self.pending = None;
        }

        egui::SidePanel::left("nav")
            .exact_width(80.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("tm");
                ui.vertical(|ui| {
                    for (icon, label, page) in [
                        ("📊", "Overview", Page::Overview),
                        ("📈", "Charts", Page::Charts),
                        ("📋", "Data", Page::Data),
                        ("📱", "Apps", Page::Apps),
                        ("🌐", "Websites", Page::Websites),
                        ("📁", "Categories", Page::Categories),
                    ] {
                        if components::nav_button::nav_button(
                            ui,
                            icon,
                            label,
                            self.state.page == page,
                        ) {
                            self.state.select_page(page);
                            self.request_current_page();
                        }
                    }
                    ui.add_space((ui.available_height() - 52.0).max(0.0));
                    if components::nav_button::nav_button(
                        ui,
                        "⚙️",
                        "Settings",
                        self.state.page == Page::Settings,
                    ) {
                        self.state.select_page(Page::Settings);
                        self.request_current_page();
                    }
                });
            });

        let mut frame = egui::Frame::central_panel(&ctx.style());
        frame.inner_margin.left = 10;
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            components::card::card(ui, |ui| match self.state.page {
                Page::Overview => match &self.state.overview {
                    LoadState::Loading => {
                        ui.label("Loading overview...");
                    }
                    LoadState::Loaded(payload) => {
                        if let Some(event) = overview::render(
                            ui,
                            self.state.time_tab,
                            self.state.overview_more_type,
                            payload,
                        ) {
                            match event {
                                overview::OverviewEvent::TimeTabChanged(tab) => {
                                    self.state.time_tab = tab;
                                    self.state.range = tab.to_range(Utc::now());
                                    self.state.overview = LoadState::Loading;
                                    self.state.charts = LoadState::Loading;
                                    self.state.data = LoadState::Loading;
                                    self.request_current_page();
                                }
                                overview::OverviewEvent::MoreTypeChanged(v) => {
                                    self.state.overview_more_type = v;
                                }
                            }
                        }
                    }
                    LoadState::Empty => {
                        ui.label("No overview data yet.");
                    }
                    LoadState::Error(message) => {
                        ui.label(message);
                    }
                },
                Page::Charts => match &self.state.charts {
                    LoadState::Loading => {
                        ui.label("Loading charts...");
                    }
                    LoadState::Loaded(payload) => charts::render(ui, payload),
                    LoadState::Empty => {
                        ui.label("No chart data yet.");
                    }
                    LoadState::Error(message) => {
                        ui.label(message);
                    }
                },
                Page::Data => match &self.state.data {
                    LoadState::Loading => {
                        ui.label("Loading sessions...");
                    }
                    LoadState::Loaded(payload) => data::render(ui, payload),
                    LoadState::Empty => {
                        ui.label("No sessions yet.");
                    }
                    LoadState::Error(message) => {
                        ui.label(message);
                    }
                },
                Page::Apps => placeholder::render(ui, "Apps"),
                Page::Websites => placeholder::render(ui, "Websites"),
                Page::Categories => placeholder::render(ui, "Categories"),
                Page::Settings => placeholder::render(ui, "Settings"),
            });
        });

        if self.pending.is_none() && matches!(self.state.connection, ConnectionState::Retrying) {
            self.request_current_page();
        }
        ctx.request_repaint();
    }
}
