# Folder Compare Architecture (Stable Baseline before Phase 18)

## Purpose

- This document summarizes the current stable product, UI, and platform baseline after:
  - `Phase 16A`
  - `Phase 16A fix-1`
  - `Phase 16B`
  - `Phase 16C`
  - `Phase 16C fix-1`
  - `Phase 17A`
  - `Phase 17A fix-1`
  - `Phase 17B`
  - `Phase 17B fix-1`
  - `Phase 17C`
  - `Phase 17D`
- It is a baseline summary for future work, not a phase diary or roadmap checklist.
- Earlier `Phase 15.x` closeouts and the standalone workspace `edition = "2024"` milestone remain accepted background and are not reopened here.

## Stable Delivery Baseline

- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`
- Release version ownership stays in the workspace manifest, and packaging derives bundle / DMG / ZIP version from that source.
- `15.2E` is shipped on this baseline.
- Accepted inherited contracts from earlier closeouts remain in force:
  - native editable-input context menus
  - `Analysis success` native text-surface right-click
  - shared readable-content typography token
  - shared editable-input typography token
  - event-driven UI synchronization
  - persistent `VecModel` row projection

## Crate Responsibilities

- `fc-core`: deterministic directory compare, text diff domain model, public API, and error contracts.
- `fc-ai`: optional AI interpretation layer for diff outputs behind a provider abstraction.
- `fc-ui-slint`: desktop entry, app-state orchestration, window/platform integration, and UI presentation.

## Hard Architectural Boundaries

- `fc-core` stays deterministic and isolated from UI, runtime, and provider concerns.
- `fc-ai` stays optional. Compare output must remain usable when AI is disabled or unavailable.
- `fc-ui-slint` handles orchestration and presentation, not core business rules.
- Workspace structure stays `Tabs -> Header -> Content`.
- Connected workspace tabs plus the attached workbench surface remain part of the accepted visual contract.
- `Compare Status` stays summary-first.

## Stable Product Structure

### Top-Level Shell

- Top-level structure stays `Window -> Top Bar -> Main Split`.
- The main split stays `Sidebar + Workspace`.
- The right side remains one continuous workbench surface rather than multiple competing nested cards.

### Sidebar: Four Stable Blocks

- `Compare Inputs`
  - Collects left/right roots and triggers compare.
  - Owns input, browse, and the primary compare action only.
  - Does not carry compare diagnostics beyond lightweight action gating.
- `Compare Status`
  - Owns compare outcome summary, compact metrics, warnings/errors, and copy-ready detail exposure.
  - Remains an inline summary-first block with an optional detail tray.
  - Does not own row selection or file-level rendering.
- `Filter / Scope`
  - Owns path/name search and status scope (`All / Diff / Equal / Left / Right`).
  - Narrows the visible navigator set only.
  - Does not mutate source compare data or change compare-summary source counts.
- `Results / Navigator`
  - Owns the flat visible collection, row scanability, and selection dispatch into the workspace.
  - Remains file-view-first rather than tree/group navigation.

### Workspace: Stable Structure

- Workspace stays `Tabs -> Header -> Helper Strip -> Body`.
- Tabs remain `Diff` and `Analysis`.
- Only one main mode renders at a time, but both modes stay inside the same attached workbench shell.
- The outer workspace wrapper is visually transparent, and the inner workbench host/panel is the one primary perceived surface on the right.

### Diff / Analysis Shared File View Shell

- `Diff` and `Analysis` share the same file-level shell rhythm in non-ready states:
  - title row
  - metadata / badge row
  - helper strip
  - shell/content body
- `DiffStateShell` remains the shared embedded state surface for non-ready rendering.
- Embedded shell mode stays low-noise and subordinate to the workbench instead of reading as a second heavy card.
- `Diff` keeps the ready-state diff table, and `Analysis` keeps the structured review-conclusion surface, but both reuse the same surrounding shell contract.
- `Diff` ready-state ergonomics stay stable:
  - explicit `ScrollView` viewport for horizontal scrolling
  - mirrored header `viewport-x`
  - content-end scrollbar-safe spacer
  - one shared column-geometry contract between header and body separators

## Stable UI / Interaction Contract

### Compare, Status, Filter, Results Responsibilities

- `Compare Inputs` keeps one dedicated full-width primary action lane.
- The inline `Ready to compare / Running compare / Select left and right folders` text pattern is intentionally gone.
- Compare action explanation now belongs to:
  - `Compare Status` for summary-first run state
  - one lightweight button tooltip only when Compare is disabled or already running
- `Filter / Scope` owns visible-set narrowing.
- `Results / Navigator` owns row scanability, selection, and stale-selection transitions.
- Workspace owns file-level view and analysis only.

### Results Row Information Hierarchy

- Primary information: status pill + filename / leaf path segment.
- Secondary information: concise capability-first summary such as `Text diff`, `Text-only preview`, `No text diff`, or `No text preview`.
- Weak information: parent-path context for disambiguation only.
- The list stays flat. No tree, grouping, alternate navigator mode, or detail-heavy row explanation layer was added.
- Search highlighting remains lightweight and row-local on filename / parent-path labels only.

### Selection and Availability Semantics

- `no-selection`
  - Nothing is actively selected.
- `stale-selection`
  - The previously focused relative path is no longer part of the current visible `Results / Navigator` set.
  - The visible row highlight clears.
  - The file view keeps explicit stale context.
  - The UI does not auto-jump to another row.
- `unavailable`
  - The selected row is valid, but the current viewer or analysis mode cannot produce supported output for it.
- Compare rerun restoration stays conservative:
  - restore by the same relative path only when it still exists and is still visible under current filters/preferences
  - otherwise remain stale
- Search, status-scope, and hidden-files preference changes reuse the same stale-selection contract when they remove the active row from the visible set.

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
- Results row tooltip completes full filename + full parent path only when visible row text is truncated.
- Input tooltip completes the full input value only when the field is not actively being edited and the visible value is truncated.

### Settings Boundary

- `App Bar -> Settings` is the single global settings entry.
- Settings currently owns two sections only:
  - `Provider`
  - `Behavior`
- `Provider` owns AI provider configuration and retains the dedicated `API Key` secret-field contract.
- `Behavior` currently owns one persisted presentation preference: `Hidden files`.
- Settings is not:
  - a second compare-status surface
  - a row-detail view
  - a general workflow controller

### Hidden Files Boundary

- `Hidden files` is a UI / presentation preference only.
- It changes the default visible `Results / Navigator` set and its summary copy immediately.
- It does not change:
  - compare request building
  - compare-summary source counts
  - `fc-core` API contracts
  - search semantics
- The collection summary explicitly reports when entries are hidden by Settings so this preference does not blur into Search or Status filtering.

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
- The immersive strip is full-bleed across the window width while the main content area keeps the existing `10px` content inset rhythm.

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

- `SelectableDiffText` and `SelectableSectionText` keep `UiTypography.selectable_content_font_family`.
- Ordinary inputs and `ApiKeyLineEdit` keep `UiTypography.editable_input_font_family`.
- Loading feedback and toast feedback remain UI-local rather than global-controller based.
- Background compare / diff / analysis work remains off the UI thread, and UI updates return through event-loop upgrade instead of broad polling.
- `Results / Navigator` and `Diff` row models stay on persistent `VecModel` instances.

### Persistence

- Settings persistence stays in `settings.rs`.
- `settings.toml` is the single active baseline.
- `provider_settings.toml` is legacy migration input only when `settings.toml` is absent.

## Deferred / Explicitly Not Doing

- Secure secret storage integration:
  - trigger: before a remote provider becomes the production-default path.
- Provider profile management:
  - trigger: when rapid provider switching becomes a real daily workflow.
- Response caching and token / cost tracking:
  - trigger: when remote analysis usage becomes an operational concern.
- Multi-provider orchestration:
  - trigger: when fallback or provider routing becomes a real reliability requirement.
- Compare-level hidden-entry policy:
  - trigger: only if compare requests, shared statistics, export, cache behavior, or future tree/navigation work all need the same policy source.
  - boundary: do not mix this with the current `Settings -> Behavior` presentation preference.
- Results match-span highlight:
  - trigger: only if the current filename/path label-level highlight proves insufficient.
  - boundary: match positions must come from a lower layer; Slint stays render-only.
- Optional diff row context menu beyond the current copy hotspots:
  - trigger: only if mouse-driven row copy becomes a demonstrated gap.
- Search clear-affordance convergence:
  - trigger: only if the native desktop style exposes a stable clear affordance or the product deliberately changes widget style.
- Analysis shell-state selectable text:
  - trigger: only if revisited as an isolated pass that does not destabilize the accepted success-body scrolling/menu contract.
- Sticky left-side line numbers:
  - trigger: only if the current diff viewer stops being sufficient for review ergonomics.
- Global loading orchestration:
  - trigger: only when broader multi-surface workflows require a shared loading model.
- Global toast orchestration:
  - trigger: only when broader save/export/report flows require a notification-center model.
- macOS title bar fine-tuning:
  - trigger: only if real smoke testing proves the fixed leading inset or current drag strip insufficient.
  - boundary: keep the follow-up inside `fc-ui-slint` first; do not jump to raw AppKit or traffic-light repositioning without a separate design pass.
- Tree explorer / dual-mode workspace:
  - trigger: only if flat-list file-view-first navigation becomes a demonstrated bottleneck.

## Phase 18 Entry Conditions

### Stable Baseline Phase 18 Can Assume

- Sidebar IA is stable as four cards with clear responsibility boundaries.
- Workspace structure is stable as one attached `Diff / Analysis` file-view shell rather than multiple competing surfaces.
- Results row hierarchy and selection-state semantics are stable.
- Tooltip, Settings, and Hidden-files boundaries are stable.
- The platform window-layer contract is stable:
  - macOS immersive strip
  - non-mac legacy top bar
  - platform branching contained in `window_chrome`
- Supporting copy/menu, typography, runtime-sync, and settings-persistence contracts are stable.

### Therefore Phase 18 Can Focus On

- File-view and analysis work inside the accepted `Diff / Analysis` shell.
- Review-efficiency or content-quality improvements that reuse the current Sidebar / Workspace / window structure instead of redesigning it.
- Narrow feature work that depends on the current selection, state-shell, and presentation-preference contracts as fixed inputs.

### Phase 18 Should Not Mix In

- New IA, tree/group navigation, or dual-mode workspace design.
- Window-system rework, `no-frame`, raw AppKit, traffic-light repositioning, or non-macOS chrome changes.
- Full settings framework work, compare-level hidden-entry policy, or other `fc-core` contract widening tied to visibility policy.
- Global tooltip/loading/toast/controller systems.
- Reopening shipped `Phase 15.x`, edition-2024, or the accepted `Phase 16A` to `Phase 17D` baseline contracts.

## Documentation Update Contract

- Update this file whenever the stable baseline, boundary, deferred list, or Phase 18 entry conditions materially change.
- Keep it as a baseline summary, not a phase-by-phase diary.
- Keep terminology aligned with `docs/thread-context.md` and `docs/upgrade-plan-rust-1.94-slint-1.15.md`.
