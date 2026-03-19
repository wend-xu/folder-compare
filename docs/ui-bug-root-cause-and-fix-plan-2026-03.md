# UI Bug Root Cause And Fix Plan (2026-03-18)

## Scope

- This document records root-cause analysis and recommended fixes for the current unplanned UI bugs and polish items raised from screenshots 1-5.
- This pass does not change product code. It only turns the findings into an implementation-ready document for later execution.
- Current code baseline referenced in this document:
  - `crates/fc-ui-slint/src/app.rs`
  - `crates/fc-ui-slint/src/state.rs`
  - `crates/fc-ui-slint/src/ui_palette.slint`

## Cross-cutting observations

### 1. Workspace top area does not use one stable contract

- `Diff` and `Analysis` both live under the same workbench shell, but their top stacks are not structurally aligned.
- `Diff` header metadata is partly conditional, and its helper strip only appears when diff rows are already rendered.
- `Analysis` always keeps a header metadata row plus a helper strip.
- Result: when users switch tabs or land on no-selection states, the visual start line of the shell body moves.

Relevant code:

- `crates/fc-ui-slint/src/app.rs:1603-1718`
- `crates/fc-ui-slint/src/app.rs:1926-2048`
- `crates/fc-ui-slint/src/app.rs:1018-1030`

### 2. Several visual primitives are reused outside their original layout assumptions

- `StatusPill` works correctly inside `HorizontalLayout`, but it is also used as an absolutely-positioned standalone element inside `DiffStateShell`.
- `DiffStateShell` was designed as a self-contained state card, but it is now embedded inside an already-bordered workbench panel.
- Result: pill width, accent bar thickness, and shell emphasis are visually over-amplified in embedded states.

Relevant code:

- `crates/fc-ui-slint/src/app.rs:194-260`
- `crates/fc-ui-slint/src/app.rs:302-424`

### 3. Some header/body geometries are duplicated instead of shared

- The diff table header and diff table rows define their own separators and padding independently.
- The workspace surface is also composed from an outer `SectionCard` plus an inner workbench panel instead of one shared surface definition.
- Result: alignment drift and "double card" visuals appear in normal and loading states.

Relevant code:

- `crates/fc-ui-slint/src/app.rs:1558-1590`
- `crates/fc-ui-slint/src/app.rs:1748-1800`
- `crates/fc-ui-slint/src/app.rs:1841-1900`

## Itemized analysis

### A. Provider Settings modal header height becomes abnormal when switching `Mock` and `OpenAI-compatible`

Screenshots:

- Figure 1

Current implementation:

- Modal outer shell: `crates/fc-ui-slint/src/app.rs:2540-2557`
- Modal content layout: `crates/fc-ui-slint/src/app.rs:2558-2695`
- Mode switch: `crates/fc-ui-slint/src/app.rs:2577-2606`
- Remote-only fields block: `crates/fc-ui-slint/src/app.rs:2632-2678`

Root cause:

- The modal height is hard-switched between two fixed values: `430px` and `338px`.
- The whole modal body sits in one `VerticalLayout`, but the remote-only block is toggled by `visible` instead of being structurally inserted/removed with `if`.
- In practice this means the layout must re-resolve the same vertical stack under two different container heights while one branch is only visibility-switched. That makes title/subtitle/form spacing sensitive to the current mode and to Slint's layout distribution behavior.
- The footer is not anchored independently from the content body, so any vertical redistribution shows up as "header height changed" instead of as a controlled body resize.

Recommended fix:

- Split modal into three stable regions:
  - fixed header
  - content body
  - fixed footer
- Replace the remote-only `VerticalLayout { visible: ... }` with structural branching:
  - `if provider_settings_mode == 1` render remote fields
  - otherwise render nothing
- Stop using two magic container heights as the primary layout tool. Prefer one of these two approaches:
  - derive modal height from content preferred height plus fixed header/footer paddings
  - or keep one fixed modal height and let only the body scroll or stretch
- Keep the title, subtitle, top separator, and footer buttons at stable y positions across both provider modes.

Acceptance criteria:

- Switching between `Mock` and `OpenAI-compatible` does not move the title/subtitle baseline.
- The first form row always starts at the same y position.
- The footer buttons stay visually anchored to the modal bottom.
- Error text appearance does not reflow the header region.

### B. Diff shell state card feels oversized, left accent looks misaligned, top state pill overflows

Screenshots:

- Figure 2

Current implementation:

- `StatusPill`: `crates/fc-ui-slint/src/app.rs:194-260`
- `DiffStateShell`: `crates/fc-ui-slint/src/app.rs:302-424`
- Embedded usage in `Diff`: `crates/fc-ui-slint/src/app.rs:1674-1682`
- Diff shell state mapping: `crates/fc-ui-slint/src/state.rs:270-497`

Root cause:

- `DiffStateShell` still renders a full-height `6px` accent bar even when `embedded: true`, but the shell is already placed inside the bordered workbench panel. This creates a doubled left edge and makes the accent look thicker than intended.
- `StatusPill` has `min-width` but no explicit intrinsic width contract for absolute placement. Inside `DiffStateShell` it is positioned by `x/y` only, not by a wrapping layout. That makes the pill stretch much wider than the compact badge style used elsewhere.
- `DiffStateShell` also remaps `neutral` to `info` for the header pill (`tone: root.tone == "neutral" ? "info" : root.tone`), which makes the no-selection state visually louder than the actual state severity.
- The result is a shell that reads like an emphasized alert card instead of a calm empty/default state.

Recommended fix:

- Introduce two shell variants or one explicit embedded style branch:
  - standalone state card
  - embedded workspace shell
- For embedded shell mode:
  - reduce or remove the full-height accent bar in neutral/no-selection states
  - keep the accent aligned with the workbench border instead of stacking on top of it
- Change the shell header pill from absolute placement to a `HorizontalLayout`, and give the pill a width that is content-driven.
- Do not remap neutral state to info in the badge. Keep no-selection visually neutral.
- If the current shell still feels too heavy after geometry fixes, reduce one more layer of chrome:
  - either the accent bar
  - or the header fill
  - but not both at full emphasis in the no-selection state

Acceptance criteria:

- `No Selection` badge width hugs content instead of spanning most of the row.
- The shell left edge no longer looks thicker than the surrounding panel border.
- No-selection state reads as neutral, not as an active info banner.
- Embedded shell and selected-content shell keep the same horizontal alignment.

### C. Diff detail header separators are not aligned with row separators

Screenshots:

- Figure 3

Current implementation:

- Header geometry: `crates/fc-ui-slint/src/app.rs:1748-1800`
- Body row geometry: `crates/fc-ui-slint/src/app.rs:1816-1900`

Root cause:

- The header uses `HorizontalLayout { padding: 5px; }`, but the body rows do not use the same left/right inset.
- Header separators use `height: parent.height - 6px`, while row separators use full row height.
- The columns are conceptually shared, but the x positions are recomputed by two different layout trees.
- This creates visible drift between:
  - header vertical lines
  - row vertical lines
  - hunk/content start edge

Recommended fix:

- Extract one shared diff-column geometry contract and reuse it in both header and body:
  - number column width
  - marker column width
  - separator x positions
  - content start inset
- Remove header-only padding unless the exact same inset is also applied to every body row.
- Use the same separator height strategy in header and body. If reduced header separator height is required, its x position must still match the body separators exactly.
- Prefer one reusable component or at least one explicit set of geometry properties instead of duplicating layout math in two places.

Acceptance criteria:

- The three vertical dividers under `old/new/content` line up exactly with the body rows.
- Horizontal scrolling keeps header and body in sync without any extra x offset.
- Hunk rows and regular rows start on the same content column baseline.

### D. Workspace panel is visually smaller than necessary, double-carded, and loading state exposes that mismatch

Screenshots:

- Figure 4

Current implementation:

- Outer workspace container: `crates/fc-ui-slint/src/app.rs:1558-1563`
- Inner host inset: `crates/fc-ui-slint/src/app.rs:1567-1579`
- Inner workbench panel: `crates/fc-ui-slint/src/app.rs:1581-1590`
- Loading mask component: `crates/fc-ui-slint/src/app.rs:262-300`
- Workspace loading-mask mount: `crates/fc-ui-slint/src/app.rs:2375-2382`
- Loading-mask visibility projection: `crates/fc-ui-slint/src/app.rs:2851-2883`

Root cause:

- The workspace is wrapped by an outer `SectionCard`, but inside it there is another bordered and rounded `workbench_panel`.
- `workbench_host` also adds fixed insets (`x: 10px`, `y: 8px`, width/height reduced accordingly), which further shrinks the usable workspace area.
- In normal view this reads as extra chrome; in loading view the mask makes the true surface bounds easier to notice, so the workspace feels smaller and less aligned with the sidebar rhythm.
- The sidebar cards are direct content surfaces, while the workspace is a card-inside-card composition. That is why the visual weight differs.

Recommended fix:

- Simplify workspace shell to one primary visual surface.
- Preferred direction:
  - make the outer workspace wrapper transparent or borderless
  - keep the inner workbench panel as the actual surface
- Reduce non-essential host insets so the workbench uses more of the available width and height.
- Mount the loading mask against the same surface that users perceive as the actual workspace panel.
- Keep sidebar and workspace using the same card logic at the same visual hierarchy level.

Acceptance criteria:

- Workspace visible area increases without changing IA.
- The workspace border/background no longer feels heavier than the sidebar cards.
- During loading, the mask boundary matches the perceived workbench surface instead of exposing an extra outer card.
- Tab row, panel edge, and content area feel like one coherent surface.

### E. Diff header helper strip is missing in no-selection states, making Diff/Analysis top regions inconsistent

Screenshots:

- Figure 5

Current implementation:

- Diff header metadata row: `crates/fc-ui-slint/src/app.rs:1614-1654`
- Diff shell branch: `crates/fc-ui-slint/src/app.rs:1674-1684`
- Diff helper strip branch: `crates/fc-ui-slint/src/app.rs:1684-1718`
- Analysis header metadata row: `crates/fc-ui-slint/src/app.rs:1945-1979`
- Analysis helper strip: `crates/fc-ui-slint/src/app.rs:2017-2045`
- Diff header text/state derivation: `crates/fc-ui-slint/src/state.rs:298-497`
- Analysis header text/state derivation: `crates/fc-ui-slint/src/state.rs:775-972`

Root cause:

- `Diff` and `Analysis` use the same `workbench_header_height`, but they do not reserve the same structural layers.
- In `Diff`, the metadata row conditionally hides pills when there is no selected result.
- In `Diff`, the helper strip only exists when `root.diff_shell_ready && root.diff_has_rows`.
- In `Analysis`, the header badges are always present and the helper strip is always rendered.
- Because the helper strip is structurally absent in one tab/state and present in the other, the shell body starts at different y positions. This is why switching tabs makes the shell state look like it moved down or up.

Recommended fix:

- Define one stable workbench top-stack contract shared by both tabs:
  - title row
  - metadata/badge row
  - helper strip
  - content/shell body
- For `Diff`, always reserve the helper strip height, even in no-selection/unavailable/loading states.
- When no file is selected, show neutral helper-strip copy instead of removing the strip.
- Standardize badge lanes:
  - either always render a fixed mode badge + state badge + optional provider/status badge
  - or reserve placeholder space so the text baseline stays stable
- Keep `Diff` and `Analysis` shell body start positions identical when both are in no-selection state.

Acceptance criteria:

- Switching between `Diff` and `Analysis` does not change the shell body's top baseline in no-selection state.
- Diff helper strip stays present with neutral copy even before a file is selected.
- Header badge/text baselines remain stable whether a result is selected or not.

## Recommended implementation order

1. Fix structural geometry first:
   - item C
   - item E
   - item D
2. Then fix component-level visual noise:
   - item B
3. Finish with modal stabilization:
   - item A

Reason:

- C and E are direct alignment and baseline issues.
- D removes the extra shell layer that currently amplifies several visual inconsistencies.
- B becomes easier to tune after the surrounding shell geometry is simplified.
- A is isolated and low-risk once the workbench fixes are no longer changing shared primitives.

## Suggested regression checklist

- Open app with no compare result and switch `Diff` / `Analysis` repeatedly.
- Run compare and inspect workspace loading mask boundaries.
- Select `different`, `left-only`, `right-only`, and `equal` entries and confirm:
  - header baseline stability
  - shell alignment
  - diff column alignment
- Open `Provider Settings`, switch `Mock` and `OpenAI-compatible` multiple times, then trigger validation error text.
- Re-check both narrow and wide window widths because several current issues are layout-distribution problems.

## Out of scope for the later fix pass

- No IA change.
- No new theme system.
- No redesign of compare workflow.
- No provider profile management.
- No change to AI business logic or settings persistence format.
