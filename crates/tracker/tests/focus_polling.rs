use chrono::Utc;
use tm_tracker::{FocusedWindowSnapshot, should_emit_focus_event};

#[test]
fn unchanged_focus_snapshot_does_not_emit_duplicate_event() {
    let previous = sample_snapshot(7, Some("firefox"), "Rust docs");
    let current = sample_snapshot(7, Some("firefox"), "Rust docs");

    assert!(!should_emit_focus_event(Some(&previous), &current));
}

#[test]
fn changed_focus_snapshot_emits_event() {
    let previous = sample_snapshot(7, Some("firefox"), "Rust docs");

    let changed_window = sample_snapshot(8, Some("firefox"), "Rust docs");
    let changed_title = sample_snapshot(7, Some("firefox"), "Tracker plan");
    let changed_app_id = sample_snapshot(7, Some("wezterm"), "Rust docs");

    assert!(should_emit_focus_event(Some(&previous), &changed_window));
    assert!(should_emit_focus_event(Some(&previous), &changed_title));
    assert!(should_emit_focus_event(Some(&previous), &changed_app_id));
}

fn sample_snapshot(window_id: u64, app_id: Option<&str>, title: &str) -> FocusedWindowSnapshot {
    FocusedWindowSnapshot {
        window_id,
        app_id: app_id.map(str::to_owned),
        title: title.to_owned(),
        pid: Some(4242),
        observed_at: Utc::now(),
    }
}
