#!/usr/bin/env python3
import json
import os
import signal
import socket
import sqlite3
import subprocess
import time
from pathlib import Path
from typing import Iterable


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def real_runtime_dir() -> Path:
    value = os.environ.get("XDG_RUNTIME_DIR")
    if not value:
        raise RuntimeError("XDG_RUNTIME_DIR is not set")
    return Path(value)


def wayland_display() -> str:
    return os.environ.get("WAYLAND_DISPLAY", "wayland-1")


def niri_socket() -> str | None:
    return os.environ.get("NIRI_SOCKET")


def format_duration_minutes_style(seconds: int) -> str:
    minutes = seconds // 60
    if minutes < 60:
        return f"{minutes} min"
    hours = minutes // 60
    remaining_minutes = minutes % 60
    return f"{hours}h {remaining_minutes}m"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2))


def link_wayland_socket(isolated_runtime_dir: Path) -> Path:
    isolated_runtime_dir.mkdir(parents=True, exist_ok=True)
    target = real_runtime_dir() / wayland_display()
    link = isolated_runtime_dir / wayland_display()
    if link.exists() or link.is_symlink():
        link.unlink()
    link.symlink_to(target)
    return target


def daemon_env(data_home: Path, isolated_runtime_dir: Path) -> dict[str, str]:
    env = os.environ.copy()
    env["XDG_DATA_HOME"] = str(data_home)
    env["XDG_RUNTIME_DIR"] = str(isolated_runtime_dir)
    env["WAYLAND_DISPLAY"] = wayland_display()
    env.setdefault("HOME", str(Path.home()))
    if niri_socket():
        env["NIRI_SOCKET"] = niri_socket()
    return env


def default_socket_path(isolated_runtime_dir: Path) -> Path:
    return isolated_runtime_dir / "tm" / "tm.sock"


def seed_sessions(db_path: Path, rows: Iterable[tuple[str, str, str, str, str, int]]) -> None:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL CHECK (kind IN ('app', 'website')), subject_id TEXT NOT NULL, title TEXT NOT NULL, started_at TEXT NOT NULL, ended_at TEXT NOT NULL, duration_seconds INTEGER NOT NULL CHECK (duration_seconds >= 0))"
    )
    conn.execute("DELETE FROM sessions")
    conn.executemany(
        "INSERT INTO sessions (kind, subject_id, title, started_at, ended_at, duration_seconds) VALUES (?, ?, ?, ?, ?, ?)",
        list(rows),
    )
    conn.commit()
    conn.close()


def start_daemon(repo: Path, env: dict[str, str], log_path: Path) -> subprocess.Popen[str]:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_file = log_path.open("w")
    return subprocess.Popen(
        [str(repo / "target" / "debug" / "tm-daemon")],
        cwd=repo,
        env=env,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )


def wait_for_socket(sock: Path, timeout_seconds: float = 10.0) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if sock.exists():
            return
        time.sleep(0.1)
    raise RuntimeError(f"socket did not appear: {sock}")


def send_request(sock: Path, payload: dict) -> dict:
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client.connect(str(sock))
    client.sendall((json.dumps(payload) + "\n").encode())
    response = client.recv(65536).decode().strip()
    client.close()
    return json.loads(response)


def ping(sock: Path) -> dict:
    return send_request(sock, {"type": "ping"})


def existing_tm_pids() -> set[int]:
    windows = json.loads(subprocess.check_output(["niri", "msg", "--json", "windows"], text=True))
    return {window["pid"] for window in windows if window.get("title") == "tm" and window.get("pid")}


def spawn_ui(repo: Path, env: dict[str, str], ui_log: Path) -> None:
    ui_log.parent.mkdir(parents=True, exist_ok=True)
    env_prefix = [
        f'XDG_RUNTIME_DIR="{env["XDG_RUNTIME_DIR"]}"',
        f'XDG_DATA_HOME="{env["XDG_DATA_HOME"]}"',
        f'WAYLAND_DISPLAY="{env["WAYLAND_DISPLAY"]}"',
        f'HOME="{env["HOME"]}"',
    ]
    if env.get("NIRI_SOCKET"):
        env_prefix.append(f'NIRI_SOCKET="{env["NIRI_SOCKET"]}"')
    command = (
        "export "
        + " ".join(env_prefix)
        + "; "
        + f'exec nix develop "{repo}" -c "{repo / "target" / "debug" / "tm-ui"}" >>"{ui_log}" 2>&1'
    )
    subprocess.run(["niri", "msg", "action", "spawn-sh", "--", command], check=True, text=True)


def wait_for_new_tm_window(existing_pids: set[int], timeout_seconds: float = 30.0) -> dict:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        windows = json.loads(subprocess.check_output(["niri", "msg", "--json", "windows"], text=True))
        for window in windows:
            if window.get("title") == "tm" and window.get("pid") not in existing_pids:
                return window
        time.sleep(0.25)
    raise RuntimeError("new tm window not found")


def focus_and_capture(window_id: int, screenshot_path: Path) -> None:
    screenshot_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["niri", "msg", "action", "focus-window", "--id", str(window_id)], check=True, text=True)
    time.sleep(1)
    subprocess.run(
        ["niri", "msg", "action", "screenshot-window", "--id", str(window_id), "--path", str(screenshot_path)],
        check=True,
        text=True,
    )


def close_window(window_id: int) -> None:
    subprocess.run(["niri", "msg", "action", "focus-window", "--id", str(window_id)], check=True, text=True)
    subprocess.run(["niri", "msg", "action", "close-window"], check=True, text=True)


def stop_process(proc: subprocess.Popen[str]) -> None:
    if proc.poll() is None:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)
