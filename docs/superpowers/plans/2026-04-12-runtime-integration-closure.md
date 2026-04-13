# TM Runtime Integration Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the foundation by wiring Niri focus polling into the daemon runtime, persisting sessions to a file-backed SQLite database, and flushing the final active session on shutdown.

**Architecture:** Keep the existing crate boundaries. Extend `tm-storage` with a file-backed repository constructor, extend `tm-tracker` with a poll-friendly focus source API, and make `tm-daemon` own the runtime loop: poll Niri focused window, normalize to `ActivityEvent`, feed `SessionService`, and flush on shutdown. This plan intentionally stops before browser IPC or UI integration.

**Tech Stack:** Rust 2024, `tokio`, `chrono`, `sqlx` + SQLite, `niri-ipc`, Unix runtime file paths.

---

## Scope

This plan closes the exact gaps found in the final foundation review:
1. runtime currently does not connect tracker -> daemon service
2. storage is only in-memory, not file-backed
3. shutdown does not flush the trailing active session

It does **not** add browser IPC transport, UI integration, idle-event runtime wiring, or export features.

## File Map

### Storage
- Modify: `crates/storage/src/lib.rs` — export new file-backed constructor support.
- Modify: `crates/storage/src/repository.rs` — add file-backed connection constructor and path bootstrap.
- Create: `crates/storage/tests/file_repository.rs` — verify persistence across reopen.

### Tracker
- Modify: `crates/tracker/src/lib.rs` — export polling helper(s) used by daemon runtime.
- Modify: `crates/tracker/src/niri.rs` — add small poll-oriented API seam without changing normalized mapping behavior.
- Create: `crates/tracker/tests/focus_polling.rs` — test new pure helper behavior for runtime polling decisions.

### Daemon runtime
- Modify: `crates/daemon/src/app.rs` — implement real runtime loop, service creation, shutdown flush, and file-backed repo path selection.
- Modify: `crates/daemon/src/main.rs` — keep thin runtime entrypoint only.
- Modify: `crates/daemon/src/session_service.rs` — keep current service API, add only the small pieces needed by runtime loop if required.
- Create: `crates/daemon/tests/runtime_shutdown.rs` — integration tests for shutdown flush and runtime composition helpers.

## Implementation Rules
- Keep runtime integration in `tm-daemon`; do not move tracker logic into storage or core.
- Keep SQLite path handling explicit and local; do not add config systems yet.
- Prefer pure helper extraction for logic that would otherwise require a live Niri compositor in tests.
- Runtime must honestly persist to disk in this phase; no more in-memory default for the daemon binary.
- Shutdown must flush the active session exactly once.

## Task 1: Add file-backed SQLite repository support

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/repository.rs`
- Create: `crates/storage/tests/file_repository.rs`

- [ ] **Step 1: Write the failing persistence test**

```rust
// crates/storage/tests/file_repository.rs
use chrono::{TimeZone, Utc};
use tempfile::tempdir;
use tm_core::{ActivityKind, ClosedSession};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn file_backed_repository_persists_sessions_across_reopen() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("tm.db");

    {
        let repo = SqliteRepository::open_at(&db_path).await.unwrap();
        let session = ClosedSession::new(
            ActivityKind::App,
            "firefox".into(),
            "Rust docs".into(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
        )
        .unwrap();
        repo.insert_session(&session).await.unwrap();
    }

    let reopened = SqliteRepository::open_at(&db_path).await.unwrap();
    let sessions = reopened.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id(), "firefox");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-storage file_backed_repository_persists_sessions_across_reopen -- --exact`
Expected: FAIL because `open_at` does not exist yet.

- [ ] **Step 3: Add the file-backed constructor**

```rust
// crates/storage/src/repository.rs
use std::path::Path;

impl SqliteRepository {
    pub async fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(sqlx::Error::Io)?;
        }

        let url = format!("sqlite://{}", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await?;

        sqlx::query(BOOTSTRAP_SQL).execute(&pool).await?;
        Ok(Self { pool })
    }
}
```

- [ ] **Step 4: Add any small dependency/export updates**

```rust
// crates/storage/src/lib.rs
mod repository;
mod schema;

pub use repository::{RepositoryError, SqliteRepository};
```

```toml
# crates/storage/Cargo.toml
[dev-dependencies]
tempfile = "3"
tokio.workspace = true
```

- [ ] **Step 5: Run the storage tests**

Run: `cargo test -p tm-storage --test file_repository && cargo test -p tm-storage --test sqlite_repository`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/storage/Cargo.toml crates/storage/src/lib.rs crates/storage/src/repository.rs crates/storage/tests/file_repository.rs
git commit -m "feat: add file-backed sqlite repository"
```

## Task 2: Add poll-friendly tracker helpers for daemon runtime

**Files:**
- Modify: `crates/tracker/src/lib.rs`
- Modify: `crates/tracker/src/niri.rs`
- Create: `crates/tracker/tests/focus_polling.rs`

- [ ] **Step 1: Write the failing polling helper test**

```rust
// crates/tracker/tests/focus_polling.rs
use chrono::{TimeZone, Utc};
use tm_tracker::{should_emit_focus_event, FocusedWindowSnapshot};

#[test]
fn unchanged_focus_snapshot_does_not_emit_duplicate_event() {
    let observed_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();
    let previous = FocusedWindowSnapshot {
        window_id: 17,
        app_id: Some("firefox".into()),
        title: "Rust docs".into(),
        pid: Some(1000),
        observed_at,
    };
    let current = FocusedWindowSnapshot {
        observed_at: observed_at + chrono::TimeDelta::seconds(5),
        ..previous.clone()
    };

    assert!(!should_emit_focus_event(Some(&previous), &current));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-tracker unchanged_focus_snapshot_does_not_emit_duplicate_event -- --exact`
Expected: FAIL because `should_emit_focus_event` does not exist.

- [ ] **Step 3: Add a pure focus-change helper**

```rust
// crates/tracker/src/niri.rs
pub fn should_emit_focus_event(
    previous: Option<&FocusedWindowSnapshot>,
    current: &FocusedWindowSnapshot,
) -> bool {
    match previous {
        None => true,
        Some(previous) => {
            previous.window_id != current.window_id
                || previous.title != current.title
                || previous.app_id != current.app_id
        }
    }
}
```

- [ ] **Step 4: Export the helper**

```rust
// crates/tracker/src/lib.rs
mod idle;
mod niri;

pub use idle::IdleState;
pub use niri::{
    focused_window_once, map_snapshot_to_event, should_emit_focus_event, FocusedWindowSnapshot,
    TrackerError,
};
```

- [ ] **Step 5: Add a complementary positive-path test**

```rust
#[test]
fn changed_focus_snapshot_emits_event() {
    let observed_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();
    let previous = FocusedWindowSnapshot {
        window_id: 17,
        app_id: Some("firefox".into()),
        title: "Rust docs".into(),
        pid: Some(1000),
        observed_at,
    };
    let current = FocusedWindowSnapshot {
        window_id: 18,
        app_id: Some("wezterm".into()),
        title: "shell".into(),
        pid: Some(2000),
        observed_at: observed_at + chrono::TimeDelta::seconds(5),
    };

    assert!(should_emit_focus_event(Some(&previous), &current));
}
```

- [ ] **Step 6: Run tracker tests**

Run: `cargo test -p tm-tracker --test focus_polling && cargo test -p tm-tracker --test niri_mapping`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/tracker/src/lib.rs crates/tracker/src/niri.rs crates/tracker/tests/focus_polling.rs
git commit -m "feat: add tracker polling helpers"
```

## Task 3: Wire tracker polling into daemon runtime and persist to file SQLite

**Files:**
- Modify: `crates/daemon/src/app.rs`
- Modify: `crates/daemon/src/session_service.rs`
- Create: `crates/daemon/tests/runtime_shutdown.rs`

- [ ] **Step 1: Write the failing shutdown-flush integration test**

```rust
// crates/daemon/tests/runtime_shutdown.rs
use chrono::{TimeZone, Utc};
use tm_core::ActivityEvent;
use tm_daemon::{FlushOutcome, SessionService};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn flush_persists_trailing_active_session() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    service
        .ingest(ActivityEvent::app_focus(
            "firefox",
            "Rust docs",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ))
        .await
        .unwrap();

    let flushed = service
        .flush(Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap())
        .await
        .unwrap();

    assert_eq!(flushed, FlushOutcome::Persisted);
    assert_eq!(service.list_sessions().await.unwrap().len(), 1);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-daemon flush_persists_trailing_active_session -- --exact`
Expected: FAIL until daemon-level flush outcome handling and assertions match the runtime integration path.

- [ ] **Step 3: Add a daemon runtime loop helper**

```rust
// crates/daemon/src/app.rs
use std::{path::PathBuf, time::Duration};

use chrono::Utc;
use tm_tracker::{focused_window_once, map_snapshot_to_event, should_emit_focus_event, FocusedWindowSnapshot};
use tm_storage::SqliteRepository;

use crate::session_service::{IngestOutcome, SessionService};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn default_db_path() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut path = PathBuf::from(std::env::var_os("HOME").expect("HOME must be set"));
            path.push(".local/share");
            path
        })
        .join("tm/tm.db")
}

pub async fn run() -> anyhow::Result<()> {
    let repo = SqliteRepository::open_at(default_db_path()).await?;
    let mut service = SessionService::new(repo);
    let mut previous_focus: Option<FocusedWindowSnapshot> = None;

    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                let _ = service.flush(Utc::now()).await?;
                break;
            }
            _ = tokio::time::sleep(POLL_INTERVAL) => {
                if let Some(current) = focused_window_once()? {
                    if should_emit_focus_event(previous_focus.as_ref(), &current) {
                        let _ = service.ingest(map_snapshot_to_event(&current)).await?;
                        previous_focus = Some(current);
                    }
                }
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Add a small service helper only if required**

```rust
// crates/daemon/src/session_service.rs
pub fn into_inner(self) -> R {
    self.repo
}
```

Use this step only if the runtime or tests need to recover the repository after service ownership; otherwise skip it.

- [ ] **Step 5: Add a focused runtime helper test**

```rust
#[test]
fn default_db_path_uses_xdg_data_home_when_present() {
    std::env::set_var("XDG_DATA_HOME", "/tmp/tm-xdg-test");
    let path = tm_daemon::default_db_path();
    assert_eq!(path, std::path::PathBuf::from("/tmp/tm-xdg-test/tm/tm.db"));
}
```

If you expose `default_db_path()` for testing, keep it crate-visible rather than broad public API.

- [ ] **Step 6: Run daemon tests and smoke path**

Run: `cargo test -p tm-daemon --test runtime_shutdown && cargo test -p tm-daemon --test session_service && timeout -s INT 5 cargo run -p tm-daemon`
Expected: PASS; daemon starts, waits, flushes on SIGINT, and exits cleanly.

- [ ] **Step 7: Commit**

```bash
git add crates/daemon/src/app.rs crates/daemon/src/session_service.rs crates/daemon/tests/runtime_shutdown.rs
git commit -m "feat: wire daemon runtime to tracker and sqlite"
```

## Task 4: Re-run full foundation validation with persistent runtime

**Files:**
- Modify: no planned source files

- [ ] **Step 1: Run full workspace validation**

Run: `cargo test --workspace && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 2: Run focused end-to-end probe**

Run: `timeout -s INT 5 cargo run -p tm-daemon`
Expected: daemon starts, uses file-backed SQLite, waits for signal, flushes on shutdown, exits cleanly.

- [ ] **Step 3: Inspect resulting database file exists**

Run: `test -f "${XDG_DATA_HOME:-$HOME/.local/share}/tm/tm.db"`
Expected: exit code 0.

- [ ] **Step 4: Commit**

```bash
git add -u
git commit -m "chore: validate runtime integration closure"
```

## Spec Coverage Check
- runtime tracker -> daemon wiring: Task 3
- persistent file-backed SQLite: Task 1 + Task 3
- shutdown flush of trailing session: Task 3
- full revalidation after integration: Task 4

## Self-Review Notes
- This plan is intentionally narrow and only closes the three blockers from the final review.
- No UI, browser, idle runtime loop, or export work is included.
- Later implementation must keep using the hardened `ClosedSession`, `RepositoryError`, and `TrackerError` APIs introduced in the current branch.
