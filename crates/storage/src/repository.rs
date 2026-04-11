use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use tm_core::{ActivityKind, ClosedSession};

use crate::schema::BOOTSTRAP_SQL;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        sqlx::query(BOOTSTRAP_SQL).execute(&pool).await?;

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
        other => return Err(anyhow!("unknown activity kind: {other}")),
    };

    let subject_id = row.get(1);
    let title = row.get(2);
    let started_at = parse_timestamp(&row.get::<String, _>(3))?;
    let ended_at = parse_timestamp(&row.get::<String, _>(4))?;
    let stored_duration = row.get::<i64, _>(5);

    let session = ClosedSession::new(kind, subject_id, title, started_at, ended_at)
        .ok_or_else(|| anyhow!("stored session has negative duration"))?;

    if session.duration_seconds() != stored_duration {
        return Err(anyhow!(
            "stored session duration mismatch: expected {}, got {}",
            stored_duration,
            session.duration_seconds()
        ));
    }

    Ok(session)
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}
