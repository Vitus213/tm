use chrono::Utc;
use tm_tracker::{map_snapshot_to_event, FocusedWindowSnapshot};

#[test]
fn focused_window_maps_to_app_focus_event() {
    let snapshot = FocusedWindowSnapshot {
        window_id: 7,
        app_id: Some("firefox".into()),
        title: "Rust docs".into(),
        pid: Some(4242),
        observed_at: Utc::now(),
    };

    let event = map_snapshot_to_event(&snapshot);
    assert_eq!(event.subject_id, "firefox");
    assert_eq!(event.title, "Rust docs");
}

#[test]
fn missing_app_id_uses_window_scoped_subject_id() {
    let observed_at = Utc::now();
    let first = FocusedWindowSnapshot {
        window_id: 41,
        app_id: None,
        title: "First".into(),
        pid: Some(1000),
        observed_at,
    };
    let second = FocusedWindowSnapshot {
        window_id: 42,
        app_id: None,
        title: "Second".into(),
        pid: Some(1001),
        observed_at,
    };

    let first_event = map_snapshot_to_event(&first);
    let second_event = map_snapshot_to_event(&second);

    assert_eq!(first_event.subject_id, "niri-window:41");
    assert_eq!(second_event.subject_id, "niri-window:42");
    assert_ne!(first_event.subject_id, second_event.subject_id);
}
