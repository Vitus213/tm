use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    FlushActiveSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonEvent {
    Ack,
}
