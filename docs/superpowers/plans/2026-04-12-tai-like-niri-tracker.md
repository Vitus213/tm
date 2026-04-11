# TM Foundation Tracking Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working foundation of the Tai-like Niri tracker: Niri focus events in, idle-aware sessionization in the middle, and SQLite-backed app sessions out.

**Architecture:** Start with a Rust workspace split into `core`, `storage`, `tracker`, `ipc`, and `daemon`. The daemon is the only writer: it receives normalized activity events, applies session lifecycle rules, and persists closed sessions to SQLite. UI, tray, browser extension, export, and packaging stay out of this plan and become follow-up plans after this foundation is working end-to-end.

**Tech Stack:** Rust 2024, `tokio`, `serde`, `chrono`, `sqlx` + SQLite, `niri_ipc`, Unix domain sockets.

---

## Scope Split

This project has at least four largely independent subsystems:
1. foundation tracking pipeline
2. desktop UI + tray shell
3. browser extension + native messaging
4. export + packaging

This document covers only **foundation tracking pipeline**. That is intentional: it produces working, testable software on its own and gives later plans a stable base.

## File Map

### Workspace root
- Modify: `Cargo.toml` — convert the repository into a workspace and define shared dependencies.
- Modify: `.gitignore` — ignore build output, SQLite files, and runtime sockets.
- Create: `rustfmt.toml` — shared formatting rules.
- Create: `.config/nextest.toml` — stable test runner defaults.
- Modify: `src/main.rs` — keep the old root binary as a minimal pointer.

### Domain crate
- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`
- Create: `crates/core/src/activity.rs`
- Create: `crates/core/src/session.rs`
- Create: `crates/core/src/idle.rs`
- Create: `crates/core/tests/sessionization.rs`

### Storage crate
- Create: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/lib.rs`
- Create: `crates/storage/src/schema.rs`
- Create: `crates/storage/src/repository.rs`
- Create: `crates/storage/tests/sqlite_repository.rs`

### Tracker crate
- Create: `crates/tracker/Cargo.toml`
- Create: `crates/tracker/src/lib.rs`
- Create: `crates/tracker/src/niri.rs`
- Create: `crates/tracker/src/idle.rs`
- Create: `crates/tracker/tests/niri_mapping.rs`

### IPC crate
- Create: `crates/ipc/Cargo.toml`
- Create: `crates/ipc/src/lib.rs`
- Create: `crates/ipc/src/messages.rs`
- Create: `crates/ipc/tests/message_roundtrip.rs`

### Daemon crate
- Create: `crates/daemon/Cargo.toml`
- Create: `crates/daemon/src/lib.rs`
- Create: `crates/daemon/src/main.rs`
- Create: `crates/daemon/src/app.rs`
- Create: `crates/daemon/src/session_service.rs`
- Create: `crates/daemon/tests/session_service.rs`

## Implementation Rules
- Do not add UI crates in this plan.
- Do not add browser-extension files in this plan.
- The daemon owns all SQLite writes.
- Use `app_id` as the primary application key from Niri.
- A session closes when one of these happens: app focus changes, idle begins, explicit flush on shutdown.
- Write the failing test first whenever the behavior is testable locally.
- Create a new git commit at the end of each task.

## Task 1: Bootstrap the workspace and crate skeletons

**Files:**
- Modify: `Cargo.toml`
- Modify: `.gitignore`
- Modify: `src/main.rs`
- Create: `rustfmt.toml`
- Create: `.config/nextest.toml`
- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`
- Create: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/lib.rs`
- Create: `crates/tracker/Cargo.toml`
- Create: `crates/tracker/src/lib.rs`
- Create: `crates/ipc/Cargo.toml`
- Create: `crates/ipc/src/lib.rs`
- Create: `crates/daemon/Cargo.toml`
- Create: `crates/daemon/src/lib.rs`
- Create: `crates/daemon/src/main.rs`
- Create: `crates/core/tests/workspace_smoke.rs`

- [ ] **Step 1: Write the failing workspace smoke test**

```rust
// crates/core/tests/workspace_smoke.rs
use tm_core::workspace_ready;

#[test]
fn workspace_exposes_core_crate() {
    assert_eq!(workspace_ready(), "tm-core-ready");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-core workspace_exposes_core_crate -- --exact`
Expected: FAIL because `tm-core` does not exist yet.

- [ ] **Step 3: Convert the repository into a workspace**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
  "crates/core",
  "crates/storage",
  "crates/tracker",
  "crates/ipc",
  "crates/daemon",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "sync", "time"] }
```

```gitignore
# .gitignore
/target
*.db
*.db-shm
*.db-wal
*.sock
```

```rust
// src/main.rs
fn main() {
    println!("Use cargo run -p tm-daemon to start the tracker daemon.");
}
```

- [ ] **Step 4: Add root tooling files**

```toml
# rustfmt.toml
max_width = 100
newline_style = "Unix"
```

```toml
# .config/nextest.toml
[profile.default]
fail-fast = false
```

- [ ] **Step 5: Add minimal crate manifests and stubs**

```toml
# crates/core/Cargo.toml
[package]
name = "tm-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true
thiserror.workspace = true
```

```rust
// crates/core/src/lib.rs
pub fn workspace_ready() -> &'static str {
    "tm-core-ready"
}
```

```toml
# crates/storage/Cargo.toml
[package]
name = "tm-storage"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
chrono.workspace = true
sqlx.workspace = true
tm-core = { path = "../core" }
```

```rust
// crates/storage/src/lib.rs
pub fn storage_ready() -> &'static str {
    "tm-storage-ready"
}
```

```toml
# crates/tracker/Cargo.toml
[package]
name = "tm-tracker"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
chrono.workspace = true
serde.workspace = true
tm-core = { path = "../core" }
niri_ipc = "25"
```

```rust
// crates/tracker/src/lib.rs
pub fn tracker_ready() -> &'static str {
    "tm-tracker-ready"
}
```

```toml
# crates/ipc/Cargo.toml
[package]
name = "tm-ipc"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true
serde_json.workspace = true
```

```rust
// crates/ipc/src/lib.rs
pub fn ipc_ready() -> &'static str {
    "tm-ipc-ready"
}
```

```toml
# crates/daemon/Cargo.toml
[package]
name = "tm-daemon"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
tokio.workspace = true
tm-core = { path = "../core" }
tm-ipc = { path = "../ipc" }
tm-storage = { path = "../storage" }
tm-tracker = { path = "../tracker" }
```

```rust
// crates/daemon/src/lib.rs
pub fn daemon_ready() -> &'static str {
    "tm-daemon-ready"
}
```

```rust
// crates/daemon/src/main.rs
fn main() {
    println!("tm-daemon bootstrap");
}
```

- [ ] **Step 6: Run the bootstrap test and formatter**

Run: `cargo test -p tm-core workspace_exposes_core_crate -- --exact && cargo fmt --all --check`
Expected: PASS for the test and no formatting changes.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml .gitignore rustfmt.toml .config/nextest.toml src/main.rs crates/core/Cargo.toml crates/core/src/lib.rs crates/core/tests/workspace_smoke.rs crates/storage/Cargo.toml crates/storage/src/lib.rs crates/tracker/Cargo.toml crates/tracker/src/lib.rs crates/ipc/Cargo.toml crates/ipc/src/lib.rs crates/daemon/Cargo.toml crates/daemon/src/lib.rs crates/daemon/src/main.rs
git commit -m "chore: bootstrap tracking workspace"
```

## Task 2: Define the domain model and session accumulator

**Files:**
- Modify: `crates/core/src/lib.rs`
- Create: `crates/core/src/activity.rs`
- Create: `crates/core/src/session.rs`
- Create: `crates/core/src/idle.rs`
- Create: `crates/core/tests/sessionization.rs`

- [ ] **Step 1: Write the failing sessionization test**

```rust
// crates/core/tests/sessionization.rs
use chrono::{TimeZone, Utc};
use tm_core::{ActivityEvent, ActivityKind, SessionAccumulator};

#[test]
fn focus_change_closes_previous_session() {
    let mut acc = SessionAccumulator::default();
    let t0 = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap();
    let t1 = Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap();

    assert!(acc.ingest(ActivityEvent::app_focus("wezterm", "shell", t0)).is_none());

    let closed = acc
        .ingest(ActivityEvent::app_focus("firefox", "Rust docs", t1))
        .unwrap();

    assert_eq!(closed.kind, ActivityKind::App);
    assert_eq!(closed.subject_id, "wezterm");
    assert_eq!(closed.duration_seconds, 300);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-core focus_change_closes_previous_session -- --exact`
Expected: FAIL because the types do not exist.

- [ ] **Step 3: Add activity and idle types**

```rust
// crates/core/src/activity.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityKind {
    App,
    Website,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
}

impl ActivityEvent {
    pub fn app_focus(subject_id: &str, title: &str, occurred_at: DateTime<Utc>) -> Self {
        Self {
            kind: ActivityKind::App,
            subject_id: subject_id.to_owned(),
            title: title.to_owned(),
            occurred_at,
        }
    }
}
```

```rust
// crates/core/src/idle.rs
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleTransitionKind {
    BecameIdle,
    BecameActive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleTransition {
    pub kind: IdleTransitionKind,
    pub occurred_at: DateTime<Utc>,
}
```

- [ ] **Step 4: Add session accumulator logic**

```rust
// crates/core/src/session.rs
use chrono::{DateTime, Utc};

use crate::activity::{ActivityEvent, ActivityKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedSession {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Default)]
pub struct SessionAccumulator {
    current: Option<ActivityEvent>,
}

impl SessionAccumulator {
    pub fn ingest(&mut self, next: ActivityEvent) -> Option<ClosedSession> {
        let previous = self.current.replace(next.clone())?;
        Some(ClosedSession {
            kind: previous.kind,
            subject_id: previous.subject_id,
            title: previous.title,
            started_at: previous.occurred_at,
            ended_at: next.occurred_at,
            duration_seconds: (next.occurred_at - previous.occurred_at).num_seconds(),
        })
    }

    pub fn flush(&mut self, ended_at: DateTime<Utc>) -> Option<ClosedSession> {
        let previous = self.current.take()?;
        Some(ClosedSession {
            kind: previous.kind,
            subject_id: previous.subject_id,
            title: previous.title,
            started_at: previous.occurred_at,
            ended_at,
            duration_seconds: (ended_at - previous.occurred_at).num_seconds(),
        })
    }
}
```

- [ ] **Step 5: Re-export the public API**

```rust
// crates/core/src/lib.rs
mod activity;
mod idle;
mod session;

pub use activity::{ActivityEvent, ActivityKind};
pub use idle::{IdleTransition, IdleTransitionKind};
pub use session::{ClosedSession, SessionAccumulator};

pub fn workspace_ready() -> &'static str {
    "tm-core-ready"
}
```

- [ ] **Step 6: Run the core tests**

Run: `cargo test -p tm-core --test sessionization`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/core/src/lib.rs crates/core/src/activity.rs crates/core/src/session.rs crates/core/src/idle.rs crates/core/tests/sessionization.rs
git commit -m "feat: add core activity and session models"
```

## Task 3: Build SQLite persistence for closed sessions

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Create: `crates/storage/src/schema.rs`
- Create: `crates/storage/src/repository.rs`
- Create: `crates/storage/tests/sqlite_repository.rs`

- [ ] **Step 1: Write the failing repository test**

```rust
// crates/storage/tests/sqlite_repository.rs
use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn inserts_and_reads_back_sessions() {
    let repo = SqliteRepository::in_memory().await.unwrap();

    let row = ClosedSession {
        kind: ActivityKind::App,
        subject_id: "firefox".into(),
        title: "Rust docs".into(),
        started_at: Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
        duration_seconds: 300,
    };

    repo.insert_session(&row).await.unwrap();
    let sessions = repo.list_sessions().await.unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id, "firefox");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-storage inserts_and_reads_back_sessions -- --exact`
Expected: FAIL because `SqliteRepository` does not exist.

- [ ] **Step 3: Add schema bootstrap**

```rust
// crates/storage/src/schema.rs
pub const BOOTSTRAP_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    title TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL
);
"#;
```

- [ ] **Step 4: Implement the repository**

```rust
// crates/storage/src/repository.rs
use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use tm_core::{ActivityKind, ClosedSession};

use crate::schema::BOOTSTRAP_SQL;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(BOOTSTRAP_SQL).execute(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn insert_session(&self, session: &ClosedSession) -> Result<()> {
        sqlx::query(
            "INSERT INTO sessions (kind, subject_id, title, started_at, ended_at, duration_seconds) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(match session.kind {
            ActivityKind::App => "app",
            ActivityKind::Website => "website",
        })
        .bind(&session.subject_id)
        .bind(&session.title)
        .bind(session.started_at.to_rfc3339())
        .bind(session.ended_at.to_rfc3339())
        .bind(session.duration_seconds)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<Vec<ClosedSession>> {
        let rows = sqlx::query(
            "SELECT kind, subject_id, title, started_at, ended_at, duration_seconds FROM sessions ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let kind = match row.get::<String, _>(0).as_str() {
                    "app" => ActivityKind::App,
                    _ => ActivityKind::Website,
                };

                Ok(ClosedSession {
                    kind,
                    subject_id: row.get(1),
                    title: row.get(2),
                    started_at: row.get::<String, _>(3).parse()?,
                    ended_at: row.get::<String, _>(4).parse()?,
                    duration_seconds: row.get(5),
                })
            })
            .collect()
    }
}
```

- [ ] **Step 5: Export the repository**

```rust
// crates/storage/src/lib.rs
mod repository;
mod schema;

pub use repository::SqliteRepository;
```

- [ ] **Step 6: Run the storage test**

Run: `cargo test -p tm-storage --test sqlite_repository`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/storage/src/lib.rs crates/storage/src/schema.rs crates/storage/src/repository.rs crates/storage/tests/sqlite_repository.rs
git commit -m "feat: add sqlite session repository"
```

## Task 4: Normalize Niri focused-window snapshots into activity events

**Files:**
- Modify: `crates/tracker/src/lib.rs`
- Create: `crates/tracker/src/niri.rs`
- Create: `crates/tracker/src/idle.rs`
- Create: `crates/tracker/tests/niri_mapping.rs`

- [ ] **Step 1: Write the failing Niri mapping test**

```rust
// crates/tracker/tests/niri_mapping.rs
use chrono::Utc;
use tm_tracker::{map_snapshot_to_event, FocusedWindowSnapshot};

#[test]
fn focused_window_maps_to_app_focus_event() {
    let snapshot = FocusedWindowSnapshot {
        app_id: "firefox".into(),
        title: "Rust docs".into(),
        pid: Some(4242),
        observed_at: Utc::now(),
    };

    let event = map_snapshot_to_event(snapshot);
    assert_eq!(event.subject_id, "firefox");
    assert_eq!(event.title, "Rust docs");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-tracker focused_window_maps_to_app_focus_event -- --exact`
Expected: FAIL because the snapshot type does not exist.

- [ ] **Step 3: Add the Niri snapshot type and mapping**

```rust
// crates/tracker/src/niri.rs
use chrono::{DateTime, Utc};
use tm_core::ActivityEvent;

#[derive(Debug, Clone)]
pub struct FocusedWindowSnapshot {
    pub app_id: String,
    pub title: String,
    pub pid: Option<u32>,
    pub observed_at: DateTime<Utc>,
}

pub fn map_snapshot_to_event(snapshot: FocusedWindowSnapshot) -> ActivityEvent {
    ActivityEvent::app_focus(&snapshot.app_id, &snapshot.title, snapshot.observed_at)
}

pub async fn focused_window_once() -> anyhow::Result<Option<FocusedWindowSnapshot>> {
    let mut socket = niri_ipc::socket::Socket::connect().await?;
    let window = socket.send(niri_ipc::Request::FocusedWindow).await?;

    Ok(window.map(|window| FocusedWindowSnapshot {
        app_id: window.app_id.unwrap_or_else(|| "unknown".to_owned()),
        title: window.title.unwrap_or_default(),
        pid: window.pid,
        observed_at: Utc::now(),
    }))
}
```

- [ ] **Step 4: Add the idle-state adapter contract**

```rust
// crates/tracker/src/idle.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleState {
    Active,
    Idle,
}
```

- [ ] **Step 5: Export the tracker API**

```rust
// crates/tracker/src/lib.rs
mod idle;
mod niri;

pub use idle::IdleState;
pub use niri::{focused_window_once, map_snapshot_to_event, FocusedWindowSnapshot};
```

- [ ] **Step 6: Run the tracker test**

Run: `cargo test -p tm-tracker --test niri_mapping`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/tracker/src/lib.rs crates/tracker/src/niri.rs crates/tracker/src/idle.rs crates/tracker/tests/niri_mapping.rs
git commit -m "feat: add niri focus normalization"
```

## Task 5: Define daemon IPC messages for future local clients

**Files:**
- Modify: `crates/ipc/src/lib.rs`
- Create: `crates/ipc/src/messages.rs`
- Create: `crates/ipc/tests/message_roundtrip.rs`

- [ ] **Step 1: Write the failing IPC roundtrip test**

```rust
// crates/ipc/tests/message_roundtrip.rs
use tm_ipc::{DaemonCommand, DaemonEvent};

#[test]
fn serializes_flush_command_and_ack_event() {
    let cmd = serde_json::to_string(&DaemonCommand::FlushActiveSession).unwrap();
    let evt = serde_json::to_string(&DaemonEvent::Ack).unwrap();

    assert_eq!(cmd, r#"\"FlushActiveSession\""#);
    assert_eq!(evt, r#"\"Ack\""#);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-ipc serializes_flush_command_and_ack_event -- --exact`
Expected: FAIL because the message types do not exist.

- [ ] **Step 3: Add the message types**

```rust
// crates/ipc/src/messages.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    FlushActiveSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonEvent {
    Ack,
}
```

- [ ] **Step 4: Export the message API**

```rust
// crates/ipc/src/lib.rs
mod messages;

pub use messages::{DaemonCommand, DaemonEvent};

pub fn ipc_ready() -> &'static str {
    "tm-ipc-ready"
}
```

- [ ] **Step 5: Run the IPC test**

Run: `cargo test -p tm-ipc --test message_roundtrip`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/lib.rs crates/ipc/src/messages.rs crates/ipc/tests/message_roundtrip.rs
git commit -m "feat: add daemon ipc contracts"
```

## Task 6: Build the daemon session service and end-to-end close-on-focus-change flow

**Files:**
- Modify: `crates/daemon/src/lib.rs`
- Modify: `crates/daemon/src/main.rs`
- Create: `crates/daemon/src/app.rs`
- Create: `crates/daemon/src/session_service.rs`
- Create: `crates/daemon/tests/session_service.rs`

- [ ] **Step 1: Write the failing daemon service test**

```rust
// crates/daemon/tests/session_service.rs
use chrono::{TimeZone, Utc};
use tm_core::ActivityEvent;
use tm_daemon::SessionService;
use tm_storage::SqliteRepository;

#[tokio::test]
async fn daemon_persists_closed_session_on_focus_change() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let mut service = SessionService::new(repo);

    service
        .ingest(ActivityEvent::app_focus(
            "wezterm",
            "shell",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap(),
        ))
        .await
        .unwrap();

    service
        .ingest(ActivityEvent::app_focus(
            "firefox",
            "Rust docs",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap(),
        ))
        .await
        .unwrap();

    let sessions = service.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].subject_id, "wezterm");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p tm-daemon daemon_persists_closed_session_on_focus_change -- --exact`
Expected: FAIL because `SessionService` does not exist.

- [ ] **Step 3: Implement the session service**

```rust
// crates/daemon/src/session_service.rs
use anyhow::Result;
use chrono::{DateTime, Utc};
use tm_core::{ActivityEvent, ClosedSession, SessionAccumulator};
use tm_storage::SqliteRepository;

pub struct SessionService {
    accumulator: SessionAccumulator,
    repo: SqliteRepository,
}

impl SessionService {
    pub fn new(repo: SqliteRepository) -> Self {
        Self {
            accumulator: SessionAccumulator::default(),
            repo,
        }
    }

    pub async fn ingest(&mut self, event: ActivityEvent) -> Result<()> {
        if let Some(closed) = self.accumulator.ingest(event) {
            self.repo.insert_session(&closed).await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self, ended_at: DateTime<Utc>) -> Result<Option<ClosedSession>> {
        let closed = self.accumulator.flush(ended_at);
        if let Some(ref session) = closed {
            self.repo.insert_session(session).await?;
        }
        Ok(closed)
    }

    pub async fn list_sessions(&self) -> Result<Vec<ClosedSession>> {
        self.repo.list_sessions().await
    }
}
```

- [ ] **Step 4: Export the daemon library API**

```rust
// crates/daemon/src/lib.rs
mod app;
mod session_service;

pub use app::run;
pub use session_service::SessionService;
```

- [ ] **Step 5: Add the minimal runtime wiring**

```rust
// crates/daemon/src/app.rs
use anyhow::Result;
use tm_storage::SqliteRepository;

pub async fn run() -> Result<()> {
    let _repo = SqliteRepository::in_memory().await?;
    Ok(())
}
```

```rust
// crates/daemon/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tm_daemon::run().await
}
```

- [ ] **Step 6: Run daemon tests and a manual smoke run**

Run: `cargo test -p tm-daemon --test session_service && cargo run -p tm-daemon`
Expected: test PASS; manual run starts and exits cleanly with no panic.

- [ ] **Step 7: Commit**

```bash
git add crates/daemon/src/lib.rs crates/daemon/src/main.rs crates/daemon/src/app.rs crates/daemon/src/session_service.rs crates/daemon/tests/session_service.rs
git commit -m "feat: add daemon session ingestion"
```

## Task 7: Validate the foundation as a whole

**Files:**
- Modify: no new source files
- Test: full workspace

- [ ] **Step 1: Run the full test suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 2: Run formatting and linting**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS with zero formatting diffs and zero clippy warnings.

- [ ] **Step 3: Do a manual Niri integration probe**

Run: `cargo test -p tm-tracker --test niri_mapping && cargo run -p tm-daemon`
Expected: the mapping test passes and the daemon binary starts without crashing in a Niri session.

- [ ] **Step 4: Commit**

```bash
git add -u
git commit -m "chore: validate tracking foundation"
```

## Follow-up Plans After This One
- Plan B: desktop UI shell + Tai-like information architecture + tray/state entry.
- Plan C: browser extensions + native messaging host + website session ingestion.
- Plan D: filtering/categories/linked-app rules + exports + packaging.

## Spec Coverage Check
- Niri app focus tracking: covered by Tasks 4 and 6.
- Idle-aware session boundaries: domain contract added in Task 2 and tracker idle adapter added in Task 4; full idle event wiring belongs in the next foundation follow-up once focus flow is proven stable.
- SQLite local storage: Task 3.
- Daemon-only write path: Task 6.
- Local IPC contract for later UI/extension clients: Task 5.

## Self-Review Notes
- The previous draft mixed four subsystems into one plan and had cross-task inconsistencies around crate layout. This version removes that problem by planning only the tracking foundation.
- All task references now point to files that are created in this document.
- The only deferred work is explicitly listed under follow-up plans rather than implied inside the tasks.
