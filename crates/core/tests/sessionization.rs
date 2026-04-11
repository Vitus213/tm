use chrono::{TimeZone, Utc};
use tm_core::{ActivityEvent, ActivityKind, SessionAccumulator};

#[test]
fn focus_change_closes_previous_session() {
    let mut acc = SessionAccumulator::default();
    let t0 = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();
    let t1 = Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap();

    assert!(acc
        .ingest(ActivityEvent::app_focus("wezterm", "shell", t0))
        .is_none());

    let closed = acc
        .ingest(ActivityEvent::app_focus("firefox", "Rust docs", t1))
        .unwrap();

    assert_eq!(closed.kind, ActivityKind::App);
    assert_eq!(closed.subject_id, "wezterm");
    assert_eq!(closed.duration_seconds, 300);
}
