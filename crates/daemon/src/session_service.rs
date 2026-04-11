use chrono::{DateTime, Utc};
use tm_core::{ActivityEvent, ClosedSession, SessionAccumulator};
use tm_storage::SqliteRepository;

pub struct SessionService {
    accumulator: SessionAccumulator,
    repo: SqliteRepository,
}

impl SessionService {
    pub fn new(repo: SqliteRepository) -> Self {
        Self {
            accumulator: SessionAccumulator::default(),
            repo,
        }
    }

    pub async fn ingest(&mut self, event: ActivityEvent) -> Result<(), tm_storage::RepositoryError> {
        if let Some(closed) = self.accumulator.ingest(event) {
            self.repo.insert_session(&closed).await?;
        }

        Ok(())
    }

    pub async fn flush(
        &mut self,
        ended_at: DateTime<Utc>,
    ) -> Result<Option<ClosedSession>, tm_storage::RepositoryError> {
        let closed = self.accumulator.flush(ended_at);
        if let Some(ref session) = closed {
            self.repo.insert_session(session).await?;
        }

        Ok(closed)
    }

    pub async fn list_sessions(&self) -> Result<Vec<ClosedSession>, tm_storage::RepositoryError> {
        self.repo.list_sessions().await
    }
}
