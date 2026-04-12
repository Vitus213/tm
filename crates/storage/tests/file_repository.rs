use chrono::{TimeZone, Utc};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tm_core::{ActivityKind, ClosedSession};
use tm_storage::SqliteRepository;

struct DatabaseFixture {
    root: PathBuf,
    db_path: PathBuf,
}

impl DatabaseFixture {
    fn new() -> Self {
        let unique = format!(
            "tm-storage-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let db_path = root.join("state").join("sessions.sqlite");

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
async fn file_backed_repository_persists_sessions_across_reopen() {
    let fixture = DatabaseFixture::new();
    let session = ClosedSession::new(
        ActivityKind::App,
        "firefox".into(),
        "Rust docs".into(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
    )
    .unwrap();

    let repo = SqliteRepository::open_at(fixture.db_path()).await.unwrap();
    repo.insert_session(&session).await.unwrap();

    assert!(fixture.db_path().parent().unwrap().exists());
    assert!(fixture.db_path().exists());

    drop(repo);

    let reopened = SqliteRepository::open_at(fixture.db_path()).await.unwrap();
    let sessions = reopened.list_sessions().await.unwrap();

    assert_eq!(sessions, vec![session]);
}
