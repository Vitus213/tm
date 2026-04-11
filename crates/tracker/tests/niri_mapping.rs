use chrono::Utc;
use tm_tracker::{map_snapshot_to_event, FocusedWindowSnapshot};

#[test]
fn focused_window_maps_to_app_focus_event() {
    let snapshot = FocusedWindowSnapshot {
        app_id: "firefox".into(),
        title: "Rust docs".into(),
        pid: Some(4242),
        observed_at: Utc::now(),
    };

    let event = map_snapshot_to_event(snapshot);
    assert_eq!(event.subject_id, "firefox");
    assert_eq!(event.title, "Rust docs");
}
