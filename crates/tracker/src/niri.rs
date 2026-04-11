use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use niri_ipc::{socket::Socket, Request, Response};
use tm_core::ActivityEvent;

#[derive(Debug, Clone)]
pub struct FocusedWindowSnapshot {
    pub app_id: String,
    pub title: String,
    pub pid: Option<u32>,
    pub observed_at: DateTime<Utc>,
}

pub fn map_snapshot_to_event(snapshot: FocusedWindowSnapshot) -> ActivityEvent {
    ActivityEvent::app_focus(&snapshot.app_id, &snapshot.title, snapshot.observed_at)
}

pub fn focused_window_once() -> Result<Option<FocusedWindowSnapshot>> {
    let mut socket = Socket::connect()?;
    let reply = socket.send(Request::FocusedWindow)?;

    let window = match reply {
        Ok(Response::FocusedWindow(window)) => window,
        Ok(other) => return Err(anyhow!("unexpected niri response: {other:?}")),
        Err(message) => return Err(anyhow!("niri returned an error: {message}")),
    };

    Ok(window.map(|window| FocusedWindowSnapshot {
        app_id: window.app_id.unwrap_or_else(|| "unknown".to_owned()),
        title: window.title.unwrap_or_default(),
        pid: window.pid.and_then(|pid| u32::try_from(pid).ok()),
        observed_at: Utc::now(),
    }))
}
