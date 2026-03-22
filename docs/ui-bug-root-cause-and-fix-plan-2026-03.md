# UI Bug Root Cause And Fix Plan (2026-03-20, Phase 17 Refresh)

## Scope

- This document records the remaining unplanned UI bugs and polish items that still need one unified repair pass on the current `Phase 17` baseline.
- This pass does not change product code. It only updates the implementation-ready plan.
- Current code baseline referenced in this document:
  - `crates/fc-ui-slint/src/app.rs`
  - `crates/fc-ui-slint/src/state.rs`
  - `crates/fc-ui-slint/src/ui_palette.slint`
  - `docs/architecture.md`
- The legacy issue `Provider Settings modal header height changes when switching Mock / OpenAI-compatible` is intentionally removed from this plan.
  - `Phase 17B` replaced `Provider Settings` with the broader `Settings` center.
  - `Phase 17B fix-1` already stabilized that modal container contract.
  - This document now covers only the remaining unresolved workspace/diff-shell issues from the original report.

## Cross-cutting observations

### 1. The workbench top stack is still asymmetric between `Diff` and `Analysis`

- `Diff` and `Analysis` still share one workbench shell, but they do not reserve the same top-layer structure.
- `Diff` only renders the helper strip when line rows are already available.
- `Analysis` always reserves a helper strip.
- `Phase 17` added `stale-selection`, which widened the number of non-ready states that now fall into the `DiffStateShell` path without closing the original structural gap.

Relevant code:

- `crates/fc-ui-slint/src/app.rs:1208-1221`
- `crates/fc-ui-slint/src/app.rs:2078-2160`
- `crates/fc-ui-slint/src/app.rs:2402-2524`
- `crates/fc-ui-slint/src/state.rs:347-478`
- `crates/fc-ui-slint/src/state.rs:1088-1292`

### 2. Embedded shell states still reuse a standalone visual primitive

- `DiffStateShell` is still rendered as a full state card even when embedded inside the bordered workbench panel.
- `StatusPill` still behaves best inside layout-driven lanes, but `DiffStateShell` places it by absolute coordinates.
- This keeps the shell visually heavier than the rest of the workbench chrome in no-selection, stale-selection, loading, and unavailable states.

Relevant code:

- `crates/fc-ui-slint/src/app.rs:194-260`
- `crates/fc-ui-slint/src/app.rs:375-505`

### 3. Geometry is still duplicated in two places that should share one contract

- The workbench surface still uses an outer card plus an inner bordered panel.
- The diff table header and diff table body still define their own column geometry separately.
- The visible result is still the same class of drift as the original report:
  - extra shell chrome
  - loading-mask boundary mismatch
  - header/body separator misalignment

Relevant code:

- `crates/fc-ui-slint/src/app.rs:2033-2065`
- `crates/fc-ui-slint/src/app.rs:2208-2277`
- `crates/fc-ui-slint/src/app.rs:2292-2378`
- `crates/fc-ui-slint/src/app.rs:2850-2858`

## Itemized analysis

### A. Diff shell state card still feels oversized, accent-heavy, and visually misaligned

Screenshots:

- Figure 1

Current implementation:

- `StatusPill`: `crates/fc-ui-slint/src/app.rs:194-260`
- `DiffStateShell`: `crates/fc-ui-slint/src/app.rs:375-505`
- Embedded `Diff` usage: `crates/fc-ui-slint/src/app.rs:2150-2158`
- Embedded `Analysis` usage: `crates/fc-ui-slint/src/app.rs:2524-2532`
- Diff shell state mapping: `crates/fc-ui-slint/src/state.rs:347-478`

Root cause:

- `DiffStateShell` still renders a full-height `6px` accent bar even when `embedded: true`, but the shell already sits inside the bordered workbench panel. This doubles the perceived left edge.
- The header pill is still positioned by `x/y` only (`StatusPill` inside `DiffStateShell`), not by a wrapping layout lane. Its width behavior is therefore not tuned for this placement pattern.
- `neutral` state is still remapped to `info` inside the shell header badge (`tone: root.tone == "neutral" ? "info" : root.tone`), so `No Selection` is more visually active than the actual state semantics justify.
- `Phase 17` added `stale-selection`, which now routes another quiet state through the same heavy shell treatment.

Recommended fix:

- Split `DiffStateShell` into two explicit presentation modes:
  - standalone state card
  - embedded workbench shell
- For the embedded mode:
  - reduce or remove the full-height accent bar for `neutral` and possibly `warn` states
  - keep the shell left edge visually subordinate to the workbench border
- Replace absolute pill placement with a layout-driven header lane and keep the pill width content-driven.
- Stop remapping `neutral` to `info` for embedded shell badges.
- Re-check `stale-selection` after the tone reduction. It should stay noticeable, but it should not read like a blocking warning banner.

Acceptance criteria:

- `No Selection` and `Stale` badges read as compact badges, not as stretched header chips.
- The shell left edge no longer looks thicker than the workbench border.
- `No Selection` reads as neutral.
- `Stale` reads as cautionary but not over-emphasized.
- Embedded shell and ready-state content keep the same horizontal alignment.

### B. Diff detail header separators are still not aligned with row separators

Screenshots:

- Figure 2

Current implementation:

- Diff table shell and header: `crates/fc-ui-slint/src/app.rs:2196-2277`
- Diff body rows: `crates/fc-ui-slint/src/app.rs:2279-2384`

Root cause:

- The header still uses `HorizontalLayout { padding: 5px; }`, but the body rows still start from a different geometry.
- Header separators still use shortened heights (`parent.height - 6px`), while the body separators use full row height.
- The header and body still compute their separator positions in separate layout trees instead of sharing one column geometry source.
- Because the diff header is mirrored with `viewport-x`, any fixed x drift becomes consistently visible during horizontal scroll instead of being hidden.

Recommended fix:

- Define one shared diff-column contract and reuse it in both header and body:
  - old-line column width
  - new-line column width
  - marker column width
  - separator x positions
  - content-start x position
- Remove header-only padding unless the body rows adopt the exact same inset.
- If a shortened separator height is visually preferred in the header, keep the same x positions and only vary height, not column math.
- Prefer one reusable component or explicit shared geometry properties over duplicated layout fragments.

Acceptance criteria:

- The vertical dividers under `old`, `new`, and `content` line up exactly with the body rows.
- Hunk rows and regular rows start from the same content baseline.
- Horizontal scrolling preserves perfect header/body column sync.

### C. Workspace panel is still visually double-carded and smaller than necessary

Screenshots:

- Figure 3

Current implementation:

- Outer workspace card: `crates/fc-ui-slint/src/app.rs:2033-2037`
- Inner host inset: `crates/fc-ui-slint/src/app.rs:2042-2054`
- Inner workbench panel: `crates/fc-ui-slint/src/app.rs:2056-2065`
- Workspace loading mask mount: `crates/fc-ui-slint/src/app.rs:2850-2858`
- Loading-mask visibility projection: `crates/fc-ui-slint/src/app.rs:3507-3534`

Root cause:

- The workspace still uses an outer `SectionCard` plus an inner rounded/bordered `workbench_panel`.
- `workbench_host` still adds fixed insets (`x: 10px`, `y: 8px`, reduced width/height), which makes the usable file-view surface smaller than the available right-column space.
- The loading mask is still mounted on the outer workspace card, so during long operations the overlay exposes the outer shell bounds rather than the inner surface users perceive as the actual workbench.
- This is why the workspace still reads as more padded and more chrome-heavy than the sidebar, especially in loading state.

Recommended fix:

- Collapse the workspace to one primary visible surface.
- Preferred direction:
  - make the outer workspace wrapper transparent or borderless
  - keep the inner workbench panel as the primary shell
- Reduce non-essential host insets so the content area expands closer to the sidebar rhythm.
- Mount the loading mask against the same surface users perceive as the actual workbench panel.
- Keep tabs, panel border, and content surface reading as one connected shell rather than nested cards.

Acceptance criteria:

- The workspace uses more of the available right-column area without changing IA.
- The border/background weight feels consistent with the sidebar.
- During loading, the mask boundary matches the perceived workbench surface.
- The tab row and content panel still read as one connected surface after the simplification.

### D. `Diff` and `Analysis` top regions are still structurally inconsistent in non-ready states

Screenshots:

- Figure 4

Current implementation:

- Diff header metadata row: `crates/fc-ui-slint/src/app.rs:2089-2130`
- Diff shell branch: `crates/fc-ui-slint/src/app.rs:2150-2158`
- Diff helper strip branch: `crates/fc-ui-slint/src/app.rs:2160-2194`
- Analysis header metadata row: `crates/fc-ui-slint/src/app.rs:2413-2455`
- Analysis helper strip: `crates/fc-ui-slint/src/app.rs:2493-2522`
- Diff header/state text derivation: `crates/fc-ui-slint/src/state.rs:379-478`
- Analysis header/state text derivation: `crates/fc-ui-slint/src/state.rs:1101-1292`

Root cause:

- `Diff` and `Analysis` still share the same `workbench_header_height`, but they still do not reserve the same top-stack layers.
- In `Diff`, the badge row remains conditional because `has_selected_result` gates the mode/status pills.
- In `Diff`, the helper strip still appears only when `root.diff_shell_ready && root.diff_has_rows`.
- In `Analysis`, the badge row is always populated and the helper strip is always present, including no-selection and stale-selection states.
- `Phase 17` made this more visible because stale-selection is now a first-class non-ready state in both tabs, but only `Analysis` preserves the full top-stack structure during that state.

Recommended fix:

- Define one stable workbench top-stack contract shared by both tabs:
  - title row
  - metadata/badge row
  - helper strip
  - shell/content body
- In `Diff`, always reserve the helper-strip height, even when the tab is in:
  - `no-selection`
  - `stale-selection`
  - `loading`
  - `unavailable`
  - `error`
- For non-ready `Diff` states, render neutral helper-strip copy instead of removing the strip.
- Standardize badge lanes so `Diff` and `Analysis` keep the same text baseline even when some badges are absent:
  - either always render a fixed mode badge lane
  - or reserve placeholder geometry for missing badges
- Keep the shell body start position identical between `Diff` and `Analysis` for matching non-ready states.

Acceptance criteria:

- Switching between `Diff` and `Analysis` does not shift the shell body vertically in `no-selection` or `stale-selection`.
- `Diff` keeps a helper strip in all states, with neutral copy when detailed rows are not present.
- Header badge/text baselines remain stable regardless of selection state.

## Recommended implementation order

1. Fix shared geometry first:
   - item B
   - item D
2. Simplify the workbench shell:
   - item C
3. Tune embedded shell visuals on top of the simplified shell:
   - item A

Reason:

- `B` and `D` are structural alignment bugs and will affect every visual review.
- `C` removes the extra shell layer that currently amplifies several misalignments.
- `A` is easier to tune after the workbench geometry is stable.

## Suggested regression checklist

- Open the app with no selection and switch `Diff` / `Analysis` repeatedly.
- Trigger stale-selection by changing `Search`, `Status`, or `Settings -> Behavior` visibility so the currently opened row drops out of the visible set.
- Confirm top baseline stability in both:
  - `no-selection`
  - `stale-selection`
  - `loading`
- Run compare and inspect the workspace loading-mask boundary.
- Open one `different` text file and verify header/body divider alignment while horizontally scrolling.
- Open `left-only`, `right-only`, and `equal` rows and confirm preview-mode shell alignment remains consistent.
- Re-check narrow and wide window widths because several current issues are layout-distribution problems, not only static geometry problems.

## Out of scope for the later fix pass

- No IA change.
- No redesign of compare workflow.
- No new theme system.
- No change to AI business logic or settings persistence format.
- No reopening of the superseded legacy `Provider Settings` modal bug.
