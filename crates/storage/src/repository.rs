use chrono::{DateTime, Utc};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;
use thiserror::Error;
use tm_core::{ActivityKind, ClosedSession};

use crate::schema::BOOTSTRAP_SQL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub idle_threshold_seconds: i64,
    pub website_tracking_enabled: bool,
    pub autostart_enabled: bool,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid activity kind stored in sqlite: {0}")]
    InvalidActivityKind(String),
    #[error("invalid stored session: {0}")]
    InvalidStoredSession(String),
    #[error("stored session duration mismatch: stored {stored}, recomputed {recomputed}")]
    DurationMismatch { stored: i64, recomputed: i64 },
}

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        bootstrap_schema(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        bootstrap_schema(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn insert_session(&self, session: &ClosedSession) -> Result<()> {
        sqlx::query(
            "INSERT INTO sessions (kind, subject_id, title, started_at, ended_at, duration_seconds) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(activity_kind_to_str(session.kind()))
        .bind(session.subject_id())
        .bind(session.title())
        .bind(session.started_at().to_rfc3339())
        .bind(session.ended_at().to_rfc3339())
        .bind(session.duration_seconds())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<Vec<ClosedSession>> {
        let rows = sqlx::query(
            "SELECT kind, subject_id, title, started_at, ended_at, duration_seconds FROM sessions ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(session_from_row).collect()
    }

    pub async fn get_settings(&self) -> Result<Settings> {
        let row = sqlx::query(
            "SELECT idle_threshold_seconds, website_tracking_enabled, autostart_enabled FROM settings WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Settings {
            idle_threshold_seconds: row.get(0),
            website_tracking_enabled: row.get::<i64, _>(1) != 0,
            autostart_enabled: row.get::<i64, _>(2) != 0,
        })
    }

    pub async fn save_settings(&self, settings: &Settings) -> Result<()> {
        sqlx::query(
            "UPDATE settings SET idle_threshold_seconds = ?, website_tracking_enabled = ?, autostart_enabled = ? WHERE id = 1",
        )
        .bind(settings.idle_threshold_seconds)
        .bind(if settings.website_tracking_enabled { 1 } else { 0 })
        .bind(if settings.autostart_enabled { 1 } else { 0 })
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

async fn bootstrap_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(BOOTSTRAP_SQL).execute(pool).await?;
    Ok(())
}

fn activity_kind_to_str(kind: ActivityKind) -> &'static str {
    match kind {
        ActivityKind::App => "app",
        ActivityKind::Website => "website",
    }
}

fn session_from_row(row: sqlx::sqlite::SqliteRow) -> Result<ClosedSession> {
    let kind = match row.get::<String, _>(0).as_str() {
        "app" => ActivityKind::App,
        "website" => ActivityKind::Website,
        other => return Err(RepositoryError::InvalidActivityKind(other.to_owned())),
    };

    let subject_id = row.get(1);
    let title = row.get(2);
    let started_at =
        parse_timestamp(&row.get::<String, _>(3)).map_err(RepositoryError::InvalidStoredSession)?;
    let ended_at =
        parse_timestamp(&row.get::<String, _>(4)).map_err(RepositoryError::InvalidStoredSession)?;
    let stored_duration = row.get::<i64, _>(5);

    let session =
        ClosedSession::new(kind, subject_id, title, started_at, ended_at).ok_or_else(|| {
            RepositoryError::InvalidStoredSession("stored session has negative duration".into())
        })?;

    if session.duration_seconds() != stored_duration {
        return Err(RepositoryError::DurationMismatch {
            stored: stored_duration,
            recomputed: session.duration_seconds(),
        });
    }

    Ok(session)
}

fn parse_timestamp(value: &str) -> std::result::Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|err| format!("invalid RFC 3339 timestamp `{value}`: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[tokio::test]
    async fn rejects_invalid_kind_with_schema_constraint() {
        let repo = SqliteRepository::in_memory().await.unwrap();

        let err = sqlx::query(
            "INSERT INTO sessions (kind, subject_id, title, started_at, ended_at, duration_seconds) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("tab")
        .bind("docs.rs")
        .bind("Rust docs")
        .bind(Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap().to_rfc3339())
        .bind(Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap().to_rfc3339())
        .bind(300_i64)
        .execute(&repo.pool)
        .await
        .unwrap_err();

        assert!(err.to_string().contains("CHECK constraint failed"));
    }

    #[tokio::test]
    async fn reports_invalid_stored_timestamp_clearly() {
        let repo = SqliteRepository::in_memory().await.unwrap();

        sqlx::query(
            "INSERT INTO sessions (kind, subject_id, title, started_at, ended_at, duration_seconds) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("app")
        .bind("firefox")
        .bind("Rust docs")
        .bind("not-a-timestamp")
        .bind(Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap().to_rfc3339())
        .bind(300_i64)
        .execute(&repo.pool)
        .await
        .unwrap();

        let err = repo.list_sessions().await.unwrap_err();

        match err {
            RepositoryError::InvalidStoredSession(message) => {
                assert!(message.contains("invalid RFC 3339 timestamp `not-a-timestamp`"));
            }
            other => panic!("expected InvalidStoredSession, got {other:?}"),
        }
    }
}
