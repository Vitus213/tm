use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityKind {
    App,
    Website,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
}

impl ActivityEvent {
    pub fn app_focus(subject_id: &str, title: &str, occurred_at: DateTime<Utc>) -> Self {
        Self {
            kind: ActivityKind::App,
            subject_id: subject_id.to_owned(),
            title: title.to_owned(),
            occurred_at,
        }
    }
}
