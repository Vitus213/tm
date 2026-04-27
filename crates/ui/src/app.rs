use std::sync::mpsc::{self, Receiver};
use std::thread;

use chrono::Utc;
use eframe::egui;
use tm_ipc::{ActivityFilter, ChartsQuery, DaemonRequest, OverviewQuery, SessionsQuery};

use crate::{
    client::IpcClient,
    pages::{apps, charts, data, overview, placeholder, settings, websites},
    state::{AppState, ConnectionState, LoadState, Page},
};

pub struct TmApp {
    client: IpcClient,
    state: AppState,
    pending: Option<Receiver<Result<tm_ipc::DaemonResponse, String>>>,
    settings_dirty: bool,
}

impl Default for TmApp {
    fn default() -> Self {
        let range = tm_ipc::TimeRange {
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
        };
        let mut state = AppState::new(range);

        // Allow overriding the startup page via env var for automated screenshot testing
        if let Ok(page_env) = std::env::var("TM_UI_PAGE") {
            let page = match page_env.as_str() {
                "Overview" => Page::Overview,
                "Charts" => Page::Charts,
                "Data" => Page::Data,
                "Apps" => Page::Apps,
                "Websites" => Page::Websites,
                "Categories" => Page::Categories,
                "Settings" => Page::Settings,
                _ => Page::Overview,
            };
            state.select_page(page);
        }

        Self {
            client: IpcClient::from_default_socket()
                .unwrap_or_else(|_| IpcClient::new(std::path::PathBuf::from("/tmp/tm.sock"))),
            state,
            pending: None,
            settings_dirty: false,
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
            Page::Apps => DaemonRequest::GetSessions(SessionsQuery {
                range: self.state.range.clone(),
                activity_filter: ActivityFilter::App,
                subject_query: None,
            }),
            Page::Websites => DaemonRequest::GetSessions(SessionsQuery {
                range: self.state.range.clone(),
                activity_filter: ActivityFilter::Website,
                subject_query: None,
            }),
            Page::Settings => DaemonRequest::GetSettings,
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

        egui::CentralPanel::default().show(ctx, |ui| match self.state.page {
            Page::Overview => match &self.state.overview {
                LoadState::Loading => {
                    ui.label("Loading overview...");
                }
                LoadState::Loaded(payload) => overview::render(ui, payload),
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
            Page::Apps => match &self.state.data {
                LoadState::Loading => {
                    ui.label("Loading apps...");
                }
                LoadState::Loaded(payload) => apps::render(ui, payload),
                LoadState::Empty => {
                    ui.label("No app activity recorded.");
                }
                LoadState::Error(message) => {
                    ui.label(message.as_str());
                }
            },
            Page::Websites => match &self.state.data {
                LoadState::Loading => {
                    ui.label("Loading websites...");
                }
                LoadState::Loaded(payload) => websites::render(ui, payload),
                LoadState::Empty => {
                    ui.label("No website activity recorded.");
                }
                LoadState::Error(message) => {
                    ui.label(message.as_str());
                }
            },
            Page::Categories => placeholder::render(ui, "Categories"),
            Page::Settings => match &mut self.state.settings {
                LoadState::Loading => {
                    ui.label("Loading settings...");
                }
                LoadState::Loaded(settings) => {
                    settings::render(ui, settings, &mut self.settings_dirty);
                    if !self.settings_dirty {
                        // Reset to Loading after save so we fetch fresh state
                        self.state.settings = LoadState::Loading;
                        self.request_current_page();
                    }
                }
                LoadState::Empty => {
                    ui.label("No settings available.");
                }
                LoadState::Error(message) => {
                    ui.label(message.as_str());
                }
            },
        });

        if self.pending.is_none() && matches!(self.state.connection, ConnectionState::Retrying) {
            self.request_current_page();
        }
        ctx.request_repaint();
    }
}
