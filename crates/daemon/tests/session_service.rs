use chrono::{TimeZone, Utc};
use tm_core::ActivityEvent;
use tm_daemon::SessionService;
use tm_storage::SqliteRepository;

#[tokio::test]
async fn daemon_persists_closed_session_on_focus_change() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    service
        .ingest(ActivityEvent::app_focus(
            "wezterm",
            "shell",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ))
        .await
        .unwrap();

    service
        .ingest(ActivityEvent::app_focus(
            "firefox",
            "Rust docs",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
        ))
        .await
        .unwrap();

    let sessions = service.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id(), "wezterm");
}
