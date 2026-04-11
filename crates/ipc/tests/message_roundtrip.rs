use tm_ipc::{DaemonCommand, DaemonEvent};

#[test]
fn serializes_flush_command_and_ack_event() {
    let cmd = serde_json::to_string(&DaemonCommand::FlushActiveSession).unwrap();
    let evt = serde_json::to_string(&DaemonEvent::Ack).unwrap();

    assert_eq!(cmd, "\"FlushActiveSession\"");
    assert_eq!(evt, "\"Ack\"");
}
