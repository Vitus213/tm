# TM Runtime Verifier Skill Design

## 1. Overview

This spec defines a repository-local Claude skill for the `tm` project that standardizes runtime verification. The skill exists because manual testing of `tm-ui` and `tm-daemon` had already converged into a repeatable workflow:
- build and test the relevant crates
- prepare isolated runtime and data directories
- seed deterministic SQLite session data for regression checks
- launch `tm-daemon`
- launch `tm-ui` through the real Niri/Wayland desktop session
- capture screenshots
- compare visible UI output against IPC payloads
- optionally validate live focus sampling on the current desktop session

The skill is project-specific. It is not intended to be a generic GUI testing framework.

## 2. Problem Statement

The project now has a meaningful split between:
- `tm-daemon` as the runtime and IPC authority
- `tm-ui` as the presentation layer

That split creates a recurring verification problem: compilation and unit tests prove only partial correctness. They do not prove that:
- the GUI renders the same values the daemon exposes over IPC
- duration formatting appears correctly in actual screenshots
- seeded session data produces the expected on-screen strings
- live desktop sampling still works in the real compositor session

Ad-hoc manual verification also drifted into an unstable pattern. Different runs reinvented the same steps, page switching was environment-sensitive, and artifacts were not always captured in a machine-readable format. The skill should reduce that drift.

## 3. Goals

1. Provide a single tm-specific verification workflow for Claude to use when the user asks to test runtime behavior.
2. Produce stable artifacts:
   - JSON reports
   - screenshot paths
   - concise pass/fail/blocked summaries
3. Verify seeded Overview regressions against known IPC payloads.
4. Verify real live-sampling behavior against the current Niri desktop session.
5. Make environment blockers explicit instead of silently degrading into vague “manual” checks.

## 4. Non-Goals

This skill does **not** aim to:
- replace unit or integration tests in Rust crates
- become a reusable generic Wayland GUI testing library
- guarantee fully automated page navigation for every `tm-ui` page
- validate arbitrary visual aesthetics or layout polish
- introduce invasive test-only hooks into production crates just to enable the skill

## 5. Current Scope Boundary

### 5.1 In-scope for v1

The first converged version of the skill supports:

#### Seeded Overview verification
- run `cargo test -p tm-ui`
- run `cargo build -p tm-ui -p tm-daemon`
- create isolated XDG runtime/data directories
- seed deterministic SQLite data with:
  - one sub-minute sample expected to render as `0 min`
  - one hour-plus sample expected to render as `Hh Mm`
- start `tm-daemon`
- start `tm-ui` through `niri msg action spawn-sh` and `nix develop`
- capture an Overview screenshot
- query Overview IPC payload
- write a machine-readable JSON report that includes expected strings and artifact paths

#### Live sampling verification
- run `tm-daemon` against an isolated database while attached to the real compositor session
- verify socket availability and ping response
- observe at least one real sampled session or focus transition
- persist the captured results into JSON artifacts
- report whether the desktop sampling path is functioning on this machine

### 5.2 Conditionally blocked in v1

The following are part of the long-term skill surface, but are not guaranteed to be fully automated in the current environment:

#### Data page verification
Blocked when the environment lacks non-interactive page-switching capability.

#### Charts page verification
Blocked when the environment lacks non-interactive page-switching capability.

The skill must report these as `BLOCKED` with an explicit reason rather than pretending they passed.

## 6. Root Cause Behind the Skill

The skill is motivated by a clear execution root cause:
- natural-language workflow instructions alone were not enough
- agents kept rediscovering the same environment constraints
- screenshot capture and artifact writing were inconsistent
- page navigation on Wayland depended on tooling that is not always present

Therefore the skill must not remain “documentation only.” It needs script-backed execution for the stable parts of the workflow.

## 7. Architecture

## 7.1 Skill structure

The skill lives in:
- `.claude/skills/tm-runtime-verifier/`

Proposed structure:

```text
.claude/skills/tm-runtime-verifier/
├── SKILL.md
├── evals/
│   └── evals.json
├── scripts/
│   ├── common.py
│   ├── run_seeded_overview_check.py
│   └── run_live_sampling_check.py
└── tests/
    ├── test_common_paths.py
    └── test_seeded_overview_check.py
```

## 7.2 Script responsibilities

### `scripts/common.py`
Shared runtime utilities:
- repository root discovery
- isolated runtime/data environment construction
- socket path helpers
- SQLite seeding helpers
- UI launch helpers
- Niri window lookup and screenshot capture helpers
- process teardown helpers
- duration formatting helper for expected display strings

### `scripts/run_seeded_overview_check.py`
Stable seeded regression runner.

Input:
- report output path
- optional dry-run mode

Output:
- JSON report containing:
  - status
  - mode
  - build/test check results
  - environment details
  - IPC payload
  - screenshot path
  - expected strings
  - notes

### `scripts/run_live_sampling_check.py`
Live sampling runner.

Input:
- report output path
- optional dry-run mode

Output:
- JSON report containing:
  - socket/ping validation
  - observed focus or session payloads
  - optional screenshot path
  - blocker reason if the session cannot be validated

## 8. Data and Artifact Model

Every executable script should emit a JSON report. That report should be sufficient to audit what happened without re-reading the whole terminal transcript.

### Seeded Overview report should include
- `status`
- `mode = seeded-overview`
- `checks`
- `artifacts`
- `environment`
- `expected_strings`
- `notes`

### Live sampling report should include
- `status`
- `mode = live-sampling`
- `checks`
- `artifacts`
- `environment`
- `observed_sessions` or `observed_focus`
- `notes`

## 9. Triggering Guidance

The skill should trigger when the user asks for things like:
- “测试 tm-ui / tm-daemon 的真实运行”
- “跑一遍 runtime 验证”
- “看截图里的字符对不对”
- “验证 Overview 是不是和后端一致”
- “做一次 seeded 回归”
- “做一次真实采样检查”

It should be preferred over generic build/run behavior whenever the user cares about runtime correctness rather than mere compilation.

## 10. Verification Strategy

### 10.1 Script-level tests
- `test_common_paths.py` verifies that repository discovery points at the real `tm` root.
- `test_seeded_overview_check.py` verifies dry-run report shape.
- Additional tests should cover real report field population where practical without needing the full compositor.

### 10.2 Runtime validation
The runtime proof remains artifact-based:
- script emits JSON
- screenshot is captured
- screenshot is read by Claude
- IPC payload and expected strings are compared against what the screenshot shows

## 11. Risk and Constraints

### Environment constraints
The largest constraint is Wayland input automation. This environment currently lacks tools like:
- `wtype`
- `ydotool`
- `dotool`
- `wlrctl`

So any page verification that depends on switching from the default Overview page is not yet reliably scriptable.

### Process collision risk
The skill must avoid clobbering an already-running user daemon or reusing stale `tm` windows. It should prefer isolated XDG paths and explicitly identify newly launched `tm` windows.

## 12. Expected v1 Outcome

After the first implementation pass, the skill should be considered successful if:
- seeded Overview verification is one-command repeatable and artifact-rich
- live sampling verification is script-backed
- Data and Charts are explicitly surfaced as blocked when automation is unavailable
- the skill no longer relies on ad-hoc rediscovery of environment quirks during each run
