use chrono::{TimeZone, Utc};
use tm_core::ActivityEvent;
use tm_daemon::{FlushOutcome, IngestOutcome, SessionService};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn daemon_persists_closed_session_on_focus_change() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    let first_outcome = service
        .ingest(ActivityEvent::app_focus(
            "wezterm",
            "shell",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(first_outcome, IngestOutcome::Buffered);

    let second_outcome = service
        .ingest(ActivityEvent::app_focus(
            "firefox",
            "Rust docs",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(second_outcome, IngestOutcome::Persisted);

    let sessions = service.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id(), "wezterm");
}

#[tokio::test]
async fn daemon_flush_persists_trailing_open_session() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    let started_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();
    let ended_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap();

    let ingest_outcome = service
        .ingest(ActivityEvent::app_focus("wezterm", "shell", started_at))
        .await
        .unwrap();

    assert_eq!(ingest_outcome, IngestOutcome::Buffered);

    let flush_outcome = service.flush(ended_at).await.unwrap();
    assert_eq!(flush_outcome, FlushOutcome::Persisted);

    let sessions = service.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id(), "wezterm");
    assert_eq!(sessions[0].started_at(), started_at);
    assert_eq!(sessions[0].ended_at(), ended_at);
}

#[tokio::test]
async fn daemon_reports_rejected_out_of_order_events() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    let later = Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap();
    let earlier = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();

    assert_eq!(
        service
            .ingest(ActivityEvent::app_focus("wezterm", "shell", later))
            .await
            .unwrap(),
        IngestOutcome::Buffered
    );

    assert_eq!(
        service
            .ingest(ActivityEvent::app_focus("firefox", "Rust docs", earlier))
            .await
            .unwrap(),
        IngestOutcome::Ignored
    );

    assert_eq!(service.flush(earlier).await.unwrap(), FlushOutcome::Ignored);

    let sessions = service.list_sessions().await.unwrap();
    assert!(sessions.is_empty());
}
