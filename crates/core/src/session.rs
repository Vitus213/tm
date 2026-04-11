use chrono::{DateTime, Utc};

use crate::activity::{ActivityEvent, ActivityKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedSession {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Default)]
pub struct SessionAccumulator {
    current: Option<ActivityEvent>,
}

impl SessionAccumulator {
    pub fn ingest(&mut self, next: ActivityEvent) -> Option<ClosedSession> {
        let previous = self.current.replace(next.clone())?;

        Some(ClosedSession {
            kind: previous.kind,
            subject_id: previous.subject_id,
            title: previous.title,
            started_at: previous.occurred_at,
            ended_at: next.occurred_at,
            duration_seconds: (next.occurred_at - previous.occurred_at).num_seconds(),
        })
    }

    pub fn flush(&mut self, ended_at: DateTime<Utc>) -> Option<ClosedSession> {
        let previous = self.current.take()?;

        Some(ClosedSession {
            kind: previous.kind,
            subject_id: previous.subject_id,
            title: previous.title,
            started_at: previous.occurred_at,
            ended_at,
            duration_seconds: (ended_at - previous.occurred_at).num_seconds(),
        })
    }
}
