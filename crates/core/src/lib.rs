mod activity;
mod idle;
mod session;

pub use activity::{ActivityEvent, ActivityKind};
pub use idle::{IdleTransition, IdleTransitionKind};
pub use session::{ClosedSession, SessionAccumulator};

pub fn workspace_ready() -> &'static str {
    "tm-core-ready"
}
