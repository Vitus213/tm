# TM GUI + Daemon IPC Read Path Design

## 1. Overview

This spec defines the next product iteration after the tracking foundation: a standalone desktop GUI that connects to `tm-daemon` over IPC and renders the first real Tai-like product surfaces.

This iteration delivers a **read-first vertical slice**:
- a new `crates/ui` desktop client using `eframe/egui`
- daemon-side IPC query support
- real data-backed `Overview`, `Charts`, and `Data` pages
- unified app + website query models at the IPC boundary
- stable placeholder pages for `Apps`, `Websites`, `Categories`, and `Settings`

The GUI remains a separate process. It does **not** read SQLite directly and does **not** own aggregation logic.

## 2. Current Context

The current workspace already provides the tracking foundation:
- `crates/core` owns activity/session domain rules
- `crates/storage` persists sessions to SQLite
- `crates/tracker` polls Niri focus and normalizes activity events
- `crates/ipc` currently contains only a minimal command/event contract
- `crates/daemon` runs the runtime loop, persists sessions, and flushes on focus loss/shutdown

Relevant current files:
- `crates/ipc/src/messages.rs`
- `crates/daemon/src/app.rs`
- `crates/daemon/src/session_service.rs`

The next gap is product-level presentation: there is no GUI crate yet, and no daemon query API for a client UI.

## 3. Goals

1. Add a standalone desktop GUI aligned with the Tai-style information architecture.
2. Keep `tm-daemon` as the single source of truth for persisted and aggregated tracking data.
3. Expose the first stable read-only GUI contract over IPC.
4. Ship three real pages backed by daemon queries: `Overview`, `Charts`, `Data`.
5. Preserve clean boundaries so later work can add details pages, categories, settings, exports, and browser-derived website activity without redesigning the client/server split.

## 4. Non-Goals

This iteration does **not** include:
- GUI write/edit flows
- category editing
- settings persistence from the GUI
- exports
- tray integration
- auto-starting `tm-daemon` from the GUI
- collapsing GUI and daemon into one process
- direct SQLite access from the GUI

## 5. Architecture

### 5.1 Process model

The runtime model is:
- `tm-daemon` runs as a long-lived background process
- `tm-ui` runs as a separate desktop client
- `tm-ui` connects to `tm-daemon` over IPC for all product data

The GUI must tolerate daemon absence and show a clear disconnected state instead of attempting to bootstrap the daemon itself.

### 5.2 Crate boundaries

#### `crates/daemon`
Responsibilities:
- keep the existing tracking runtime intact
- host the IPC server
- own query and aggregation logic for the GUI
- translate persisted session data into page-oriented response models

#### `crates/ipc`
Responsibilities:
- define request/response message contracts
- define page-oriented query types and response payloads
- define shared app/website summary shapes exposed to the GUI

#### `crates/ui`
Responsibilities:
- `eframe/egui` application shell
- Tai-style left navigation + content area
- page-local state and user interaction
- asynchronous IPC requests and rendering of returned view models

The UI does not implement business aggregation logic. It renders data returned by daemon.

#### `crates/storage`
Responsibilities:
- continue owning persistence and low-level read/write primitives
- optionally provide helper queries needed by daemon aggregation

It remains an internal persistence layer, not a GUI dependency.

## 6. GUI Information Architecture

### 6.1 Primary navigation

The desktop client includes these first-level entries:
- Overview
- Charts
- Data
- Apps
- Websites
- Categories
- Settings

### 6.2 Real pages in scope

#### Overview
Shows:
- total tracked time for the current range
- top apps
- top websites
- recent activity fragments
- loading, empty, error, and disconnected states

#### Charts
Shows:
- app share distribution
- website share distribution
- time trend series
- at least one stable time distribution view

#### Data
Shows:
- session table
- date range filtering
- activity-kind filtering (`app`, `website`, or all)
- subject search/filter
- loading, empty, and error states

### 6.3 Placeholder pages in scope

These pages exist as stable navigation destinations but are not fully implemented in this iteration:
- Apps
- Websites
- Categories
- Settings

They should communicate that deeper product support is planned, while preserving the final navigation skeleton.

## 7. IPC Query Design

The IPC API should be page-oriented rather than row-oriented.

### 7.1 Commands

Recommended first query commands:
- `Ping`
- `GetOverview`
- `GetSessions`
- `GetCharts`

These commands live alongside any existing operational commands rather than replacing the current IPC contract outright.

### 7.2 Response model strategy

Daemon responses should be **page view models**, not direct database rows.

That means daemon is responsible for:
- time range normalization
- app/website unification
- top-N aggregation
- chart bucketing/grouping
- empty-result stability

The GUI should not compute dashboard summaries or chart aggregates from raw sessions.

### 7.3 Shared query concepts

The read APIs should support a shared set of query concepts where applicable:
- date/time range
- activity kind (`app`, `website`, or all)
- subject filter/search
- category filter placeholder for future work

The website dimension should exist in the data model even if the current repository mostly contains app-derived sessions. This prevents a later contract break when browser tracking arrives.

## 8. Data Flow and State Management

### 8.1 Data flow

For each page:
1. user lands on a page
2. UI builds request parameters
3. UI sends IPC request to daemon
4. daemon queries/aggregates data
5. daemon returns a page response model
6. UI renders the returned state

When the user changes page or filters, the page re-requests its own data.

### 8.2 UI state model

Keep state intentionally lightweight.

Global state:
- selected navigation page
- current shared time range
- daemon connection state (`connected`, `disconnected`, `retrying`)

Per-page state:
- `loading`
- `loaded(data)`
- `empty`
- `error(message)`

No additional global event bus or cache invalidation framework is needed in this iteration.

## 9. Error Handling

Only implement error handling for real boundaries in scope.

### 9.1 Daemon unavailable
- show a clear disconnected/error panel
- instruct the user to start `tm-daemon`
- keep the GUI process alive
- do not auto-launch daemon

### 9.2 Empty results
- render explicit empty states on `Overview`, `Charts`, and `Data`
- treat empty data as valid, not exceptional

### 9.3 Query failures
- surface the error on the affected page only
- do not crash or reset the whole app shell
- allow manual retry through page refresh/reload interaction

## 10. Testing Strategy

### 10.1 `crates/ipc`
- serialization/deserialization round-trip tests
- stable coverage for new query/response message shapes

### 10.2 `crates/daemon`
- unit tests for aggregation/query helpers
- integration tests for IPC request handling
- regression coverage proving tracking runtime still works while query support is added

### 10.3 `crates/ui`
- tests for pure state transitions and response-to-view mapping
- avoid heavy snapshot-driven GUI testing in this iteration

### 10.4 End-to-end validation
Manual validation should cover:
- daemon running + GUI connects successfully
- daemon absent + GUI shows disconnected state
- Overview renders real data
- Charts renders real aggregated series
- Data renders real sessions with filters

## 11. Implementation Slices

### Slice 1: Expand IPC contract
Files likely touched:
- `crates/ipc/src/messages.rs`
- `crates/ipc/src/lib.rs`

Deliverables:
- add query commands and typed responses
- keep contract explicitly tagged and version-stable

### Slice 2: Add daemon query service + IPC server
Files likely touched:
- `crates/daemon/src/app.rs`
- new daemon IPC/query modules
- `crates/daemon/src/lib.rs`

Deliverables:
- start serving query requests without regressing tracking runtime
- return page-oriented data for overview, sessions, and charts

### Slice 3: Add UI crate shell
Files likely touched:
- new `crates/ui/Cargo.toml`
- new `crates/ui/src/main.rs`
- new `crates/ui/src/app.rs`
- new page modules

Deliverables:
- `eframe/egui` application shell
- left navigation + content region
- IPC client wiring
- disconnected/loading/error/empty states

### Slice 4: Render three real pages
Files likely touched:
- `crates/ui/src/pages/overview.rs`
- `crates/ui/src/pages/charts.rs`
- `crates/ui/src/pages/data.rs`
- related shared view/state modules

Deliverables:
- Overview, Charts, and Data render daemon-backed results
- placeholder pages exist for remaining sections

## 12. Definition of Done

This iteration is complete when:
- a new standalone `crates/ui` exists and runs
- the GUI reads data only through daemon IPC
- `Overview`, `Charts`, and `Data` are backed by real daemon responses
- the query contract already accounts for both app and website dimensions
- `Apps`, `Websites`, `Categories`, and `Settings` exist as stable placeholders
- existing tracking runtime behavior remains intact
