# Duration Display Formatting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update UI duration formatting so durations under one minute still display as `0 min`, while durations of one hour or more display as `Hh Mm` instead of large raw minute counts.

**Architecture:** Add one small shared duration-formatting helper inside the `tm-ui` crate and use it from the existing Overview and Data pages. Cover the formatter with focused unit tests first, then update the rendering code to call the helper without changing the IPC payload shape or page state flow.

**Tech Stack:** Rust, eframe/egui, existing `tm-ui` crate tests

---

## File Structure

- Modify: `crates/ui/src/lib.rs`
  - Export the new formatting module so tests and page renderers can reuse it.
- Create: `crates/ui/src/format.rs`
  - Hold a single `format_duration_minutes_style(seconds: i64) -> String` helper with unit tests.
- Modify: `crates/ui/src/pages/overview.rs`
  - Replace inline integer-minute formatting with the shared helper for total time, top apps/websites, and recent activity.
- Modify: `crates/ui/src/pages/data.rs`
  - Replace inline integer-minute formatting in the sessions grid with the shared helper.

## Task 1: Add formatter with test-first flow

**Files:**
- Create: `crates/ui/src/format.rs`
- Modify: `crates/ui/src/lib.rs`
- Test: `crates/ui/src/format.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::format_duration_minutes_style;

    #[test]
    fn formats_sub_minute_durations_as_zero_minutes() {
        assert_eq!(format_duration_minutes_style(0), "0 min");
        assert_eq!(format_duration_minutes_style(45), "0 min");
        assert_eq!(format_duration_minutes_style(59), "0 min");
    }

    #[test]
    fn formats_minute_range_durations_as_whole_minutes() {
        assert_eq!(format_duration_minutes_style(60), "1 min");
        assert_eq!(format_duration_minutes_style(65), "1 min");
        assert_eq!(format_duration_minutes_style(3599), "59 min");
        assert_eq!(format_duration_minutes_style(59 * 60), "59 min");
    }

    #[test]
    fn formats_hour_plus_durations_as_hours_and_minutes() {
        assert_eq!(format_duration_minutes_style(60 * 60), "1h 0m");
        assert_eq!(format_duration_minutes_style(61 * 60), "1h 1m");
        assert_eq!(format_duration_minutes_style((2 * 60 + 5) * 60), "2h 5m");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tm-ui`
Expected: FAIL because the formatter module/function does not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
pub fn format_duration_minutes_style(seconds: i64) -> String {
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes} min");
    }

    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    format!("{hours}h {remaining_minutes}m")
}
```

Also export the module from `crates/ui/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tm-ui`
Expected: PASS

## Task 2: Wire formatter into Overview and Data pages

**Files:**
- Modify: `crates/ui/src/pages/overview.rs`
- Modify: `crates/ui/src/pages/data.rs`
- Test: `crates/ui/src/format.rs`

- [ ] **Step 1: Update Overview rendering to use shared formatting**

Replace the four inline `... / 60` display expressions with the formatter:

```rust
ui.label(format!("Tracked: {}", format_duration_minutes_style(payload.total_seconds)));
```

and equivalent replacements for top apps, top websites, and recent activity.

- [ ] **Step 2: Update Data page rendering to use shared formatting**

Replace:

```rust
ui.label(format!("{} min", row.duration_seconds / 60));
```

with:

```rust
ui.label(format_duration_minutes_style(row.duration_seconds));
```

- [ ] **Step 3: Run focused tests**

Run: `cargo test -p tm-ui`
Expected: PASS

## Task 3: Verify with real runtime evidence

**Files:**
- No code changes required

- [ ] **Step 1: Run workspace verification for touched crate**

Run: `cargo test -p tm-ui && cargo build -p tm-ui`
Expected: PASS

- [ ] **Step 2: Re-run live GUI verification**

Run the same real-session flow used in manual verification:
- start `target/debug/tm-daemon`
- generate at least one sub-minute sample so the UI must show `0 min`
- generate at least one >=60 minute-equivalent or synthetic testable duration so the UI must show `Hh Mm`
- launch `tm-ui` in `nix develop`
- capture a screenshot of the Overview page
- switch to the Data page and capture a second screenshot
- verify the screenshots text matches backend IPC values (`0 min`, `N min`, `Hh Mm` as appropriate)

Expected: Overview and Data screenshots agree with IPC values.

## Commit guidance

- [ ] **Step 1: Commit formatter change**

```bash
git add crates/ui/src/lib.rs crates/ui/src/format.rs crates/ui/src/pages/overview.rs crates/ui/src/pages/data.rs
git commit -m "fix: format long activity durations in ui"
```
