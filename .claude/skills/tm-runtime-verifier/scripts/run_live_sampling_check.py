#!/usr/bin/env python3
import argparse
import os
import tempfile
from pathlib import Path

from common import (
    daemon_env,
    default_socket_path,
    link_wayland_socket,
    ping,
    repo_root,
    start_daemon,
    stop_process,
    wait_for_socket,
    write_json,
)


def build_report(dry_run: bool) -> dict:
    return {
        "status": "BLOCKED" if dry_run else "FAIL",
        "mode": "live-sampling",
        "checks": {
            "daemon_started": "not-run" if dry_run else "pending",
            "socket_ping": "not-run" if dry_run else "pending",
            "sampling_observed": "not-run" if dry_run else "blocked",
        },
        "artifacts": {
            "report": None,
            "screenshot": None,
            "daemon_log": None,
            "ipc_payload": None,
        },
        "environment": {
            "cwd": os.getcwd(),
            "wayland_display": os.environ.get("WAYLAND_DISPLAY"),
            "display": os.environ.get("DISPLAY"),
            "xdg_runtime_dir": os.environ.get("XDG_RUNTIME_DIR"),
            "dry_run": dry_run,
        },
        "notes": [
            "dry-run mode only validates report generation shape"
            if dry_run
            else "real live sampling started"
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", required=True)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    report = build_report(args.dry_run)
    report_path = Path(args.report)
    report["artifacts"]["report"] = str(report_path)

    if args.dry_run:
        write_json(report_path, report)
        return 0

    repo = repo_root()
    work_root = Path(tempfile.mkdtemp(prefix="tmvr-live-", dir="/tmp"))
    data_home = work_root / "data"
    isolated_runtime_dir = work_root / "rt"
    logs_dir = work_root / "logs"
    daemon_log = logs_dir / "tm-daemon.log"
    report["artifacts"]["daemon_log"] = str(daemon_log)

    link_wayland_socket(isolated_runtime_dir)
    env = daemon_env(data_home, isolated_runtime_dir)
    sock = default_socket_path(isolated_runtime_dir)
    report["environment"]["socket_path"] = str(sock)
    report["environment"]["db_path"] = str(data_home / "tm" / "tm.db")

    daemon = start_daemon(repo, env, daemon_log)
    try:
        report["checks"]["daemon_started"] = "pass"
        wait_for_socket(sock)
        pong = ping(sock)
        report["artifacts"]["ipc_payload"] = pong
        report["checks"]["socket_ping"] = "pass"
        report["status"] = "BLOCKED"
        report["notes"] = [
            "Live daemon startup and socket ping succeeded.",
            "Real focus-sampling observation is not scripted yet in this version.",
        ]
    finally:
        stop_process(daemon)

    write_json(report_path, report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
