use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_daemon::{QueryService, run_ipc_server};
use tm_ipc::{DaemonRequest, DaemonResponse, OverviewQuery, TimeRange};
use tm_storage::SqliteRepository;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::oneshot,
};

#[tokio::test]
async fn ipc_server_roundtrips_overview_requests() {
    let socket_path = unique_socket_path();
    let repo = SqliteRepository::in_memory().await.unwrap();
    repo.insert_session(
        &ClosedSession::new(
            ActivityKind::App,
            "wezterm".into(),
            "shell".into(),
            Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 4, 13, 9, 10, 0).unwrap(),
        )
        .unwrap(),
    )
    .await
    .unwrap();

    let listener = UnixListener::bind(&socket_path).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = tokio::spawn(run_ipc_server(
        listener,
        QueryService::new(repo),
        shutdown_rx,
    ));

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let request = serde_json::to_string(&DaemonRequest::GetOverview(OverviewQuery {
        range: TimeRange {
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
        },
    }))
    .unwrap();

    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    let response: DaemonResponse = serde_json::from_str(line.trim_end()).unwrap();
    match response {
        DaemonResponse::Overview(overview) => assert_eq!(overview.total_seconds, 600),
        other => panic!("expected overview response, got {other:?}"),
    }

    let _ = shutdown_tx.send(());
    server.await.unwrap().unwrap();
    let _ = std::fs::remove_file(socket_path);
}

fn unique_socket_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "tm-daemon-ipc-{}-{}.sock",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
