use chrono::{TimeZone, Utc};
use tm_core::ActivityKind;
use tm_ipc::{ActivityFilter, DaemonResponse, SessionRow, SessionsResponse, TimeRange};
use tm_ui::{AppState, LoadState};

#[test]
fn app_filter_response_populates_data_state() {
    let mut state = AppState::new(day_range());
    state.apply_response(DaemonResponse::Sessions(SessionsResponse {
        range: day_range(),
        activity_filter: ActivityFilter::App,
        subject_query: None,
        items: vec![
            SessionRow {
                kind: ActivityKind::App,
                subject_id: "wezterm".into(),
                title: "shell".into(),
                started_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
                ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 15, 0).unwrap(),
                duration_seconds: 900,
            },
            SessionRow {
                kind: ActivityKind::App,
                subject_id: "firefox".into(),
                title: "Rust docs".into(),
                started_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(),
                ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 30, 0).unwrap(),
                duration_seconds: 1800,
            },
        ],
    }));

    assert!(matches!(state.data, LoadState::Loaded(payload) if payload.items.len() == 2));
}

fn day_range() -> TimeRange {
    TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    }
}
