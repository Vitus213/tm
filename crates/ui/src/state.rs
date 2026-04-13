use tm_ipc::{ChartsResponse, DaemonResponse, OverviewResponse, SessionsResponse, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Charts,
    Data,
    Apps,
    Websites,
    Categories,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected(String),
    Retrying,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    Loading,
    Loaded(T),
    Empty,
    Error(String),
}

pub struct AppState {
    pub page: Page,
    pub range: TimeRange,
    pub connection: ConnectionState,
    pub overview: LoadState<OverviewResponse>,
    pub charts: LoadState<ChartsResponse>,
    pub data: LoadState<SessionsResponse>,
}

impl AppState {
    pub fn new(range: TimeRange) -> Self {
        Self {
            page: Page::Overview,
            range,
            connection: ConnectionState::Retrying,
            overview: LoadState::Loading,
            charts: LoadState::Loading,
            data: LoadState::Loading,
        }
    }

    pub fn select_page(&mut self, page: Page) {
        self.page = page;
    }

    pub fn apply_client_error(&mut self, message: String) {
        self.connection = ConnectionState::Disconnected(message.clone());
        match self.page {
            Page::Overview => self.overview = LoadState::Error(message),
            Page::Charts => self.charts = LoadState::Error(message),
            Page::Data => self.data = LoadState::Error(message),
            _ => {}
        }
    }

    pub fn apply_response(&mut self, response: DaemonResponse) {
        self.connection = ConnectionState::Connected;
        match response {
            DaemonResponse::Overview(payload) => {
                self.overview = if payload.recent_sessions.is_empty() {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Charts(payload) => {
                self.charts = if payload.daily_totals.is_empty() && payload.hourly_totals.is_empty()
                {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Sessions(payload) => {
                self.data = if payload.items.is_empty() {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Pong | DaemonResponse::Error { .. } => {}
        }
    }
}
