use chrono::{TimeZone, Utc};
use serde_json::json;
use tm_core::ActivityKind;
use tm_ipc::{
    ActivityFilter, DaemonCommand, DaemonEvent, DaemonRequest, DaemonResponse, OverviewResponse,
    SessionRow, SessionsQuery, Settings, SummaryBucket, TimeRange,
};

#[test]
fn serializes_read_requests_and_responses_to_explicit_tagged_wire_format() {
    let range = TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    };

    let request = serde_json::to_value(DaemonRequest::GetSessions(SessionsQuery {
        range: range.clone(),
        activity_filter: ActivityFilter::Website,
        subject_query: Some("docs.rs".into()),
    }))
    .unwrap();

    let response = serde_json::to_value(DaemonResponse::Overview(OverviewResponse {
        range,
        total_seconds: 600,
        top_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 600,
        }],
        top_websites: vec![],
        more_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 600,
        }],
        more_websites: vec![],
        recent_sessions: vec![SessionRow {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 10, 0).unwrap(),
            duration_seconds: 600,
        }],
    }))
    .unwrap();

    assert_eq!(
        request,
        json!({
            "type": "get_sessions",
            "range": {
                "started_at": "2026-04-13T00:00:00Z",
                "ended_at": "2026-04-13T23:59:59Z"
            },
            "activity_filter": "website",
            "subject_query": "docs.rs"
        })
    );

    assert_eq!(
        response,
        json!({
            "type": "overview",
            "range": {
                "started_at": "2026-04-13T00:00:00Z",
                "ended_at": "2026-04-13T23:59:59Z"
            },
            "total_seconds": 600,
            "top_apps": [{
                "kind": "App",
                "subject_id": "wezterm",
                "title": "shell",
                "total_seconds": 600
            }],
            "top_websites": [],
            "more_apps": [{
                "kind": "App",
                "subject_id": "wezterm",
                "title": "shell",
                "total_seconds": 600
            }],
            "more_websites": [],
            "recent_sessions": [{
                "kind": "App",
                "subject_id": "wezterm",
                "title": "shell",
                "started_at": "2026-04-13T09:00:00Z",
                "ended_at": "2026-04-13T09:10:00Z",
                "duration_seconds": 600
            }]
        })
    );
}

#[test]
fn keeps_existing_operational_command_and_event_roundtrip() {
    let cmd_json = serde_json::to_string(&DaemonCommand::FlushActiveSession).unwrap();
    let evt_json = serde_json::to_string(&DaemonEvent::Ack).unwrap();

    let cmd: DaemonCommand = serde_json::from_str(&cmd_json).unwrap();
    let evt: DaemonEvent = serde_json::from_str(&evt_json).unwrap();

    assert_eq!(cmd, DaemonCommand::FlushActiveSession);
    assert_eq!(evt, DaemonEvent::Ack);
}

#[test]
fn serializes_settings_request_and_response() {
    let request = serde_json::to_value(DaemonRequest::GetSettings).unwrap();
    assert_eq!(request, json!({"type": "get_settings"}));

    let update = serde_json::to_value(DaemonRequest::UpdateSettings(Settings {
        idle_threshold_seconds: 600,
        website_tracking_enabled: false,
        autostart_enabled: true,
    }))
    .unwrap();
    assert_eq!(
        update,
        json!({
            "type": "update_settings",
            "idle_threshold_seconds": 600,
            "website_tracking_enabled": false,
            "autostart_enabled": true
        })
    );

    let response = serde_json::to_value(DaemonResponse::Settings(Settings {
        idle_threshold_seconds: 300,
        website_tracking_enabled: true,
        autostart_enabled: false,
    }))
    .unwrap();
    assert_eq!(
        response,
        json!({
            "type": "settings",
            "idle_threshold_seconds": 300,
            "website_tracking_enabled": true,
            "autostart_enabled": false
        })
    );
}

#[test]
fn settings_roundtrips_through_json() {
    let settings = Settings {
        idle_threshold_seconds: 900,
        website_tracking_enabled: false,
        autostart_enabled: true,
    };

    let json = serde_json::to_string(&settings).unwrap();
    let back: Settings = serde_json::from_str(&json).unwrap();

    assert_eq!(back, settings);
}
