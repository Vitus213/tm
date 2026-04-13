use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

use tm_ipc::{DaemonRequest, DaemonResponse, default_socket_path};

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub fn from_default_socket() -> Result<Self, String> {
        Ok(Self::new(default_socket_path()?))
    }

    pub fn send(&self, request: DaemonRequest) -> Result<DaemonResponse, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|err| format!("failed to connect to tm-daemon: {err}"))?;
        let payload = serde_json::to_string(&request).map_err(|err| err.to_string())?;
        stream
            .write_all(format!("{payload}\n").as_bytes())
            .map_err(|err| err.to_string())?;

        let mut line = String::new();
        BufReader::new(stream)
            .read_line(&mut line)
            .map_err(|err| err.to_string())?;

        serde_json::from_str(line.trim_end()).map_err(|err| err.to_string())
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}
