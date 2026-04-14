use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeTab {
    Today,
    Week,
    Month,
    Year,
}

impl TimeTab {
    pub fn to_range(self, now: DateTime<Utc>) -> TimeRange {
        let date = now.date_naive();
        let start_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

        match self {
            TimeTab::Today => {
                let started_at = date.and_time(start_time).and_local_timezone(Utc).unwrap();
                let ended_at = date.and_time(end_time).and_local_timezone(Utc).unwrap();
                TimeRange {
                    started_at,
                    ended_at,
                }
            }
            TimeTab::Week => {
                let weekday = date.weekday().num_days_from_monday();
                let monday = date - Duration::days(weekday as i64);
                let sunday = monday + Duration::days(6);
                let started_at = monday.and_time(start_time).and_local_timezone(Utc).unwrap();
                let ended_at = sunday.and_time(end_time).and_local_timezone(Utc).unwrap();
                TimeRange {
                    started_at,
                    ended_at,
                }
            }
            TimeTab::Month => {
                let first_day = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
                let last_day = if date.month() == 12 {
                    NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                } else {
                    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                };
                let started_at = first_day
                    .and_time(start_time)
                    .and_local_timezone(Utc)
                    .unwrap();
                let ended_at = last_day.and_time(end_time).and_local_timezone(Utc).unwrap();
                TimeRange {
                    started_at,
                    ended_at,
                }
            }
            TimeTab::Year => {
                let first_day = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
                let last_day = NaiveDate::from_ymd_opt(date.year(), 12, 31).unwrap();
                let started_at = first_day
                    .and_time(start_time)
                    .and_local_timezone(Utc)
                    .unwrap();
                let ended_at = last_day.and_time(end_time).and_local_timezone(Utc).unwrap();
                TimeRange {
                    started_at,
                    ended_at,
                }
            }
        }
    }
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
    pub time_tab: TimeTab,
    pub range: TimeRange,
    pub connection: ConnectionState,
    pub overview: LoadState<OverviewResponse>,
    pub charts: LoadState<ChartsResponse>,
    pub data: LoadState<SessionsResponse>,
}

impl AppState {
    pub fn new(range: TimeRange, time_tab: TimeTab) -> Self {
        Self {
            page: Page::Overview,
            time_tab,
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
