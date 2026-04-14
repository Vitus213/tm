use chrono::{TimeZone, Utc};
use tm_core::ActivityKind;
use tm_ipc::{
    ActivityFilter, ChartBucket, ChartsResponse, DaemonResponse, OverviewResponse, SessionRow,
    SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};
use tm_ui::{AppState, ConnectionState, LoadState, Page, TimeTab};

#[test]
fn disconnected_error_marks_connection_state_and_current_page() {
    let mut state = AppState::new(day_range(), TimeTab::Today);
    state.select_page(Page::Overview);
    state.apply_client_error("socket missing".into());

    assert!(matches!(state.connection, ConnectionState::Disconnected(_)));
    assert!(
        matches!(state.overview, LoadState::Error(message) if message.contains("socket missing"))
    );
}

#[test]
fn overview_response_populates_loaded_state() {
    let mut state = AppState::new(day_range(), TimeTab::Today);
    state.apply_response(DaemonResponse::Overview(OverviewResponse {
        range: day_range(),
        total_seconds: 900,
        top_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 900,
        }],
        top_websites: vec![],
        more_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 900,
        }],
        more_websites: vec![],
        recent_sessions: vec![SessionRow {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 15, 0).unwrap(),
            duration_seconds: 900,
        }],
    }));

    assert!(matches!(state.connection, ConnectionState::Connected));
    assert!(matches!(state.overview, LoadState::Loaded(data) if data.total_seconds == 900));
}

#[test]
fn sessions_response_populates_loaded_data_state() {
    let mut state = AppState::new(day_range(), TimeTab::Today);
    state.apply_response(DaemonResponse::Sessions(SessionsResponse {
        range: day_range(),
        activity_filter: ActivityFilter::All,
        subject_query: None,
        items: vec![SessionRow {
            kind: ActivityKind::Website,
            subject_id: "docs.rs".into(),
            title: "Rust docs".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 15, 0).unwrap(),
            duration_seconds: 900,
        }],
    }));

    assert!(matches!(state.data, LoadState::Loaded(payload) if payload.items.len() == 1));
}

#[test]
fn charts_response_populates_loaded_chart_state() {
    let mut state = AppState::new(day_range(), TimeTab::Today);
    state.apply_response(DaemonResponse::Charts(ChartsResponse {
        range: day_range(),
        app_share: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 900,
        }],
        website_share: vec![SummaryBucket {
            kind: ActivityKind::Website,
            subject_id: "docs.rs".into(),
            title: "Rust docs".into(),
            total_seconds: 600,
        }],
        daily_totals: vec![TrendPoint {
            bucket_start: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
            total_seconds: 1_500,
        }],
        hourly_totals: vec![ChartBucket {
            label: "09:00".into(),
            total_seconds: 900,
        }],
    }));

    assert!(matches!(state.charts, LoadState::Loaded(payload) if payload.daily_totals.len() == 1));
}

fn day_range() -> TimeRange {
    TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    }
}
