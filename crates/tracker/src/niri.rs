use std::{borrow::Cow, io};

use chrono::{DateTime, Utc};
use niri_ipc::{Reply, Request, Response, Window, socket::Socket};
use thiserror::Error;
use tm_core::ActivityEvent;

#[derive(Debug, Clone)]
pub struct FocusedWindowSnapshot {
    pub window_id: u64,
    pub app_id: Option<String>,
    pub title: String,
    pub pid: Option<u32>,
    pub observed_at: DateTime<Utc>,
}

impl FocusedWindowSnapshot {
    pub fn subject_id(&self) -> Cow<'_, str> {
        self.app_id
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(format!("niri-window:{}", self.window_id)))
    }
}

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("failed to communicate with niri: {0}")]
    NiriIo(#[source] io::Error),
    #[error("niri returned an error: {0}")]
    Niri(String),
    #[error("unexpected niri response: {0:?}")]
    UnexpectedReply(Box<Response>),
    #[error("niri reported invalid pid: {0}")]
    InvalidPid(i32),
}

pub fn map_snapshot_to_event(snapshot: &FocusedWindowSnapshot) -> ActivityEvent {
    let subject_id = snapshot.subject_id();
    ActivityEvent::app_focus(subject_id.as_ref(), &snapshot.title, snapshot.observed_at)
}

pub fn focused_window_once() -> Result<Option<FocusedWindowSnapshot>, TrackerError> {
    let mut socket = Socket::connect().map_err(TrackerError::NiriIo)?;
    let reply = socket
        .send(Request::FocusedWindow)
        .map_err(TrackerError::NiriIo)?;

    snapshot_from_reply(reply, Utc::now())
}

fn snapshot_from_reply(
    reply: Reply,
    observed_at: DateTime<Utc>,
) -> Result<Option<FocusedWindowSnapshot>, TrackerError> {
    match reply {
        Ok(Response::FocusedWindow(window)) => window
            .map(|window| snapshot_from_window(window, observed_at))
            .transpose(),
        Ok(other) => Err(TrackerError::UnexpectedReply(Box::new(other))),
        Err(message) => Err(TrackerError::Niri(message)),
    }
}

fn snapshot_from_window(
    window: Window,
    observed_at: DateTime<Utc>,
) -> Result<FocusedWindowSnapshot, TrackerError> {
    Ok(FocusedWindowSnapshot {
        window_id: window.id,
        app_id: window.app_id,
        title: window.title.unwrap_or_default(),
        pid: convert_pid(window.pid)?,
        observed_at,
    })
}

fn convert_pid(pid: Option<i32>) -> Result<Option<u32>, TrackerError> {
    match pid {
        Some(pid) => Ok(Some(
            u32::try_from(pid).map_err(|_| TrackerError::InvalidPid(pid))?,
        )),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niri_ipc::WindowLayout;

    #[test]
    fn helper_defaults_missing_title_to_empty_string() {
        let observed_at = Utc::now();
        let snapshot = snapshot_from_window(
            sample_window(None, Some("firefox".into()), Some(77)),
            observed_at,
        )
        .expect("snapshot should build");

        assert_eq!(snapshot.title, "");
        assert_eq!(snapshot.observed_at, observed_at);
    }

    #[test]
    fn helper_converts_valid_pid_and_rejects_invalid_pid() {
        let observed_at = Utc::now();

        let positive = snapshot_from_window(
            sample_window(Some("Title".into()), Some("firefox".into()), Some(77)),
            observed_at,
        )
        .expect("positive pid should convert");
        let absent = snapshot_from_window(
            sample_window(Some("Title".into()), Some("firefox".into()), None),
            observed_at,
        )
        .expect("missing pid should stay absent");
        let error = snapshot_from_window(
            sample_window(Some("Title".into()), Some("firefox".into()), Some(-7)),
            observed_at,
        )
        .expect_err("negative pid should be rejected");

        assert_eq!(positive.pid, Some(77));
        assert_eq!(absent.pid, None);
        match error {
            TrackerError::InvalidPid(pid) => assert_eq!(pid, -7),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn helper_rejects_unexpected_replies() {
        let observed_at = Utc::now();
        let error = snapshot_from_reply(Ok(Response::Version("25.0".into())), observed_at)
            .expect_err("unexpected reply should fail");

        match error {
            TrackerError::UnexpectedReply(reply) => match *reply {
                Response::Version(version) => assert_eq!(version, "25.0"),
                other => panic!("unexpected response payload: {other:?}"),
            },
            other => panic!("unexpected error: {other:?}"),
        }
    }

    fn sample_window(title: Option<String>, app_id: Option<String>, pid: Option<i32>) -> Window {
        Window {
            id: 17,
            title,
            app_id,
            pid,
            workspace_id: Some(2),
            is_focused: true,
            is_floating: false,
            is_urgent: false,
            layout: WindowLayout {
                pos_in_scrolling_layout: Some((1, 1)),
                tile_size: (1200.0, 800.0),
                window_size: (1200, 800),
                tile_pos_in_workspace_view: Some((0.0, 0.0)),
                window_offset_in_tile: (0.0, 0.0),
            },
            focus_timestamp: None,
        }
    }
}
