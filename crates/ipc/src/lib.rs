mod messages;

pub use messages::{DaemonCommand, DaemonEvent};

pub fn ipc_ready() -> &'static str {
    "tm-ipc-ready"
}
