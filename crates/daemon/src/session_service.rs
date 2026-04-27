use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tm_core::{ActivityEvent, ClosedSession, SessionAccumulator};
use tm_storage::{RepositoryError, SqliteRepository};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestOutcome {
    Buffered,
    Persisted,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushOutcome {
    Persisted,
    Ignored,
}

#[async_trait]
pub trait SessionRepository {
    async fn insert_session(&self, session: &ClosedSession) -> Result<(), RepositoryError>;
    async fn list_sessions(&self) -> Result<Vec<ClosedSession>, RepositoryError>;
    async fn get_settings(&self) -> Result<tm_storage::Settings, RepositoryError>;
    async fn save_settings(&self, settings: &tm_storage::Settings) -> Result<(), RepositoryError>;
}

#[async_trait]
impl SessionRepository for SqliteRepository {
    async fn insert_session(&self, session: &ClosedSession) -> Result<(), RepositoryError> {
        SqliteRepository::insert_session(self, session).await
    }

    async fn list_sessions(&self) -> Result<Vec<ClosedSession>, RepositoryError> {
        SqliteRepository::list_sessions(self).await
    }

    async fn get_settings(&self) -> Result<tm_storage::Settings, RepositoryError> {
        SqliteRepository::get_settings(self).await
    }

    async fn save_settings(&self, settings: &tm_storage::Settings) -> Result<(), RepositoryError> {
        SqliteRepository::save_settings(self, settings).await
    }
}

pub struct SessionService<R = SqliteRepository> {
    accumulator: SessionAccumulator,
    repo: R,
}

impl<R> SessionService<R>
where
    R: SessionRepository,
{
    pub fn new(repo: R) -> Self {
        Self {
            accumulator: SessionAccumulator::default(),
            repo,
        }
    }

    pub async fn ingest(&mut self, event: ActivityEvent) -> Result<IngestOutcome, RepositoryError> {
        let had_active_session = self.accumulator.has_active_session();

        if let Some(closed) = self.accumulator.ingest(event) {
            self.repo.insert_session(&closed).await?;
            return Ok(IngestOutcome::Persisted);
        }

        if had_active_session {
            Ok(IngestOutcome::Ignored)
        } else {
            Ok(IngestOutcome::Buffered)
        }
    }

    pub async fn flush(
        &mut self,
        ended_at: DateTime<Utc>,
    ) -> Result<FlushOutcome, RepositoryError> {
        if let Some(session) = self.accumulator.flush(ended_at) {
            self.repo.insert_session(&session).await?;
            return Ok(FlushOutcome::Persisted);
        }

        Ok(FlushOutcome::Ignored)
    }

    pub async fn list_sessions(&self) -> Result<Vec<ClosedSession>, RepositoryError> {
        self.repo.list_sessions().await
    }
}
