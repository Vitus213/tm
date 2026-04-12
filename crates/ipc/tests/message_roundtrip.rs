use serde_json::json;
use tm_ipc::{DaemonCommand, DaemonEvent};

#[test]
fn serializes_command_and_event_to_explicit_tagged_wire_format() {
    let cmd = serde_json::to_value(&DaemonCommand::FlushActiveSession).unwrap();
    let evt = serde_json::to_value(&DaemonEvent::Ack).unwrap();

    assert_eq!(cmd, json!({ "type": "flush_active_session" }));
    assert_eq!(evt, json!({ "type": "ack" }));
}

#[test]
fn roundtrips_command_and_event_through_json() {
    let cmd_json = serde_json::to_string(&DaemonCommand::FlushActiveSession).unwrap();
    let evt_json = serde_json::to_string(&DaemonEvent::Ack).unwrap();

    let cmd: DaemonCommand = serde_json::from_str(&cmd_json).unwrap();
    let evt: DaemonEvent = serde_json::from_str(&evt_json).unwrap();

    assert_eq!(cmd, DaemonCommand::FlushActiveSession);
    assert_eq!(evt, DaemonEvent::Ack);
}
