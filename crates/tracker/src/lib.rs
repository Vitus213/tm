mod idle;
mod niri;

pub use idle::IdleState;
pub use niri::{FocusedWindowSnapshot, TrackerError, focused_window_once, map_snapshot_to_event};

pub fn tracker_ready() -> &'static str {
    "tm-tracker-ready"
}
