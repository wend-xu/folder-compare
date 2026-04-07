# Folder Compare Architecture (Stable Baseline + Phase 18 Closeout)

## Purpose

- This document records two layers at once:
  - the current real stable baseline closed through `Phase 17D`
  - the currently implemented `Phase 18` closeout baseline inside that shell
- It is a baseline and boundary document, not a phase diary and not an implementation checklist.
- The current default next-stage entry is `Phase 19` draft discussion, but `Phase 19` is not implemented in this document's baseline.
- Older wording such as "flat list only" or "do not mix tree/group navigation" remains useful as historical description of the pre-`Phase 18` stable baseline, but it is no longer the forward-looking boundary after the `2026-03-22` alignment.

## Stable Delivery Baseline

- Stable evolution baseline: post-`0.2.18`
- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`
- `15.2E` is already shipped on this baseline.
- Accepted inherited contracts from earlier closeouts remain in force:
  - native editable-input context menus
  - `Analysis success` native text-surface right-click
  - default generic-family text path with the current centralized macOS bootstrap shim
  - event-driven UI synchronization
  - persistent `VecModel` row projection

## Completed Stable Baseline Through Phase 17D

- `Phase 16`
  - Sidebar information architecture closeout
  - `Results / Navigator` filtering and scanability closeout
  - `Compare Status` summary-first static block closeout
  - results-row hierarchy, stale-selection, unavailable, and no-selection semantics closeout
  - capability-first secondary text and input-font TOFU fixes
- `Phase 17A`
  - shared tooltip infrastructure
  - tooltip completion for results rows, compare inputs, and search
  - long-text truncation and tooltip stability closeout
- `Phase 17B`
  - `Provider Settings` promoted to `Settings`
  - first `Provider / Behavior` split
  - `Hidden files` introduced as the first UI/presentation preference
  - `settings.toml` established as the only active persistence baseline
  - legacy provider settings reduced to migration input only
  - settings container size stabilized
  - `Hidden files` intentionally not pushed down into `fc-core`
- `Phase 17C`
  - historical UI bug A/B/C/D closeout
  - compare-input action finishing pass
  - Compare button tooltip
  - full-width Compare action lane
  - workspace geometry and embedded `DiffStateShell` closeout
- `Phase 17D`
  - macOS immersive title bar landed
  - macOS uses immersive strip
  - Windows/Linux keep legacy top bar
  - no `no-frame`
  - drag moved to explicit blank area inside immersive strip only
  - regression fixes closed out

## Current Implemented Phase 18 Closeout Baseline

- `Results / Navigator` now supports dual-view operation inside the existing sidebar block:
  - non-search runtime tree view
  - flat view
- `Phase 18A + 18B + 18C` closeout is complete on the current code baseline, including the later search-state `Locate and Open` cleanup fix.
- Non-search state defaults to the persisted `Settings -> Behavior -> Default view` preference.
- Search text still follows the stable `path / name only` contract and forces flat results mode.
- Tree data/state is Rust-owned inside `fc-ui-slint` presenter/state:
  - canonical merged tree built from existing compare entries over the left/right path union
  - flattened visible tree rows projected from Rust to Slint
  - directory expansion/collapse state owned outside Slint
- Slint now uses one independent tree renderer component for tree rows; it does not own recursive tree state.
- Tree rows now use a drawn disclosure chevron plus trailing lightweight status text rather than flat-row pill/card inheritance.
- Tree row label color now carries a restrained status tone so files/directories stay scannable without reintroducing heavy card/pill language.
- Directory-node click in tree mode only expands/collapses; directory nodes do not enter right-side file-view selection.
- File-leaf click in tree mode reuses the existing `selected_row -> load diff -> load analysis` path.
- Hidden-files preference remains a UI/presentation boundary and now also applies to tree-mode projection.
- Status filter now prunes tree visibility with necessary ancestors retained, and directory `display_status` is recomputed from the filtered visible subtree.
- `Settings -> Behavior` now persists:
  - `Hidden files`
  - default non-search result view (`Tree` / `Flat`)
- Selection remains conservative:
  - tree/flat switching preserves the currently open file only when it remains a member of the target visible set
  - switching into tree reveals the selected file by expanding its ancestor chain when that file is still valid
  - when one file survives a tree/flat mode switch, the active navigator view now ensure-scrolls that file back into the visible viewport
  - collapsing an ancestor directory does not force `stale-selection` when file membership has not changed
- Flat results now support `Locate and Open` from both search results and explicit flat browsing:
  - clear search-result mode when present
  - switch back to tree mode
  - expand the ancestor chain
  - ensure the target file leaf is visible inside the tree viewport
  - select the target file leaf
  - reopen the right-side file view through the existing diff/load pipeline
- Compare reruns now preserve expansion overrides conservatively:
  - keep still-valid expanded/collapsed directory paths
  - prune invalid paths
- Workspace structure is still the existing attached `Diff / Analysis` file-view shell; the later `Compare View / File View` dual-mode workspace redesign is not implemented.
- The following items are still intentionally deferred beyond this baseline:
  - animated locate feedback beyond the current ensure-visible baseline
  - directory summary/counts/secondary text
  - directory selection / directory detail presentation in the workspace
  - tree-internal search / content search / richer match-span semantics
  - `Compare View / File View` workspace redesign
  - compare-core contract widening

## Crate Responsibilities

- `fc-core`: deterministic directory compare, text diff domain model, public API, and error contracts.
- `fc-ai`: optional AI interpretation layer for diff outputs behind a provider abstraction.
- `fc-ui-slint`: desktop entry, app-state orchestration, window/platform integration, presenter logic, and UI presentation.

## Hard Architectural Boundaries

- `fc-core` stays deterministic and isolated from UI, runtime, and provider concerns.
- `fc-ai` stays optional. Compare output must remain usable when AI is disabled or unavailable.
- `fc-ui-slint` handles orchestration and presentation, not core compare semantics.
- Sidebar IA stays fixed as four blocks.
- Workspace stays one continuous attached workbench surface rather than multiple competing shells.
- `Compare Status` stays summary-first.

## Current Stable Product / UI / Platform Baseline

### Top-Level Shell

- Top-level structure stays `Window -> Top Bar -> Main Split`.
- The main split stays `Sidebar + Workspace`.
- The right side remains one continuous workbench surface rather than nested competing cards.

### Sidebar: Four Stable Blocks

- `Compare Inputs`
  - collects left/right roots and triggers compare
  - owns input, browse, and the primary compare action only
  - does not carry compare diagnostics beyond lightweight action gating
- `Compare Status`
  - owns compare outcome summary, compact metrics, warnings/errors, and copy-ready detail exposure
  - remains an inline summary-first block with optional detail tray
  - does not own row selection or file-level rendering
- `Filter / Scope`
  - owns `path / name` search and status scope (`All / Diff / Equal / Left / Right`)
  - narrows the visible results set only
  - does not mutate source compare data or compare-summary source counts
- `Results / Navigator`
  - owns result browsing, row scanability, and selection dispatch into the workspace
  - current code baseline is dual-view:
    - tree mode for non-search navigation
    - flat mode for search results and explicit flat browsing
  - this remains an evolution inside the same block rather than a new IA

### Workspace: Stable Structure

- Workspace stays `Tabs -> Header -> Content`.
- Tabs remain `Diff` and `Analysis`.
- Only one main mode renders at a time, but both modes stay inside the same attached workbench shell.
- The outer workspace wrapper stays visually subordinate to the inner workbench host/panel.

### Diff / Analysis Shared File View Shell

- `Diff` and `Analysis` share the same file-level shell rhythm in non-ready states:
  - title row
  - metadata/badge row
  - helper strip where applicable
  - shell/content body
- `DiffStateShell` remains the shared embedded state surface for non-ready rendering.
- Embedded shell mode stays low-noise and subordinate to the workbench instead of reading as a second heavy card.
- `Diff` keeps the ready-state diff table, and `Analysis` keeps the structured review-conclusion surface, but both reuse the same surrounding shell contract.

## Stable UI / Interaction Contract

### Compare, Status, Filter, Results Responsibilities

- `Compare Inputs` keeps one dedicated full-width primary action lane.
- Inline `Ready to compare / Running compare / Select left and right folders` text next to the button remains intentionally removed.
- Compare action explanation belongs to:
  - `Compare Status` for summary-first run state
  - one lightweight button tooltip only when Compare is disabled or already running
- `Filter / Scope` owns visible-set narrowing.
- `Results / Navigator` owns row scanability, selection, and stale-selection transitions.
- Workspace owns file-level view and analysis only.

### Results Row and Search Contract

- Flat results row hierarchy remains:
  - primary: status pill + filename / leaf path segment
  - secondary: capability-first summary such as `Text diff`, `Text-only preview`, `No text diff`, or `No text preview`
  - weak: parent-path disambiguation only
- Tree-mode first-pass row expression stays intentionally smaller:
  - node label first
  - trailing lightweight status text / tone
  - drawn disclosure chevron for directories where applicable
  - restrained list-style selection instead of flat-card inheritance
- Search contract remains `path / name only`.
- Search highlighting remains lightweight and row-local on filename / parent-path labels only.
- Tooltip for results rows remains truncated-text completion only, not a second explanation system.

### Selection and Availability Semantics

- `no-selection`
  - nothing is actively selected
- `stale-selection`
  - the previously focused relative path is no longer part of the current visible results set
  - visible row highlight clears
  - file view keeps explicit stale context
  - UI does not auto-jump to another row
- `unavailable`
  - the selected row is valid, but the current viewer or analysis mode cannot produce supported output for it
- Compare rerun restoration stays conservative:
  - restore by the same relative path only when it still exists and is still visible under current filters/preferences
  - otherwise remain stale
- Search, status scope, and hidden-files preference changes reuse the same stale-selection contract when they remove the active row from the visible set.
- Tree-mode directory collapse does not by itself make the current file selection stale; stale-selection only follows actual visible-set membership change.
- Tree/flat runtime switching reuses the same conservative contract, and surviving file selections are ensure-scrolled back into the active viewport.

### Diff and Analysis Shell Semantics

- `Diff` state machine stays:
  - `no-selection | stale-selection -> loading -> unavailable | error -> preview-ready | detailed-ready`
- Single-side preview remains first-class for `left-only`, `right-only`, and `equal` rows when preview is the right viewer mode.
- `Analysis` state machine stays:
  - `no-selection | stale-selection -> waiting | ready | unavailable -> loading -> error | success`
- `Analysis success` remains a structured review-conclusion surface with:
  - `Summary`
  - `Risk Level`
  - `Core Judgment`
  - `Key Points`
  - `Review Suggestions`
  - `Notes`

### Tooltip Boundary

- Tooltip is one shared window-local overlay.
- Its stable role is:
  - truncated-text completion for results rows and inputs
  - restrained state hint for the disabled/running Compare action
- Tooltip is not:
  - a second explanation surface
  - a detail-panel replacement
  - a platform-native tooltip framework

### Settings Boundary

- `App Bar -> Settings` is the single global settings entry.
- Settings currently owns two sections only:
  - `Provider`
  - `Behavior`
- `Provider` owns AI provider configuration and retains the dedicated `API Key` secret-field contract.
- `Behavior` currently owns two persisted presentation preferences:
  - `Hidden files`
  - default non-search result view
- Settings is not:
  - a second compare-status surface
  - a row-detail view
  - a general workflow controller

### Hidden Files Boundary

- `Hidden files` is a UI/presentation preference only.
- It changes the default visible results set and its summary copy immediately.
- It does not change:
  - compare request building
  - compare-summary source counts
  - `fc-core` API contracts
  - search semantics

## Platform and Window Baseline

### Current Window-Layer Contract

- Platform branching stays inside `fc-ui-slint::window_chrome`.
- The root Slint window only receives read-only platform chrome properties:
  - `immersive_titlebar_enabled`
  - `titlebar_visual_height`
  - `titlebar_leading_inset`
- `app.rs` keeps presentation declarative and does not own backend-selection policy directly.
- The window layer remains one standard framed window.
- No `no-frame` mode, custom resize hit testing, raw AppKit `NSWindow` ownership, or `objc2` integration was adopted.

### macOS Immersive Title Bar

- macOS alone installs the explicit platform windowing path before `MainWindow::new()`.
- That path uses Slint's winit hook to apply:
  - transparent title bar
  - full-size content view
  - hidden native title text
- The macOS top bar is one immersive strip that visually merges with the window top edge and reserves a conservative leading safety inset for traffic lights.
- Dragging is explicit and local to the immersive strip via `drag_window()`.
- Whole-window background dragging is intentionally not reopened.

### non-macOS Legacy Top Bar

- Windows and Linux keep the existing `SectionCard` top bar.
- They do not opt into forced `winit` backend selection, title bar API customization, or fallback window-behavior logic.
- Zero-behavior-change for non-macOS remains part of the accepted contract.

## Accepted Supporting Contracts

### Menus, Text, and Copy Behavior

- `Compare Status` stays summary-first and exposes inline `Show details / Hide details` plus shared `Copy Summary` / `Copy Detail`.
- `Analysis success` keeps lightweight inline `Copy` actions plus `Copy All`.
- `Analysis success` body text stays on Slint native text surface right-click.
- Section chrome stays on the shared non-input `Copy` / `Copy Summary` menu.
- `Risk Level` stays explicit `Copy` button-only.
- Ordinary editable inputs keep native editable-input context menus.
- `Settings -> Provider -> API Key` keeps one dedicated `ApiKeyLineEdit` contract:
  - hidden: `Paste` only
  - visible: `Select All`, `Copy`, `Paste`, `Cut`

### Typography, Feedback, and Runtime Sync

- Window-local text surfaces now rely on Slint's default generic family path, with the existing macOS bootstrap wiring system fonts back into that route for Latin/CJK/full-width text.
- On macOS, that bootstrap remains a temporary compatibility shim centralized in `crates/fc-ui-slint/src/macos_font_bootstrap.rs`:
  - it currently compensates for two confirmed dependency-stack issues in the current `Slint 1.15.1` baseline:
    - an older mixed-text fallback/selection problem that was already user-visible on `macOS 13.5`
    - the later `fontique 0.7.0` macOS font-discovery problem that became visible after upgrading to `macOS 15.7`
  - the discovery portion is already known to be fixed in `fontique 0.8.0`, but actual removal timing still depends on when the Slint version used by this project absorbs that fix and can be revalidated on real macOS rendering samples
  - this shim should be removed once the upstream stack behaves correctly without it; it is not the application's long-term font-policy layer
- `SelectableDiffText` and `SelectableSectionText` no longer add a runtime font-family override on top of that shared baseline.
- Ordinary inputs and `ApiKeyLineEdit` likewise stay on the default generic-family path without an extra runtime typography layer.
- Loading feedback and toast feedback remain UI-local rather than global-controller based.
- Background compare/diff/analysis work remains off the UI thread, and UI updates return through event-loop upgrade instead of broad polling.
- `Results / Navigator` and `Diff` row models stay on persistent `VecModel` instances.

### Persistence

- Settings persistence stays in `settings.rs`.
- `settings.toml` is the single active baseline.
- `provider_settings.toml` is legacy migration input only when `settings.toml` is absent.

## Historical Pre-Phase 18 Limitation Note

- The pre-`Phase 18` stable baseline correctly treated `Results / Navigator` as a flat-only result browser.
- Earlier wording such as:
  - "do not mix tree / group navigation"
  - "the list stays flat"
  - "Phase 18 should not mix in hierarchy"
  described the accepted pre-`18A` implementation boundary and should be read as historical baseline context.
- Those statements must not be reused as a current prohibition, because `Phase 18` has now been formally reopened for hierarchical result-view evolution inside the existing Sidebar/Workspace/window contracts.

## Closed Phase 18 Record Status

- The `Phase 18` sections below are kept as the architectural record of what was decided and implemented across `18A / 18B / 18C`.
- They are not the default execution entry anymore.
- New work should default to `Phase 19` draft discussion unless a concrete regression requires a narrow `18C fix-*` follow-up.
- Nothing below should be read as evidence that `Compare View / File View` workspace work, tree search, directory detail panes, or compare-core widening already exist.

## Why Phase 18 Does Not Rewrite Compare Core Semantics

- The `Phase 18` tree is a presenter/UI navigation upgrade, not a compare-core semantic rewrite.
- `fc-core` already provides deterministic compare entries over the union of left/right paths, including directory entries.
- `Phase 18` does not change:
  - compare request construction
  - hidden-entry policy in `fc-core`
  - source compare-summary counts
  - base diff/analysis pipelines
- What changes in `Phase 18` lives in `fc-ui-slint` presentation/state:
  - merged tree construction from existing compare entries
  - visible-row projection for tree rendering
  - expanded/collapsed state
  - mode switching between tree and flat views
  - filtered display-status aggregation for directories

## Phase 18 Definition

- `Phase 18` introduces a hierarchical results view built from the union of left/right paths, using an independent tree component to carry directory structure, expansion, and status expression, while retaining flat results for search and concentrated scanning and establishing a reusable data expression for later Compare View work.

## Phase 18 Product Positioning

### Dual View Coexistence

- `Results / Navigator` enters dual-view operation:
  - tree view
  - flat view
- Tree view is the primary non-search navigation surface.
- Flat view remains valuable for search results and concentrated scanning.
- This is not a new IA and not a second workspace; it is an evolution inside the existing `Results / Navigator` block.

### Search Flat Fallback

- Search results do not map directly into the tree.
- When search is non-empty, `Results / Navigator` must enter flat results mode.
- Clearing search returns to the current non-search runtime mode.
- Search contract remains `path / name only`; `Phase 18A` does not introduce tree search, content search, or deep tree highlight rules.

### Independent Tree Component Boundary

- The tree component must be independent; tree behavior must not be stacked directly into `app.rs`.
- Rust side owns:
  - merged tree construction
  - filtered visible-tree-row flattening
  - selection/stale-selection decisions
  - expanded-path state
  - runtime mode decisions
- Slint side owns render-only tree-row presentation plus row-level event dispatch.
- Slint must not become the owner of recursive tree data, directory aggregation rules, or long-lived expansion/selection state.

## Phase 18A / 18B / 18C Split

### `18A`: Landed Correctness Baseline

- merged tree builder from left/right union entries
- independent tree component
- flattened visible tree-row projection from Rust to Slint
- hidden-files compatibility
- status-filter pruning on tree
- file-leaf select/open reuse of the existing file-view pipeline
- non-search runtime tree/flat toggle in `Results / Navigator`
- default expansion rule:
  - synthetic root implicitly expanded
  - depth-1 directories expanded by default
- conservative selection retention/stale handling for:
  - filter/search/hidden-files changes
  - tree/flat runtime switching
  - ancestor collapse without false stale

### `18B`: Mode Linkage and Locate Flow

- persisted default result-view setting in `Settings`
- `Locate and Open` from flat search results back into tree mode
- ancestor-chain reveal for tree linkage and locate flow
- expanded-path pruning/restore refinement across compare reruns

### `18C`: Stabilization and Polish

- visible-region continuity for tree/flat switching
- locate visible-region closure for tree leaf reveal
- `Locate and Open` parity for explicit flat browsing
- lightweight tree visual polish
- presenter/state contract tests for scroll/locate continuity

## Phase 18A Scope Boundary

### In Scope

- introduce tree mode alongside flat mode inside `Results / Navigator`
- keep tree data/state in Rust presenter/state rather than Slint local state
- render flattened visible tree rows in Slint
- reuse existing file-leaf open/diff/load pipeline
- apply status filter as tree pruning with necessary ancestors retained
- preserve current stale-selection semantics

### Out of Scope

- `Settings` persistence for default result view
- `Locate and Open`
- auto reveal / auto scroll / animated locate feedback
- directory selection entering the right-side file view
- descendant counts, subtree summary text, complex directory statistics
- content search, tree-internal search highlighting, or match-span semantics
- dual-tree layout, Compare View / File View workspace redesign, or compare-core widening

## Phase 18A Confirmed Decisions

### Decision 1: non-search defaults to tree mode; search forces flat mode

- In non-search state, `Results / Navigator` defaults to the persisted `Settings -> Behavior -> Default view`.
- When search text is non-empty, search results must not be projected into the tree and the UI must switch to flat results mode.
- Clearing search returns to the current non-search runtime mode.
- The persisted default result view changes only the non-search baseline; it must not override search fallback.

### Decision 2: directory nodes do not enter file-view selection in v1

- Directory-node click only expands/collapses.
- Directory nodes do not become the right-side file-view selection target.
- Only file-leaf selection reuses the existing `selected_row -> open file -> load diff/analysis` path.

### Decision 3: directory `display_status` must be recomputed from the filtered visible subtree

- Canonical/base status may remain available for unfiltered semantics.
- Displayed directory status must serve the currently filtered tree.
- Unfiltered directory status must not be shown unchanged after status-filter pruning.

### Decision 4: tree/flat runtime switching keeps the conservative selection contract

- If the currently open file is visible in the target mode, map highlight and keep the file open.
- If the target mode is tree, reveal the file by expanding its ancestor chain rather than forcing a stale transition just because the branch was collapsed.
- If it is not visible in the target mode, reuse existing `stale-selection` semantics.
- If it survives the mode change, ensure-scroll it back into the active viewport; this is scoped to mode linkage and locate flow rather than a general animation system.

### Decision 6: `Locate and Open` is a flat-results workflow, not a general tree action

- `Locate and Open` starts from flat results only, whether the flat surface comes from search fallback or explicit flat browsing.
- It clears search when needed, switches to tree mode, expands ancestors, ensure-scrolls the target leaf into view, and then reuses the existing file-leaf open pipeline.
- It does not introduce directory selection, tree-internal search, or a second file-view mode.

### Decision 7: compare rerun preserves expansion state conservatively

- Expanded/collapsed overrides are restored only for directory paths that still map to expandable nodes in the new compare tree.
- Invalid or default-equivalent overrides are pruned.
- No additional "smart" expansion heuristics are introduced in `18B`.

### Decision 5: directory nodes keep minimal information expression in v1

- Directory nodes keep only the minimum first-pass expression:
  - node name
  - expand/collapse affordance
  - restrained status tone / status label
- `18A` does not add secondary text, descendant counts, complex summaries, or content-search highlight to directory rows.

## Deferred / Explicitly Not Doing Yet

- Secure secret storage integration:
  - trigger: before a remote provider becomes the production-default path
- Provider profile management:
  - trigger: when rapid provider switching becomes a real daily workflow
- Response caching and token/cost tracking:
  - trigger: when remote analysis usage becomes an operational concern
- Multi-provider orchestration:
  - trigger: when fallback or provider routing becomes a real reliability requirement
- Compare-level hidden-entry policy:
  - trigger: only if compare requests, shared statistics, export, cache behavior, or future navigation work all need the same policy source
  - boundary: do not mix this with the current `Settings -> Behavior` presentation preference
- Match-span highlight:
  - trigger: only if current filename/path label-level highlight proves insufficient
- Tree locate animation / extra emphasis beyond ensure-visible:
  - trigger: only if the current selection highlight plus ensure-visible scroll is insufficient for real workflows
- Tree-internal search / content search:
  - trigger: only if later work intentionally expands beyond the current `path / name only` search contract
- Directory selection / directory detail surface:
  - trigger: only if later workspace work explicitly needs directories to enter a right-side detail mode
- Window-system rework:
  - trigger: only if the current framed-window contract becomes a demonstrated blocker
- Compare-core widening:
  - trigger: only if later workspace/data-model work proves the current compare entry contract insufficient
- Compare View / File View workspace split:
  - trigger: only if later work proves the current attached file-view shell insufficient

## Next-Stage Activation

- Default next entry is `Phase 19` draft discussion.
- `Phase 19` still refers to future `Compare View / File View` workspace work; it has not entered implementation in the current baseline.
- Only return to `18C fix-*` as the main thread when a concrete regression is identified in the shipped `Phase 18` baseline.

## Documentation Update Contract

- Update this file whenever the stable baseline, the `Phase 18` closeout boundary, the default next-stage activation, or the active deferred list materially changes.
- Keep stable-baseline facts separate from future-phase scope so new threads do not confuse shipped behavior with planned behavior.
- Keep terminology aligned with `docs/thread-context.md` and `README.md`.
