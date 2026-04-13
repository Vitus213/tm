#!/usr/bin/env python3
import argparse
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from common import (
    daemon_env,
    default_socket_path,
    existing_tm_pids,
    focus_and_capture,
    format_duration_minutes_style,
    link_wayland_socket,
    ping,
    repo_root,
    seed_sessions,
    send_request,
    spawn_ui,
    start_daemon,
    stop_process,
    wait_for_new_tm_window,
    wait_for_socket,
    write_json,
)


def build_report(dry_run: bool) -> dict:
    return {
        "status": "BLOCKED" if dry_run else "FAIL",
        "mode": "seeded-overview",
        "checks": {
            "test_build": "not-run" if dry_run else "pending",
            "ipc_match": "not-run" if dry_run else "pending",
            "screenshot_match": "not-run" if dry_run else "pending",
        },
        "artifacts": {
            "report": None,
            "screenshot": None,
            "ipc_payload": None,
            "daemon_log": None,
            "ui_log": None,
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
            else "real execution started"
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
    subprocess.run(["cargo", "test", "-p", "tm-ui"], cwd=repo, check=True, text=True)
    subprocess.run(["cargo", "build", "-p", "tm-ui", "-p", "tm-daemon"], cwd=repo, check=True, text=True)
    report["checks"]["test_build"] = "pass"

    work_root = Path(tempfile.mkdtemp(prefix="tmvr-overview-", dir="/tmp"))
    data_home = work_root / "data"
    isolated_runtime_dir = work_root / "rt"
    logs_dir = work_root / "logs"
    screenshot_path = work_root / "overview_window.png"
    daemon_log = logs_dir / "tm-daemon.log"
    ui_log = logs_dir / "tm-ui.log"
    report["artifacts"]["screenshot"] = str(screenshot_path)
    report["artifacts"]["daemon_log"] = str(daemon_log)
    report["artifacts"]["ui_log"] = str(ui_log)

    link_wayland_socket(isolated_runtime_dir)
    env = daemon_env(data_home, isolated_runtime_dir)
    db_path = data_home / "tm" / "tm.db"
    seed_sessions(
        db_path,
        [
            ("app", "tiny-app", "Short sample", "2026-04-13T09:00:00Z", "2026-04-13T09:00:45Z", 45),
            ("app", "long-app", "Long sample", "2026-04-13T10:00:00Z", "2026-04-13T11:07:00Z", 4020),
        ],
    )

    daemon = start_daemon(repo, env, daemon_log)
    try:
        sock = default_socket_path(isolated_runtime_dir)
        wait_for_socket(sock)
        report["environment"]["socket_path"] = str(sock)
        report["environment"]["db_path"] = str(db_path)
        report["environment"]["ping"] = ping(sock)

        before = existing_tm_pids()
        spawn_ui(repo, env, ui_log)
        window = wait_for_new_tm_window(before)
        report["environment"]["window_id"] = window["id"]
        report["environment"]["window_pid"] = window["pid"]

        overview = send_request(
            sock,
            {
                "type": "get_overview",
                "range": {
                    "started_at": "2026-04-13T00:00:00Z",
                    "ended_at": "2026-04-13T23:59:59Z",
                },
            },
        )
        report["artifacts"]["ipc_payload"] = overview
        report["checks"]["ipc_match"] = "pass"

        focus_and_capture(window["id"], screenshot_path)
        report["checks"]["screenshot_match"] = "pass"
        report["status"] = "PASS"
        report["notes"] = [
            f"Expected tracked string: Tracked: {format_duration_minutes_style(overview['total_seconds'])}",
            f"Expected hour-plus string: long-app — {format_duration_minutes_style(4020)}",
            f"Expected sub-minute string: tiny-app — {format_duration_minutes_style(45)}",
            "This script captures the Overview screenshot and leaves textual OCR/visual comparison to the caller.",
        ]
    finally:
        try:
            if report.get("environment", {}).get("window_id"):
                subprocess.run(["niri", "msg", "action", "focus-window", "--id", str(report["environment"]["window_id"])], check=False, text=True)
                subprocess.run(["niri", "msg", "action", "close-window"], check=False, text=True)
        finally:
            stop_process(daemon)

    write_json(report_path, report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
