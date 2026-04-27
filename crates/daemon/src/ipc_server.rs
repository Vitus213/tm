use std::path::PathBuf;

use anyhow::Result;
use tm_ipc::{DaemonRequest, DaemonResponse, default_socket_path};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::oneshot,
};

use crate::QueryService;

pub async fn bind_listener() -> Result<(UnixListener, PathBuf)> {
    let socket_path = default_socket_path().map_err(anyhow::Error::msg)?;
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    Ok((UnixListener::bind(&socket_path)?, socket_path))
}

pub async fn run_ipc_server<R>(
    listener: UnixListener,
    query_service: QueryService<R>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()>
where
    R: crate::SessionRepository + Clone + Send + Sync + 'static,
{
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => return Ok(()),
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let service = QueryService::new(query_service.repo().clone());
                tokio::spawn(async move {
                    let _ = handle_client(stream, service).await;
                });
            }
        }
    }
}

async fn handle_client<R>(stream: UnixStream, query_service: QueryService<R>) -> Result<()>
where
    R: crate::SessionRepository,
{
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let request: DaemonRequest = serde_json::from_str(line.trim_end())?;
    let response = match request {
        DaemonRequest::Ping => DaemonResponse::Pong,
        DaemonRequest::GetOverview(query) => {
            DaemonResponse::Overview(query_service.get_overview(query).await?)
        }
        DaemonRequest::GetSessions(query) => {
            DaemonResponse::Sessions(query_service.get_sessions(query).await?)
        }
        DaemonRequest::GetCharts(query) => {
            DaemonResponse::Charts(query_service.get_charts(query).await?)
        }
        DaemonRequest::GetSettings => DaemonResponse::Settings(query_service.get_settings().await?),
        DaemonRequest::UpdateSettings(settings) => {
            query_service.update_settings(settings).await?;
            DaemonResponse::Settings(query_service.get_settings().await?)
        }
    };

    writer
        .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
        .await?;
    Ok(())
}
