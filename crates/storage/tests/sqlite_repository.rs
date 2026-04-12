use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn inserts_and_reads_back_sessions() {
    let repo = SqliteRepository::in_memory().await.unwrap();

    let row = ClosedSession::new(
        ActivityKind::App,
        "firefox".into(),
        "Rust docs".into(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
    )
    .unwrap();

    repo.insert_session(&row).await.unwrap();
    let sessions = repo.list_sessions().await.unwrap();

    assert_eq!(sessions, vec![row]);
}

#[tokio::test]
async fn inserts_and_reads_back_website_sessions() {
    let repo = SqliteRepository::in_memory().await.unwrap();

    let row = ClosedSession::new(
        ActivityKind::Website,
        "docs.rs".into(),
        "Rust docs".into(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 10, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 15, 0).unwrap(),
    )
    .unwrap();

    repo.insert_session(&row).await.unwrap();
    let sessions = repo.list_sessions().await.unwrap();

    assert_eq!(sessions, vec![row]);
}
