use chrono::{TimeZone, Utc};
use tm_core::{ActivityEvent, ActivityKind, ClosedSession, SessionAccumulator};

fn ts(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 4, 12, hour, minute, 0)
        .single()
        .unwrap()
}

#[test]
fn closed_session_constructor_rejects_out_of_order_timestamps() {
    let started_at = ts(9, 5);
    let ended_at = ts(9, 0);

    assert!(
        ClosedSession::new(
            ActivityKind::App,
            "wezterm".to_owned(),
            "shell".to_owned(),
            started_at,
            ended_at,
        )
        .is_none()
    );
}

#[test]
fn focus_change_closes_previous_session() {
    let mut acc = SessionAccumulator::default();
    let t0 = ts(9, 0);
    let t1 = ts(9, 5);

    assert!(
        acc.ingest(ActivityEvent::app_focus("wezterm", "shell", t0))
            .is_none()
    );

    let closed = acc
        .ingest(ActivityEvent::app_focus("firefox", "Rust docs", t1))
        .unwrap();

    assert_eq!(closed.kind(), ActivityKind::App);
    assert_eq!(closed.subject_id(), "wezterm");
    assert_eq!(closed.duration_seconds(), 300);
}

#[test]
fn flush_closes_active_session() {
    let mut acc = SessionAccumulator::default();
    let started_at = ts(9, 0);
    let ended_at = ts(9, 5);

    assert!(
        acc.ingest(ActivityEvent::app_focus("wezterm", "shell", started_at))
            .is_none()
    );

    let closed = acc.flush(ended_at).unwrap();

    assert_eq!(closed.kind(), ActivityKind::App);
    assert_eq!(closed.subject_id(), "wezterm");
    assert_eq!(closed.title(), "shell");
    assert_eq!(closed.started_at(), started_at);
    assert_eq!(closed.ended_at(), ended_at);
    assert_eq!(closed.duration_seconds(), 300);
}

#[test]
fn flush_without_active_session_returns_none() {
    let mut acc = SessionAccumulator::default();

    assert!(acc.flush(ts(9, 5)).is_none());
}

#[test]
fn equal_timestamp_boundary_produces_zero_duration_session() {
    let mut acc = SessionAccumulator::default();
    let boundary = ts(9, 0);

    assert!(
        acc.ingest(ActivityEvent::app_focus("wezterm", "shell", boundary))
            .is_none()
    );

    let closed = acc
        .ingest(ActivityEvent::app_focus("firefox", "Rust docs", boundary))
        .unwrap();

    assert_eq!(closed.started_at(), boundary);
    assert_eq!(closed.ended_at(), boundary);
    assert_eq!(closed.duration_seconds(), 0);
}

#[test]
fn out_of_order_ingest_is_rejected_and_preserves_active_session() {
    let mut acc = SessionAccumulator::default();
    let started_at = ts(9, 5);
    let earlier = ts(9, 0);
    let ended_at = ts(9, 10);

    assert!(
        acc.ingest(ActivityEvent::app_focus("wezterm", "shell", started_at))
            .is_none()
    );

    assert!(
        acc.ingest(ActivityEvent::app_focus("firefox", "Rust docs", earlier))
            .is_none()
    );

    let closed = acc.flush(ended_at).unwrap();

    assert_eq!(closed.subject_id(), "wezterm");
    assert_eq!(closed.title(), "shell");
    assert_eq!(closed.started_at(), started_at);
    assert_eq!(closed.ended_at(), ended_at);
    assert_eq!(closed.duration_seconds(), 300);
}

#[test]
fn out_of_order_flush_is_rejected_and_preserves_active_session() {
    let mut acc = SessionAccumulator::default();
    let started_at = ts(9, 5);
    let earlier = ts(9, 0);
    let ended_at = ts(9, 10);

    assert!(
        acc.ingest(ActivityEvent::app_focus("wezterm", "shell", started_at))
            .is_none()
    );

    assert!(acc.flush(earlier).is_none());

    let closed = acc.flush(ended_at).unwrap();

    assert_eq!(closed.subject_id(), "wezterm");
    assert_eq!(closed.started_at(), started_at);
    assert_eq!(closed.ended_at(), ended_at);
    assert_eq!(closed.duration_seconds(), 300);
}
