# TM GUI + Daemon IPC Read Path Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone `tm-ui` desktop client that reads real Overview, Charts, and Data pages from `tm-daemon` over IPC without direct SQLite access.

**Architecture:** Keep the current tracking runtime in `tm-daemon`, extend `tm-ipc` with typed query/response contracts plus a shared socket-path helper, add a daemon-side query service + Unix socket server, then add a new `crates/ui` `eframe/egui` client that renders daemon-backed pages and stable placeholders. Keep aggregation logic on the daemon side; the UI only owns navigation, local page state, and rendering.

**Tech Stack:** Rust 2024, `tokio`, Unix domain sockets, `serde`/`serde_json`, `chrono`, `eframe`/`egui`, `egui_plot`, SQLite via existing `tm-storage`.

---

## File Map

### Workspace
- Modify: `Cargo.toml` — add `crates/ui` to workspace members and extend the shared `tokio` feature list with `io-util` for socket line I/O.

### IPC contract
- Modify: `crates/ipc/src/messages.rs` — add typed read-query requests, responses, and shared page payload structs.
- Create: `crates/ipc/src/socket.rs` — shared socket-path resolution for daemon and UI.
- Modify: `crates/ipc/src/lib.rs` — export new message and socket APIs.
- Modify: `crates/ipc/tests/message_roundtrip.rs` — cover new explicit tagged wire format.
- Create: `crates/ipc/tests/socket_path.rs` — lock down runtime-dir/home fallback rules.

### Daemon query + IPC server
- Modify: `crates/storage/src/repository.rs` — derive `Clone` for `SqliteRepository` so runtime and query services can share one pool.
- Create: `crates/daemon/src/query.rs` — page-oriented aggregation service for overview, sessions, and charts.
- Create: `crates/daemon/src/ipc_server.rs` — Unix socket server that reads JSON requests and writes JSON responses.
- Modify: `crates/daemon/src/app.rs` — start the IPC server alongside the tracker loop and clean up the socket on exit.
- Modify: `crates/daemon/src/lib.rs` — export query service pieces needed by tests.
- Create: `crates/daemon/tests/query_service.rs` — daemon aggregation tests.
- Create: `crates/daemon/tests/ipc_server.rs` — real socket request/response integration tests.

### UI crate
- Create: `crates/ui/Cargo.toml` — GUI crate dependencies and package metadata.
- Create: `crates/ui/src/lib.rs` — export app/state/client modules for tests.
- Create: `crates/ui/src/main.rs` — desktop entrypoint.
- Create: `crates/ui/src/client.rs` — blocking Unix socket client for daemon requests.
- Create: `crates/ui/src/state.rs` — page enum, connection state, per-page load state, shared time range, and response reducers.
- Create: `crates/ui/src/app.rs` — `eframe::App` shell, left nav, background request orchestration, and content routing.
- Create: `crates/ui/src/pages/mod.rs` — page module exports.
- Create: `crates/ui/src/pages/overview.rs` — Overview rendering.
- Create: `crates/ui/src/pages/charts.rs` — Charts rendering via `egui_plot`.
- Create: `crates/ui/src/pages/data.rs` — Data table + filters rendering.
- Create: `crates/ui/src/pages/placeholder.rs` — stable placeholder views for Apps / Websites / Categories / Settings.
- Create: `crates/ui/tests/app_state.rs` — pure state transition tests for connection, overview, charts, and sessions responses.

## Task 1: Expand the IPC contract for GUI read queries

**Files:**
- Modify: `crates/ipc/src/messages.rs`
- Create: `crates/ipc/src/socket.rs`
- Modify: `crates/ipc/src/lib.rs`
- Modify: `crates/ipc/tests/message_roundtrip.rs`
- Create: `crates/ipc/tests/socket_path.rs`

- [ ] **Step 1: Add a failing round-trip test for the new read-path wire format**

```rust
// crates/ipc/tests/message_roundtrip.rs
use chrono::{TimeZone, Utc};
use serde_json::json;
use tm_core::ActivityKind;
use tm_ipc::{
    ActivityFilter, ChartBucket, ChartsQuery, ChartsResponse, DaemonCommand, DaemonEvent,
    DaemonRequest, DaemonResponse, OverviewQuery, OverviewResponse, SessionRow, SessionsQuery,
    SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};

#[test]
fn serializes_read_requests_and_responses_to_explicit_tagged_wire_format() {
    let range = TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    };

    let request = serde_json::to_value(DaemonRequest::GetSessions(SessionsQuery {
        range: range.clone(),
        activity_filter: ActivityFilter::Website,
        subject_query: Some("docs.rs".into()),
    }))
    .unwrap();

    let response = serde_json::to_value(DaemonResponse::Overview(OverviewResponse {
        range,
        total_seconds: 600,
        top_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 600,
        }],
        top_websites: vec![],
        recent_sessions: vec![SessionRow {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 10, 0).unwrap(),
            duration_seconds: 600,
        }],
    }))
    .unwrap();

    assert_eq!(request, json!({
        "type": "get_sessions",
        "range": {
            "started_at": "2026-04-13T00:00:00Z",
            "ended_at": "2026-04-13T23:59:59Z"
        },
        "activity_filter": "website",
        "subject_query": "docs.rs"
    }));

    assert_eq!(response, json!({
        "type": "overview",
        "range": {
            "started_at": "2026-04-13T00:00:00Z",
            "ended_at": "2026-04-13T23:59:59Z"
        },
        "total_seconds": 600,
        "top_apps": [{
            "kind": "App",
            "subject_id": "wezterm",
            "title": "shell",
            "total_seconds": 600
        }],
        "top_websites": [],
        "recent_sessions": [{
            "kind": "App",
            "subject_id": "wezterm",
            "title": "shell",
            "started_at": "2026-04-13T09:00:00Z",
            "ended_at": "2026-04-13T09:10:00Z",
            "duration_seconds": 600
        }]
    }));
}

#[test]
fn keeps_existing_operational_command_and_event_roundtrip() {
    let cmd_json = serde_json::to_string(&DaemonCommand::FlushActiveSession).unwrap();
    let evt_json = serde_json::to_string(&DaemonEvent::Ack).unwrap();

    let cmd: DaemonCommand = serde_json::from_str(&cmd_json).unwrap();
    let evt: DaemonEvent = serde_json::from_str(&evt_json).unwrap();

    assert_eq!(cmd, DaemonCommand::FlushActiveSession);
    assert_eq!(evt, DaemonEvent::Ack);
}
```

- [ ] **Step 2: Run the IPC test to verify the new types do not exist yet**

Run: `cargo test -p tm-ipc --test message_roundtrip -- --exact serializes_read_requests_and_responses_to_explicit_tagged_wire_format`
Expected: FAIL with unresolved imports like `no DaemonRequest in the root`.

- [ ] **Step 3: Implement the read-path message types**

```rust
// crates/ipc/src/messages.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tm_core::ActivityKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityFilter {
    All,
    App,
    Website,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverviewQuery {
    pub range: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsQuery {
    pub range: TimeRange,
    pub activity_filter: ActivityFilter,
    pub subject_query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartsQuery {
    pub range: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryBucket {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendPoint {
    pub bucket_start: DateTime<Utc>,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartBucket {
    pub label: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverviewResponse {
    pub range: TimeRange,
    pub total_seconds: i64,
    pub top_apps: Vec<SummaryBucket>,
    pub top_websites: Vec<SummaryBucket>,
    pub recent_sessions: Vec<SessionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub range: TimeRange,
    pub activity_filter: ActivityFilter,
    pub subject_query: Option<String>,
    pub items: Vec<SessionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartsResponse {
    pub range: TimeRange,
    pub app_share: Vec<SummaryBucket>,
    pub website_share: Vec<SummaryBucket>,
    pub daily_totals: Vec<TrendPoint>,
    pub hourly_totals: Vec<ChartBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonCommand {
    FlushActiveSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonEvent {
    Ack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequest {
    Ping,
    GetOverview(OverviewQuery),
    GetSessions(SessionsQuery),
    GetCharts(ChartsQuery),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonResponse {
    Pong,
    Overview(OverviewResponse),
    Sessions(SessionsResponse),
    Charts(ChartsResponse),
    Error { message: String },
}
```

- [ ] **Step 4: Add a failing test for shared socket-path resolution**

```rust
// crates/ipc/tests/socket_path.rs
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
```

- [ ] **Step 5: Run the new socket-path test to verify the helper is missing**

Run: `cargo test -p tm-ipc --test socket_path -- --exact socket_path_prefers_xdg_runtime_dir`
Expected: FAIL with `unresolved import tm_ipc::socket_path_from_env`.

- [ ] **Step 6: Implement and export the shared socket-path helper**

```rust
// crates/ipc/src/socket.rs
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

    let mut home = home.ok_or_else(|| "HOME is not set; cannot resolve tm socket path".to_owned())?;
    home.push(".local");
    home.push("state");
    home.push("tm");
    home.push("tm.sock");
    Ok(home)
}
```

```rust
// crates/ipc/src/lib.rs
mod messages;
mod socket;

pub use messages::{
    ActivityFilter, ChartBucket, ChartsQuery, ChartsResponse, DaemonCommand, DaemonEvent,
    DaemonRequest, DaemonResponse, OverviewQuery, OverviewResponse, SessionRow, SessionsQuery,
    SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};
pub use socket::{default_socket_path, socket_path_from_env};

pub fn ipc_ready() -> &'static str {
    "tm-ipc-ready"
}
```

- [ ] **Step 7: Run the IPC test suite**

Run: `cargo test -p tm-ipc --test message_roundtrip && cargo test -p tm-ipc --test socket_path`
Expected: PASS.

- [ ] **Step 8: Commit the IPC contract work**

```bash
git add crates/ipc/src/messages.rs crates/ipc/src/socket.rs crates/ipc/src/lib.rs crates/ipc/tests/message_roundtrip.rs crates/ipc/tests/socket_path.rs
git commit -m "feat: add gui read ipc contract"
```

## Task 2: Add a daemon-side query service for Overview, Sessions, and Charts

**Files:**
- Modify: `crates/storage/src/repository.rs`
- Create: `crates/daemon/src/query.rs`
- Modify: `crates/daemon/src/lib.rs`
- Create: `crates/daemon/tests/query_service.rs`

- [ ] **Step 1: Add a failing daemon query-service test covering app + website aggregation**

```rust
// crates/daemon/tests/query_service.rs
use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_daemon::QueryService;
use tm_ipc::{ActivityFilter, ChartsQuery, OverviewQuery, SessionsQuery, TimeRange};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn overview_splits_app_and_website_rankings() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 10),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 9, 10, 9, 25),
            session(ActivityKind::App, "firefox", "ChatGPT", 9, 25, 9, 40),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_overview(OverviewQuery { range: day_range() })
        .await
        .unwrap();

    assert_eq!(result.total_seconds, 2_400);
    assert_eq!(result.top_apps.len(), 2);
    assert_eq!(result.top_apps[0].subject_id, "firefox");
    assert_eq!(result.top_websites.len(), 1);
    assert_eq!(result.top_websites[0].subject_id, "docs.rs");
    assert_eq!(result.recent_sessions[0].subject_id, "firefox");
}

#[tokio::test]
async fn sessions_query_filters_by_kind_and_subject() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 10),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 9, 10, 9, 25),
            session(ActivityKind::Website, "news.ycombinator.com", "HN", 9, 25, 9, 35),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_sessions(SessionsQuery {
            range: day_range(),
            activity_filter: ActivityFilter::Website,
            subject_query: Some("docs".into()),
        })
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].subject_id, "docs.rs");
}

#[tokio::test]
async fn charts_query_returns_daily_and_hourly_series() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 30),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 10, 0, 10, 15),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_charts(ChartsQuery { range: day_range() })
        .await
        .unwrap();

    assert_eq!(result.app_share[0].subject_id, "wezterm");
    assert_eq!(result.website_share[0].subject_id, "docs.rs");
    assert_eq!(result.daily_totals.len(), 1);
    assert_eq!(result.hourly_totals.len(), 2);
}

fn day_range() -> TimeRange {
    TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    }
}

fn session(
    kind: ActivityKind,
    subject_id: &str,
    title: &str,
    start_hour: u32,
    start_minute: u32,
    end_hour: u32,
    end_minute: u32,
) -> ClosedSession {
    ClosedSession::new(
        kind,
        subject_id.into(),
        title.into(),
        Utc.with_ymd_and_hms(2026, 4, 13, start_hour, start_minute, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 4, 13, end_hour, end_minute, 0).unwrap(),
    )
    .unwrap()
}

async fn seed(repo: &SqliteRepository, sessions: &[ClosedSession]) {
    for session in sessions {
        repo.insert_session(session).await.unwrap();
    }
}
```

- [ ] **Step 2: Run the query-service test to verify the service does not exist yet**

Run: `cargo test -p tm-daemon --test query_service -- --exact overview_splits_app_and_website_rankings`
Expected: FAIL with unresolved import `tm_daemon::QueryService`.

- [ ] **Step 3: Make `SqliteRepository` clonable so runtime and query service can share one pool**

```rust
// crates/storage/src/repository.rs
#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}
```

- [ ] **Step 4: Implement the daemon query service and pure aggregation helpers**

```rust
// crates/daemon/src/query.rs
use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Timelike, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_ipc::{
    ActivityFilter, ChartBucket, ChartsQuery, ChartsResponse, OverviewQuery, OverviewResponse,
    SessionRow, SessionsQuery, SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};
use tm_storage::RepositoryError;

use crate::SessionRepository;

pub struct QueryService<R> {
    repo: R,
}

impl<R> QueryService<R>
where
    R: SessionRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_overview(&self, query: OverviewQuery) -> Result<OverviewResponse, RepositoryError> {
        let sessions = scoped_sessions(self.repo.list_sessions().await?, &query.range, ActivityFilter::All, None);
        Ok(OverviewResponse {
            range: query.range,
            total_seconds: sessions.iter().map(|row| row.duration_seconds).sum(),
            top_apps: top_buckets(&sessions, ActivityKind::App),
            top_websites: top_buckets(&sessions, ActivityKind::Website),
            recent_sessions: recent_rows(&sessions),
        })
    }

    pub async fn get_sessions(&self, query: SessionsQuery) -> Result<SessionsResponse, RepositoryError> {
        let items = scoped_sessions(
            self.repo.list_sessions().await?,
            &query.range,
            query.activity_filter,
            query.subject_query.as_deref(),
        );
        Ok(SessionsResponse {
            range: query.range,
            activity_filter: query.activity_filter,
            subject_query: query.subject_query,
            items,
        })
    }

    pub async fn get_charts(&self, query: ChartsQuery) -> Result<ChartsResponse, RepositoryError> {
        let sessions = scoped_sessions(self.repo.list_sessions().await?, &query.range, ActivityFilter::All, None);
        Ok(ChartsResponse {
            range: query.range,
            app_share: top_buckets(&sessions, ActivityKind::App),
            website_share: top_buckets(&sessions, ActivityKind::Website),
            daily_totals: daily_totals(&sessions),
            hourly_totals: hourly_totals(&sessions),
        })
    }
}

fn scoped_sessions(
    sessions: Vec<ClosedSession>,
    range: &TimeRange,
    filter: ActivityFilter,
    subject_query: Option<&str>,
) -> Vec<SessionRow> {
    let query = subject_query.map(|value| value.to_ascii_lowercase());
    let mut rows = sessions
        .into_iter()
        .filter(|session| session.ended_at() > range.started_at && session.started_at() < range.ended_at)
        .filter(|session| match filter {
            ActivityFilter::All => true,
            ActivityFilter::App => session.kind() == ActivityKind::App,
            ActivityFilter::Website => session.kind() == ActivityKind::Website,
        })
        .filter_map(|session| row_for_range(session, range))
        .filter(|row| {
            query
                .as_ref()
                .map(|query| row.subject_id.to_ascii_lowercase().contains(query))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    rows.sort_by_key(|row| std::cmp::Reverse(row.started_at));
    rows
}

fn row_for_range(session: ClosedSession, range: &TimeRange) -> Option<SessionRow> {
    let started_at = session.started_at().max(range.started_at);
    let ended_at = session.ended_at().min(range.ended_at);
    let duration_seconds = (ended_at - started_at).num_seconds();

    (duration_seconds > 0).then_some(SessionRow {
        kind: session.kind(),
        subject_id: session.subject_id().to_owned(),
        title: session.title().to_owned(),
        started_at,
        ended_at,
        duration_seconds,
    })
}

fn top_buckets(rows: &[SessionRow], kind: ActivityKind) -> Vec<SummaryBucket> {
    let mut grouped: BTreeMap<String, SummaryBucket> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.kind == kind) {
        let entry = grouped.entry(row.subject_id.clone()).or_insert_with(|| SummaryBucket {
            kind,
            subject_id: row.subject_id.clone(),
            title: row.title.clone(),
            total_seconds: 0,
        });
        entry.total_seconds += row.duration_seconds;
        entry.title = row.title.clone();
    }

    let mut buckets = grouped.into_values().collect::<Vec<_>>();
    buckets.sort_by_key(|bucket| std::cmp::Reverse(bucket.total_seconds));
    buckets.truncate(5);
    buckets
}

fn recent_rows(rows: &[SessionRow]) -> Vec<SessionRow> {
    rows.iter().take(10).cloned().collect()
}

fn daily_totals(rows: &[SessionRow]) -> Vec<TrendPoint> {
    let mut grouped: BTreeMap<DateTime<Utc>, i64> = BTreeMap::new();
    for row in rows {
        let day = Utc
            .with_ymd_and_hms(row.started_at.year(), row.started_at.month(), row.started_at.day(), 0, 0, 0)
            .unwrap();
        *grouped.entry(day).or_default() += row.duration_seconds;
    }

    grouped
        .into_iter()
        .map(|(bucket_start, total_seconds)| TrendPoint {
            bucket_start,
            total_seconds,
        })
        .collect()
}

fn hourly_totals(rows: &[SessionRow]) -> Vec<ChartBucket> {
    let mut grouped: BTreeMap<u32, i64> = BTreeMap::new();
    for row in rows {
        *grouped.entry(row.started_at.hour()).or_default() += row.duration_seconds;
    }

    grouped
        .into_iter()
        .map(|(hour, total_seconds)| ChartBucket {
            label: format!("{hour:02}:00"),
            total_seconds,
        })
        .collect()
}
```

- [ ] **Step 5: Export the query service from the daemon crate**

```rust
// crates/daemon/src/lib.rs
mod app;
mod query;
mod session_service;

pub use app::run;
pub use query::QueryService;
pub use session_service::{FlushOutcome, IngestOutcome, SessionRepository, SessionService};
```

- [ ] **Step 6: Run the query-service tests**

Run: `cargo test -p tm-daemon --test query_service`
Expected: PASS.

- [ ] **Step 7: Commit the daemon query layer**

```bash
git add crates/storage/src/repository.rs crates/daemon/src/query.rs crates/daemon/src/lib.rs crates/daemon/tests/query_service.rs
git commit -m "feat: add daemon read query service"
```

## Task 3: Serve daemon queries over a Unix socket without regressing the tracking runtime

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/daemon/src/ipc_server.rs`
- Modify: `crates/daemon/src/app.rs`
- Modify: `crates/daemon/src/lib.rs`
- Create: `crates/daemon/tests/ipc_server.rs`

- [ ] **Step 1: Add a failing end-to-end IPC server test**

```rust
// crates/daemon/tests/ipc_server.rs
use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_daemon::{run_ipc_server, QueryService};
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
    let server = tokio::spawn(run_ipc_server(listener, QueryService::new(repo), shutdown_rx));

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
```

- [ ] **Step 2: Run the new IPC integration test to verify the server entrypoint is missing**

Run: `cargo test -p tm-daemon --test ipc_server -- --exact ipc_server_roundtrips_overview_requests`
Expected: FAIL with unresolved import `tm_daemon::run_ipc_server` and/or missing `io-util` methods.

- [ ] **Step 3: Extend the shared Tokio feature set for line-based socket I/O**

```toml
# Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "signal", "sync", "time", "io-util"] }
```

- [ ] **Step 4: Implement the daemon IPC server**

```rust
// crates/daemon/src/ipc_server.rs
use std::path::PathBuf;

use anyhow::Result;
use tm_ipc::{default_socket_path, DaemonRequest, DaemonResponse};
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
    };

    writer
        .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
        .await?;
    Ok(())
}
```

- [ ] **Step 5: Add the tiny repo accessor needed by the server spawn path**

```rust
// crates/daemon/src/query.rs
impl<R> QueryService<R> {
    pub fn repo(&self) -> &R {
        &self.repo
    }
}
```

- [ ] **Step 6: Start and stop the IPC server from the main daemon runtime**

```rust
// crates/daemon/src/app.rs
use tokio::sync::oneshot;
use tm_storage::SqliteRepository;

use crate::{ipc_server::{bind_listener, run_ipc_server}, QueryService, SessionRepository, SessionService};

pub async fn run() -> Result<()> {
    let db_path = default_db_path().context("failed to resolve daemon database path")?;
    let repo = SqliteRepository::open_at(db_path).await?;

    let (listener, socket_path) = bind_listener().await?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let ipc_task = tokio::spawn(run_ipc_server(listener, QueryService::new(repo.clone()), shutdown_rx));

    let mut service = SessionService::new(repo);
    let mut previous_focus = None;
    let mut interval = tokio::time::interval(POLL_INTERVAL);

    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                flush_active_session(&mut service, Utc::now()).await?;
                let _ = shutdown_tx.send(());
                ipc_task.await??;
                let _ = std::fs::remove_file(socket_path);
                return Ok(());
            }
            _ = interval.tick() => {
                poll_tracker_once(&mut service, &mut previous_focus).await?;
            }
        }
    }
}
```

- [ ] **Step 7: Export the IPC server entrypoint for integration tests**

```rust
// crates/daemon/src/lib.rs
mod app;
mod ipc_server;
mod query;
mod session_service;

pub use app::run;
pub use ipc_server::run_ipc_server;
pub use query::QueryService;
pub use session_service::{FlushOutcome, IngestOutcome, SessionRepository, SessionService};
```

- [ ] **Step 8: Run the daemon IPC + runtime tests**

Run: `cargo test -p tm-daemon --test ipc_server && cargo test -p tm-daemon --test runtime_shutdown && cargo test -p tm-daemon --test session_service`
Expected: PASS.

- [ ] **Step 9: Commit the daemon IPC work**

```bash
git add Cargo.toml crates/daemon/src/ipc_server.rs crates/daemon/src/app.rs crates/daemon/src/query.rs crates/daemon/src/lib.rs crates/daemon/tests/ipc_server.rs
git commit -m "feat: serve daemon read queries over ipc"
```

## Task 4: Add the standalone `tm-ui` shell, client, and state model

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ui/Cargo.toml`
- Create: `crates/ui/src/lib.rs`
- Create: `crates/ui/src/main.rs`
- Create: `crates/ui/src/client.rs`
- Create: `crates/ui/src/state.rs`
- Create: `crates/ui/src/app.rs`
- Create: `crates/ui/src/pages/mod.rs`
- Create: `crates/ui/src/pages/placeholder.rs`
- Create: `crates/ui/tests/app_state.rs`

- [ ] **Step 1: Add a failing state test for disconnected startup and overview loading**

```rust
// crates/ui/tests/app_state.rs
use chrono::{TimeZone, Utc};
use tm_core::ActivityKind;
use tm_ipc::{DaemonResponse, OverviewResponse, SessionRow, SummaryBucket, TimeRange};
use tm_ui::{AppState, ConnectionState, LoadState, Page};

#[test]
fn disconnected_error_marks_connection_state_and_current_page() {
    let mut state = AppState::new(day_range());
    state.select_page(Page::Overview);
    state.apply_client_error("socket missing".into());

    assert!(matches!(state.connection, ConnectionState::Disconnected(_)));
    assert!(matches!(state.overview, LoadState::Error(message) if message.contains("socket missing")));
}

#[test]
fn overview_response_populates_loaded_state() {
    let mut state = AppState::new(day_range());
    state.apply_response(DaemonResponse::Overview(OverviewResponse {
        range: day_range(),
        total_seconds: 900,
        top_apps: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 900,
        }],
        top_websites: vec![],
        recent_sessions: vec![SessionRow {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 9, 15, 0).unwrap(),
            duration_seconds: 900,
        }],
    }));

    assert!(matches!(state.connection, ConnectionState::Connected));
    assert!(matches!(state.overview, LoadState::Loaded(data) if data.total_seconds == 900));
}

fn day_range() -> TimeRange {
    TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    }
}
```

- [ ] **Step 2: Run the UI state test to verify the crate does not exist yet**

Run: `cargo test -p tm-ui --test app_state -- --exact disconnected_error_marks_connection_state_and_current_page`
Expected: FAIL with `package ID specification 'tm-ui' did not match any packages`.

- [ ] **Step 3: Add the workspace member and crate manifest for the UI**

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
  "crates/ui",
]
```

```toml
# crates/ui/Cargo.toml
[package]
name = "tm-ui"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
chrono.workspace = true
eframe = "0.31"
egui_plot = "0.31"
serde_json.workspace = true
tm-ipc = { path = "../ipc" }

[dev-dependencies]
tm-core = { path = "../core" }
```

- [ ] **Step 4: Implement the UI client and state model**

```rust
// crates/ui/src/client.rs
use std::{io::{BufRead, BufReader, Write}, os::unix::net::UnixStream};

use tm_ipc::{default_socket_path, DaemonRequest, DaemonResponse};

pub struct IpcClient {
    socket_path: std::path::PathBuf,
}

impl IpcClient {
    pub fn new(socket_path: std::path::PathBuf) -> Self {
        Self { socket_path }
    }

    pub fn default() -> Result<Self, String> {
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
}
```

```rust
// crates/ui/src/state.rs
use tm_ipc::{ChartsResponse, DaemonResponse, OverviewResponse, SessionsResponse, TimeRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Charts,
    Data,
    Apps,
    Websites,
    Categories,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected(String),
    Retrying,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    Loading,
    Loaded(T),
    Empty,
    Error(String),
}

pub struct AppState {
    pub page: Page,
    pub range: TimeRange,
    pub connection: ConnectionState,
    pub overview: LoadState<OverviewResponse>,
    pub charts: LoadState<ChartsResponse>,
    pub data: LoadState<SessionsResponse>,
}

impl AppState {
    pub fn new(range: TimeRange) -> Self {
        Self {
            page: Page::Overview,
            range,
            connection: ConnectionState::Retrying,
            overview: LoadState::Loading,
            charts: LoadState::Loading,
            data: LoadState::Loading,
        }
    }

    pub fn select_page(&mut self, page: Page) {
        self.page = page;
    }

    pub fn apply_client_error(&mut self, message: String) {
        self.connection = ConnectionState::Disconnected(message.clone());
        match self.page {
            Page::Overview => self.overview = LoadState::Error(message),
            Page::Charts => self.charts = LoadState::Error(message),
            Page::Data => self.data = LoadState::Error(message),
            _ => {}
        }
    }

    pub fn apply_response(&mut self, response: DaemonResponse) {
        self.connection = ConnectionState::Connected;
        match response {
            DaemonResponse::Overview(payload) => {
                self.overview = if payload.recent_sessions.is_empty() {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Charts(payload) => {
                self.charts = if payload.daily_totals.is_empty() && payload.hourly_totals.is_empty() {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Sessions(payload) => {
                self.data = if payload.items.is_empty() {
                    LoadState::Empty
                } else {
                    LoadState::Loaded(payload)
                };
            }
            DaemonResponse::Pong | DaemonResponse::Error { .. } => {}
        }
    }
}
```

- [ ] **Step 5: Implement the shell app, placeholder page, and public exports**

```rust
// crates/ui/src/lib.rs
pub mod app;
pub mod client;
pub mod pages;
pub mod state;

pub use app::TmApp;
pub use state::{AppState, ConnectionState, LoadState, Page};
```

```rust
// crates/ui/src/pages/mod.rs
pub mod placeholder;
```

```rust
// crates/ui/src/pages/placeholder.rs
use eframe::egui;

pub fn render(ui: &mut egui::Ui, title: &str) {
    ui.heading(title);
    ui.label("This section is part of the Tai-like navigation shell and will gain real content in a later slice.");
}
```

```rust
// crates/ui/src/app.rs
use chrono::{TimeZone, Utc};
use eframe::egui;

use crate::{
    client::IpcClient,
    pages::placeholder,
    state::{AppState, Page},
};

pub struct TmApp {
    client: IpcClient,
    state: AppState,
}

impl Default for TmApp {
    fn default() -> Self {
        let range = tm_ipc::TimeRange {
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
        };
        Self {
            client: IpcClient::default().unwrap_or_else(|_| IpcClient::new(std::path::PathBuf::from("/tmp/tm.sock"))),
            state: AppState::new(range),
        }
    }
}

impl eframe::App for TmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("nav").show(ctx, |ui| {
            ui.heading("tm");
            for (label, page) in [
                ("Overview", Page::Overview),
                ("Charts", Page::Charts),
                ("Data", Page::Data),
                ("Apps", Page::Apps),
                ("Websites", Page::Websites),
                ("Categories", Page::Categories),
                ("Settings", Page::Settings),
            ] {
                if ui.selectable_label(self.state.page == page, label).clicked() {
                    self.state.select_page(page);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.state.page {
            Page::Apps => placeholder::render(ui, "Apps"),
            Page::Websites => placeholder::render(ui, "Websites"),
            Page::Categories => placeholder::render(ui, "Categories"),
            Page::Settings => placeholder::render(ui, "Settings"),
            Page::Overview => ui.label("Overview wiring lands in Task 5."),
            Page::Charts => ui.label("Charts wiring lands in Task 5."),
            Page::Data => ui.label("Data wiring lands in Task 5."),
        });
    }
}
```

```rust
// crates/ui/src/main.rs
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("tm", options, Box::new(|_cc| Ok(Box::new(tm_ui::TmApp::default()))))
}
```

- [ ] **Step 6: Run the UI state test suite**

Run: `cargo test -p tm-ui --test app_state`
Expected: PASS.

- [ ] **Step 7: Commit the UI shell**

```bash
git add Cargo.toml crates/ui/Cargo.toml crates/ui/src/lib.rs crates/ui/src/main.rs crates/ui/src/client.rs crates/ui/src/state.rs crates/ui/src/app.rs crates/ui/src/pages/mod.rs crates/ui/src/pages/placeholder.rs crates/ui/tests/app_state.rs
git commit -m "feat: add tm ui application shell"
```

## Task 5: Render real Overview, Charts, and Data pages from daemon responses

**Files:**
- Modify: `crates/ui/src/app.rs`
- Modify: `crates/ui/src/pages/mod.rs`
- Create: `crates/ui/src/pages/overview.rs`
- Create: `crates/ui/src/pages/charts.rs`
- Create: `crates/ui/src/pages/data.rs`
- Modify: `crates/ui/tests/app_state.rs`

- [ ] **Step 1: Extend the UI state tests to cover charts and sessions payload handling**

```rust
// crates/ui/tests/app_state.rs
use tm_ipc::{
    ActivityFilter, ChartBucket, ChartsResponse, SessionsResponse, TrendPoint,
};

#[test]
fn sessions_response_populates_loaded_data_state() {
    let mut state = AppState::new(day_range());
    state.apply_response(DaemonResponse::Sessions(SessionsResponse {
        range: day_range(),
        activity_filter: ActivityFilter::All,
        subject_query: None,
        items: vec![SessionRow {
            kind: ActivityKind::Website,
            subject_id: "docs.rs".into(),
            title: "Rust docs".into(),
            started_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(),
            ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 10, 15, 0).unwrap(),
            duration_seconds: 900,
        }],
    }));

    assert!(matches!(state.data, LoadState::Loaded(payload) if payload.items.len() == 1));
}

#[test]
fn charts_response_populates_loaded_chart_state() {
    let mut state = AppState::new(day_range());
    state.apply_response(DaemonResponse::Charts(ChartsResponse {
        range: day_range(),
        app_share: vec![SummaryBucket {
            kind: ActivityKind::App,
            subject_id: "wezterm".into(),
            title: "shell".into(),
            total_seconds: 900,
        }],
        website_share: vec![SummaryBucket {
            kind: ActivityKind::Website,
            subject_id: "docs.rs".into(),
            title: "Rust docs".into(),
            total_seconds: 600,
        }],
        daily_totals: vec![TrendPoint {
            bucket_start: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
            total_seconds: 1_500,
        }],
        hourly_totals: vec![ChartBucket {
            label: "09:00".into(),
            total_seconds: 900,
        }],
    }));

    assert!(matches!(state.charts, LoadState::Loaded(payload) if payload.daily_totals.len() == 1));
}
```

- [ ] **Step 2: Run the extended UI test suite to verify the shell does not yet render real pages**

Run: `cargo test -p tm-ui --test app_state`
Expected: FAIL until the new page modules are wired and the app requests daemon data.

- [ ] **Step 3: Implement the real page renderers**

```rust
// crates/ui/src/pages/mod.rs
pub mod charts;
pub mod data;
pub mod overview;
pub mod placeholder;
```

```rust
// crates/ui/src/pages/overview.rs
use eframe::egui;
use tm_ipc::OverviewResponse;

pub fn render(ui: &mut egui::Ui, payload: &OverviewResponse) {
    ui.heading("Overview");
    ui.label(format!("Tracked: {} minutes", payload.total_seconds / 60));

    ui.separator();
    ui.label("Top apps");
    for row in &payload.top_apps {
        ui.label(format!("{} — {} min", row.subject_id, row.total_seconds / 60));
    }

    ui.separator();
    ui.label("Top websites");
    for row in &payload.top_websites {
        ui.label(format!("{} — {} min", row.subject_id, row.total_seconds / 60));
    }

    ui.separator();
    ui.label("Recent activity");
    for row in &payload.recent_sessions {
        ui.label(format!("{} · {} min", row.subject_id, row.duration_seconds / 60));
    }
}
```

```rust
// crates/ui/src/pages/charts.rs
use eframe::egui;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use tm_ipc::ChartsResponse;

pub fn render(ui: &mut egui::Ui, payload: &ChartsResponse) {
    ui.heading("Charts");

    let bars = payload
        .hourly_totals
        .iter()
        .enumerate()
        .map(|(index, bucket)| Bar::new(index as f64, bucket.total_seconds as f64 / 60.0).name(bucket.label.clone()))
        .collect::<Vec<_>>();

    Plot::new("hourly-distribution").show(ui, |plot_ui| {
        plot_ui.bar_chart(BarChart::new("Hourly", bars));
    });

    let points = payload
        .daily_totals
        .iter()
        .enumerate()
        .map(|(index, point)| [index as f64, point.total_seconds as f64 / 60.0])
        .collect::<PlotPoints<'_>>();

    Plot::new("daily-trend").show(ui, |plot_ui| {
        plot_ui.line(Line::new("Daily total", points));
    });
}
```

```rust
// crates/ui/src/pages/data.rs
use eframe::egui;
use tm_ipc::SessionsResponse;

pub fn render(ui: &mut egui::Ui, payload: &SessionsResponse) {
    ui.heading("Data");
    egui::Grid::new("session-grid").striped(true).show(ui, |ui| {
        ui.label("Kind");
        ui.label("Subject");
        ui.label("Title");
        ui.label("Duration");
        ui.end_row();

        for row in &payload.items {
            ui.label(format!("{:?}", row.kind));
            ui.label(&row.subject_id);
            ui.label(&row.title);
            ui.label(format!("{} min", row.duration_seconds / 60));
            ui.end_row();
        }
    });
}
```

- [ ] **Step 4: Wire background daemon requests and page routing into the app shell**

```rust
// crates/ui/src/app.rs
use std::sync::mpsc::{self, Receiver};
use std::thread;

use eframe::egui;
use tm_ipc::{ActivityFilter, ChartsQuery, DaemonRequest, OverviewQuery, SessionsQuery};

use crate::{
    client::IpcClient,
    pages::{charts, data, overview, placeholder},
    state::{AppState, ConnectionState, LoadState, Page},
};

pub struct TmApp {
    client: IpcClient,
    state: AppState,
    pending: Option<Receiver<Result<tm_ipc::DaemonResponse, String>>>,
}

impl TmApp {
    fn request_current_page(&mut self) {
        let request = match self.state.page {
            Page::Overview => DaemonRequest::GetOverview(OverviewQuery {
                range: self.state.range.clone(),
            }),
            Page::Charts => DaemonRequest::GetCharts(ChartsQuery {
                range: self.state.range.clone(),
            }),
            Page::Data => DaemonRequest::GetSessions(SessionsQuery {
                range: self.state.range.clone(),
                activity_filter: ActivityFilter::All,
                subject_query: None,
            }),
            _ => return,
        };

        let client = IpcClient::new(self.client.socket_path().to_path_buf());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(client.send(request));
        });
        self.pending = Some(rx);
    }
}

impl eframe::App for TmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.pending {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(response) => self.state.apply_response(response),
                    Err(message) => self.state.apply_client_error(message),
                }
                self.pending = None;
            }
        }

        egui::SidePanel::left("nav").show(ctx, |ui| {
            ui.heading("tm");
            for (label, page) in [
                ("Overview", Page::Overview),
                ("Charts", Page::Charts),
                ("Data", Page::Data),
                ("Apps", Page::Apps),
                ("Websites", Page::Websites),
                ("Categories", Page::Categories),
                ("Settings", Page::Settings),
            ] {
                if ui.selectable_label(self.state.page == page, label).clicked() {
                    self.state.select_page(page);
                    self.request_current_page();
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.state.page {
            Page::Overview => match &self.state.overview {
                LoadState::Loading => ui.label("Loading overview..."),
                LoadState::Loaded(payload) => overview::render(ui, payload),
                LoadState::Empty => ui.label("No overview data yet."),
                LoadState::Error(message) => ui.label(message),
            },
            Page::Charts => match &self.state.charts {
                LoadState::Loading => ui.label("Loading charts..."),
                LoadState::Loaded(payload) => charts::render(ui, payload),
                LoadState::Empty => ui.label("No chart data yet."),
                LoadState::Error(message) => ui.label(message),
            },
            Page::Data => match &self.state.data {
                LoadState::Loading => ui.label("Loading sessions..."),
                LoadState::Loaded(payload) => data::render(ui, payload),
                LoadState::Empty => ui.label("No sessions yet."),
                LoadState::Error(message) => ui.label(message),
            },
            Page::Apps => placeholder::render(ui, "Apps"),
            Page::Websites => placeholder::render(ui, "Websites"),
            Page::Categories => placeholder::render(ui, "Categories"),
            Page::Settings => placeholder::render(ui, "Settings"),
        });

        if self.pending.is_none() && matches!(self.state.connection, ConnectionState::Retrying) {
            self.request_current_page();
        }
        ctx.request_repaint();
    }
}
```

- [ ] **Step 5: Add the tiny socket-path accessor used by the background thread spawn**

```rust
// crates/ui/src/client.rs
impl IpcClient {
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }
}
```

- [ ] **Step 6: Run the UI tests and then smoke the GUI binary**

Run: `cargo test -p tm-ui --test app_state && cargo run -p tm-ui`
Expected: tests PASS; the GUI opens with Tai-style left navigation and attempts to load daemon data for Overview/Charts/Data.

- [ ] **Step 7: Commit the first real GUI pages**

```bash
git add crates/ui/src/app.rs crates/ui/src/client.rs crates/ui/src/pages/mod.rs crates/ui/src/pages/overview.rs crates/ui/src/pages/charts.rs crates/ui/src/pages/data.rs crates/ui/tests/app_state.rs
git commit -m "feat: render daemon-backed gui pages"
```

## Task 6: Validate the full read-path slice end-to-end

**Files:**
- Modify: no planned source files

- [ ] **Step 1: Run the full workspace test suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 2: Run formatting and lint validation**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 3: Smoke the daemon and GUI together**

Run: `cargo run -p tm-daemon`
Expected: daemon starts, binds the shared socket path, and continues polling without exiting.

In a second terminal, run: `cargo run -p tm-ui`
Expected: GUI starts, Overview/Charts/Data connect over IPC, and placeholder pages remain reachable.

- [ ] **Step 4: Verify disconnected handling**

Stop `tm-daemon`, then relaunch only `cargo run -p tm-ui`.
Expected: GUI stays open and shows a clear disconnected/error state instead of crashing or touching SQLite directly.

## Self-Review

- **Spec coverage:**
  - standalone GUI client: Tasks 4-5
  - daemon-owned read models over IPC: Tasks 1-3
  - real Overview / Charts / Data pages: Task 5
  - app + website unified query models: Tasks 1-2
  - placeholder pages for Apps / Websites / Categories / Settings: Task 4
  - tracking runtime preserved: Task 3 + Task 6
- **Placeholder scan:** no `TODO`, `TBD`, or “similar to Task N” placeholders remain.
- **Type consistency:** `TimeRange`, `ActivityFilter`, `DaemonRequest`, `DaemonResponse`, `QueryService`, `AppState`, and page modules use the same names across all tasks.
