use std::path::PathBuf;

use tm_ipc::socket_path_from_env;

#[test]
fn socket_path_prefers_xdg_runtime_dir() {
    let path = socket_path_from_env(
        Some(PathBuf::from("/tmp/tm-runtime")),
        Some(PathBuf::from("/tmp/tm-home")),
    )
    .unwrap();

    assert_eq!(path, PathBuf::from("/tmp/tm-runtime/tm/tm.sock"));
}

#[test]
fn socket_path_falls_back_to_home_local_state() {
    let path = socket_path_from_env(None, Some(PathBuf::from("/tmp/tm-home"))).unwrap();

    assert_eq!(path, PathBuf::from("/tmp/tm-home/.local/state/tm/tm.sock"));
}
