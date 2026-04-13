use std::path::PathBuf;

pub fn default_socket_path() -> Result<PathBuf, String> {
    socket_path_from_env(
        std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

pub fn socket_path_from_env(
    xdg_runtime_dir: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(runtime_dir) = xdg_runtime_dir {
        return Ok(runtime_dir.join("tm").join("tm.sock"));
    }

    let mut home =
        home.ok_or_else(|| "HOME is not set; cannot resolve tm socket path".to_owned())?;
    home.push(".local");
    home.push("state");
    home.push("tm");
    home.push("tm.sock");
    Ok(home)
}
