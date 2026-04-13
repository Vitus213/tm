# TM Runtime Verifier Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the `tm-runtime-verifier` skill into a script-backed, repeatable verifier for seeded Overview checks and live sampling checks, while clearly marking Data and Charts verification as blocked when page automation is unavailable.

**Architecture:** Keep the skill project-local and script-backed. Put shared environment/process helpers in `scripts/common.py`, implement a real seeded Overview runner first, then implement a live-sampling runner, and only then tighten the skill instructions and eval set so they reflect the real executable boundary instead of the aspirational full boundary.

**Tech Stack:** Python 3, Niri CLI, cargo, nix develop, repository-local Claude skill files

---

## File Structure

- Modify: `.claude/skills/tm-runtime-verifier/SKILL.md`
  - Align the documented scope with what the scripts can actually execute.
- Modify: `.claude/skills/tm-runtime-verifier/evals/evals.json`
  - Ensure the eval prompts reflect current v1 support and blocked areas honestly.
- Create/Modify: `.claude/skills/tm-runtime-verifier/scripts/common.py`
  - Shared helpers for repo discovery, isolated runtime setup, daemon/UI launch, IPC, screenshots, teardown.
- Create/Modify: `.claude/skills/tm-runtime-verifier/scripts/run_seeded_overview_check.py`
  - Executable seeded Overview regression check.
- Create: `.claude/skills/tm-runtime-verifier/scripts/run_live_sampling_check.py`
  - Executable live-sampling check.
- Create/Modify: `.claude/skills/tm-runtime-verifier/tests/test_common_paths.py`
  - Unit test for repo-root discovery.
- Create/Modify: `.claude/skills/tm-runtime-verifier/tests/test_seeded_overview_check.py`
  - Unit tests for seeded Overview script report shape.
- Create: `.claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py`
  - Unit test for live-sampling report shape.

## Task 1: Lock shared script assumptions with tests

**Files:**
- Modify: `.claude/skills/tm-runtime-verifier/tests/test_common_paths.py`
- Modify: `.claude/skills/tm-runtime-verifier/tests/test_seeded_overview_check.py`
- Create: `.claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py`

- [ ] **Step 1: Add the failing live-sampling report-shape test**

Write a small `unittest` that expects `run_live_sampling_check.py --dry-run --report ...` to:
- exit 0
- write a JSON file
- include `status`, `mode`, `checks`, `artifacts`, and `environment`

- [ ] **Step 2: Run the new tests and verify RED**

Run:

```bash
python .claude/skills/tm-runtime-verifier/tests/test_common_paths.py && \
python .claude/skills/tm-runtime-verifier/tests/test_seeded_overview_check.py && \
python .claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py
```

Expected: FAIL because the live script does not exist yet.

## Task 2: Implement the shared runtime helper layer

**Files:**
- Modify: `.claude/skills/tm-runtime-verifier/scripts/common.py`
- Test: `.claude/skills/tm-runtime-verifier/tests/test_common_paths.py`

- [ ] **Step 1: Keep `repo_root()` correct and tested**

Ensure it resolves to the actual `tm` repository root and not `.claude/`.

- [ ] **Step 2: Implement only the shared helpers the scripts really use**

Keep this file focused on:
- repo path discovery
- isolated runtime/data env creation
- SQLite seeding
- daemon spawn/stop
- IPC send/ping
- tm window discovery
- screenshot capture
- duration formatting for expected strings

- [ ] **Step 3: Re-run path/unit tests**

Run:

```bash
python .claude/skills/tm-runtime-verifier/tests/test_common_paths.py
```

Expected: PASS

## Task 3: Finish the seeded Overview runner

**Files:**
- Modify: `.claude/skills/tm-runtime-verifier/scripts/run_seeded_overview_check.py`
- Modify: `.claude/skills/tm-runtime-verifier/tests/test_seeded_overview_check.py`

- [ ] **Step 1: Keep the dry-run contract intact**

The existing dry-run test must still pass after real execution logic is added.

- [ ] **Step 2: Implement the real seeded Overview path**

The real path must:
- run `cargo test -p tm-ui`
- run `cargo build -p tm-ui -p tm-daemon`
- create isolated runtime and data directories
- seed deterministic `tiny-app` and `long-app` sessions
- start `tm-daemon`
- launch `tm-ui` via Niri + `nix develop`
- query Overview IPC payload
- capture an Overview screenshot
- write a JSON report with expected strings and artifact paths

- [ ] **Step 3: Run the script manually for GREEN verification**

Run:

```bash
python .claude/skills/tm-runtime-verifier/scripts/run_seeded_overview_check.py \
  --report /tmp/tmvr-seeded-overview-report.json
```

Expected: PASS report and a readable screenshot path.

- [ ] **Step 4: Read the screenshot and verify it matches the report**

Use `Read` on the screenshot file and verify it shows the expected strings from the report.

## Task 4: Add the live-sampling runner

**Files:**
- Create: `.claude/skills/tm-runtime-verifier/scripts/run_live_sampling_check.py`
- Create: `.claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py`

- [ ] **Step 1: Write the failing dry-run test**

The test should require:
- `mode = live-sampling`
- a valid report shape
- dry-run blocker or placeholder notes

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
python .claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py
```

Expected: FAIL because the script does not exist yet.

- [ ] **Step 3: Implement the minimal live-sampling script**

The first real version must:
- create isolated XDG data/runtime locations
- preserve access to the real compositor socket
- start `tm-daemon`
- ping the socket
- observe at least one focus snapshot or persisted sampled session
- emit a JSON report with status, checks, artifacts, and notes

- [ ] **Step 4: Re-run the live-sampling test and then one real execution**

Run:

```bash
python .claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py && \
python .claude/skills/tm-runtime-verifier/scripts/run_live_sampling_check.py \
  --report /tmp/tmvr-live-sampling-report.json
```

Expected: tests pass; real execution returns PASS or a truthful BLOCKED with a concrete reason.

## Task 5: Bring the skill document and evals back into sync

**Files:**
- Modify: `.claude/skills/tm-runtime-verifier/SKILL.md`
- Modify: `.claude/skills/tm-runtime-verifier/evals/evals.json`

- [ ] **Step 1: Narrow the documented v1 scope**

Update the skill text so it clearly distinguishes:
- stable v1 support: seeded Overview + live sampling
- conditional/blocked areas: Data and Charts when no page automation exists

- [ ] **Step 2: Update eval prompts to match the real executable boundary**

Prompts can still mention Data/Charts, but expectations must require the skill to report `BLOCKED` honestly rather than pretending full automation exists.

- [ ] **Step 3: Manually review the skill for consistency**

Verify the documented output format, script names, and blocker behavior match the actual scripts.

## Task 6: Re-run iteration-1 with the tightened skill

**Files:**
- Modify outputs under `.claude/skills/tm-runtime-verifier-workspace/iteration-1/` as needed

- [ ] **Step 1: Re-run one with-skill seeded regression prompt**

Use the duration regression eval first because it already proved the most concrete value.

- [ ] **Step 2: Capture artifacts and summarize gaps**

Save:
- final markdown report
- JSON artifact manifest
- seeded Overview JSON report
- screenshot path

- [ ] **Step 3: Record the current blocked areas honestly**

If Data/Charts still cannot be automated, the report must say so explicitly.

## Commit guidance

- [ ] **Step 1: Commit script-backed verifier changes**

```bash
git add \
  .claude/skills/tm-runtime-verifier/SKILL.md \
  .claude/skills/tm-runtime-verifier/evals/evals.json \
  .claude/skills/tm-runtime-verifier/scripts/common.py \
  .claude/skills/tm-runtime-verifier/scripts/run_seeded_overview_check.py \
  .claude/skills/tm-runtime-verifier/scripts/run_live_sampling_check.py \
  .claude/skills/tm-runtime-verifier/tests/test_common_paths.py \
  .claude/skills/tm-runtime-verifier/tests/test_seeded_overview_check.py \
  .claude/skills/tm-runtime-verifier/tests/test_live_sampling_check.py \
  docs/superpowers/specs/2026-04-13-tm-runtime-verifier-skill-design.md \
  docs/superpowers/plans/2026-04-13-tm-runtime-verifier-skill.md

git commit -m "feat: add tm runtime verifier skill scaffold"
```
