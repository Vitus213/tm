use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleTransitionKind {
    BecameIdle,
    BecameActive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleTransition {
    pub kind: IdleTransitionKind,
    pub occurred_at: DateTime<Utc>,
}
