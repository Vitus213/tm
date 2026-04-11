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

impl ClosedSession {
    pub fn new(
        kind: ActivityKind,
        subject_id: String,
        title: String,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
    ) -> Option<Self> {
        let duration_seconds = (ended_at - started_at).num_seconds();
        (duration_seconds >= 0).then_some(Self {
            kind,
            subject_id,
            title,
            started_at,
            ended_at,
            duration_seconds,
        })
    }

    fn from_event(event: ActivityEvent, ended_at: DateTime<Utc>) -> Option<Self> {
        Self::new(
            event.kind,
            event.subject_id,
            event.title,
            event.occurred_at,
            ended_at,
        )
    }
}

#[derive(Debug, Default)]
pub struct SessionAccumulator {
    current: Option<ActivityEvent>,
}

impl SessionAccumulator {
    pub fn ingest(&mut self, next: ActivityEvent) -> Option<ClosedSession> {
        match self.current.take() {
            None => {
                self.current = Some(next);
                None
            }
            Some(previous) => match ClosedSession::from_event(previous.clone(), next.occurred_at) {
                Some(closed) => {
                    self.current = Some(next);
                    Some(closed)
                }
                None => {
                    self.current = Some(previous);
                    None
                }
            },
        }
    }

    pub fn flush(&mut self, ended_at: DateTime<Utc>) -> Option<ClosedSession> {
        let previous = self.current.take()?;

        match ClosedSession::from_event(previous.clone(), ended_at) {
            Some(closed) => Some(closed),
            None => {
                self.current = Some(previous);
                None
            }
        }
    }
}
