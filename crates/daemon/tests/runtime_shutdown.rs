use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{TimeZone, Utc};
use tm_core::ActivityEvent;
use tm_daemon::{FlushOutcome, SessionService};
use tm_storage::SqliteRepository;

struct DatabaseFixture {
    root: PathBuf,
    db_path: PathBuf,
}

impl DatabaseFixture {
    fn new() -> Self {
        let unique = format!(
            "tm-daemon-runtime-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let db_path = root.join("state").join("tm.db");

        Self { root, db_path }
    }

    fn db_path(&self) -> &Path {
        &self.db_path
    }
}

impl Drop for DatabaseFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn flush_persists_trailing_active_session() {
    let fixture = DatabaseFixture::new();
    let repo = SqliteRepository::open_at(fixture.db_path()).await.unwrap();
    let mut service = SessionService::new(repo);

    service
        .ingest(ActivityEvent::app_focus(
            "firefox",
            "Rust docs",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ))
        .await
        .unwrap();

    let flushed = service
        .flush(Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap())
        .await
        .unwrap();

    assert_eq!(flushed, FlushOutcome::Persisted);

    drop(service);

    let reopened = SqliteRepository::open_at(fixture.db_path()).await.unwrap();
    let sessions = reopened.list_sessions().await.unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id(), "firefox");
    assert_eq!(
        sessions[0].ended_at(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap()
    );
}
