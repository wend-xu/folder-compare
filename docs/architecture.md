# Folder Compare Architecture (Current Baseline after Phase 16B)

## Current status

- `phase15 summary` is complete as a documentation closeout.
- The following work is completed and closed:
  - `Phase 16A`
  - `Phase 16A fix-1`
  - `Phase 16B`
  - `Phase 15.3A`
  - `Phase 15.3B`
  - `Phase 15.4`
  - `Phase 15.5`
  - `Phase 15.5 fix-1`
  - `Phase 15.5 fix-2`
  - `Phase 15.5 fix-3`
  - `Phase 15.6`
  - `Phase 15.7`
  - `Phase 15.8`
  - `Phase 15.8 fix-1`
  - the independent workspace `edition = "2024"` milestone
- Current shipped baseline:
  - workspace `version = "0.2.18"`
  - workspace `edition = "2024"`
  - `rust-toolchain = 1.94.0`
  - workspace `rust-version = 1.94`
  - `slint = 1.15.1`
  - `slint-build = 1.15.1`
  - release version ownership lives in the workspace manifest, and packaging derives bundle / DMG / ZIP version from that source
  - `15.2E` is shipped on this baseline
- Current working baseline on top of that shipped base:
  - `Phase 16A` is complete
  - `Phase 16A fix-1` is complete
  - `Phase 16B` is complete
  - Sidebar IA remains unchanged, but `Compare Status`, `Filter / Scope`, and `Results / Navigator` now use the tightened `Phase 16A + 16A fix-1 + 16B` presentation contract
- Why `Phase 16` still remains the active train:
  - the dependency-upgrade train and the edition milestone are already finished;
  - `Phase 16A`, `16A fix-1`, and `16B` closed the Sidebar expression and row-scanability pass without reopening old closeouts;
  - the next thread should therefore continue the remaining `Phase 16` work instead of reopening `15.3A` to `15.8 fix-1` or edition-2024 tasks.

## Phase 15 summary

- `Phase 15.3A` aligned version ownership around the workspace manifest and packaging script.
- `Phase 15.3B` locked Rust to `1.94.0` and raised workspace `rust-version` to `1.94`.
- `Phase 15.4` moved the workspace to `slint 1.15.1` / `slint-build 1.15.1` without widening product scope.
- `Phase 15.5` shipped editable-input context-menu coverage on native Slint surfaces for `Compare Inputs`, `Filter / Scope -> Search`, and `Provider Settings`, while keeping `Provider Settings -> API Key` on one dedicated `ApiKeyLineEdit` with a narrower secret contract.
- `Phase 15.5 fix-1` repaired mixed Latin/CJK glyph fallback for read-only selectable content.
- `Phase 15.5 fix-2` moved that glyph-fallback policy onto the shared `UiTypography.selectable_content_font_family` token.
- `Phase 15.5 fix-3` moved `Diff` detail horizontal scrolling onto an explicit `ScrollView` viewport with a content-end scrollbar-safe spacer.
- `Phase 15.6` shipped event-driven sync, phase-driven one-shot loading-mask timeout copy, and persistent `VecModel` projection, while intentionally keeping the inline `slint::slint!` surface.
- `Phase 15.7` shipped non-input context-menu visual polish as a style-only pass.
- `Phase 15.8` shipped `Analysis success` native text-surface right-click for `Summary`, `Core Judgment`, `Key Points`, `Review Suggestions`, and `Notes`.
- `Phase 15.8 fix-1` restored explicit section-header left alignment without changing the `15.8` menu boundary.
- The independent workspace `edition = "2024"` milestone used `cargo fix --edition --workspace` as the starting point, retained only direct compatibility edits, and bumped the workspace version to `0.2.18` without introducing new product behavior.

## Crate responsibilities

- `fc-core`: owns directory compare, text diff domain model, public API, and error contracts.
- `fc-ai`: owns AI-based interpretation for diff outputs through a provider abstraction.
- `fc-ui-slint`: owns desktop app entry, app state orchestration, and UI presentation.

## `fc-core` internal boundaries

- `api/`: external entry points such as `compare_dirs` and `diff_text_file`.
- `domain/`: pure domain types.
- `services/`: scan, alignment, large-directory policy, hashing fallback, text loading, and text diff construction.
- `infra/`: path normalization and thin filesystem helpers.

The dependency direction stays `api -> services -> domain/infra`. `domain` does not depend on `services`.

## Hard architectural boundaries

- `fc-core` stays deterministic and isolated from UI, runtime, and provider concerns.
- `fc-ai` stays optional. Core compare output must remain usable when AI is disabled or unavailable.
- `fc-ui-slint` handles orchestration and presentation, not core business rules.
- Workspace structure stays `Tabs -> Header -> Content`.
- Compare Status stays summary-first.

## `fc-ui-slint` current baseline

### IA and workspace shell

- IA remains:
  - `App Bar`
  - `Sidebar` (`Compare Inputs / Compare Status / Filter / Scope / Results / Navigator`)
  - `Workspace` (`Diff / Analysis` tabs + header + content)
- Workspace remains one continuous shell, and only one major mode participates in layout at a time.
- Connected workspace tabs and the attached workbench surface remain part of the accepted visual contract.

### Compare workflow

- Compare entry flow remains state-driven:
  - left/right path input
  - validation
  - compare trigger
  - summary-first status update
- `Compare Inputs` keeps the same interaction model, with only a light presentation pass around input/browse/compare grouping.
- `Compare Status` remains one static sidebar result block:
  - summary-first by default
  - inline `Show details / Hide details` tray inside the block
  - shared context-menu coverage on both the collapsed summary surface and the expanded detail tray
  - no modal or secondary report flow
- Filter flow remains state-driven:
  - path/name search text
  - segmented status scope (`All / Diff / Equal / Left / Right`)
  - segmented visual state stays in lockstep with filter state
  - the user-facing summary no longer repeats status scope as a second `scope` label
- `Results / Navigator` keeps the same row model and selection behavior, but the top summary now expresses the visible collection state (`Showing visible / total ...`) instead of raw filter field labels.
- `Results / Navigator` row items now follow a tighter flat-list scan contract:
  - primary information: status pill + filename / leaf path segment
  - secondary information: concise reason summary for `diff / equal / left / right`
  - weak information: parent-path context for disambiguation only
  - path/name filter hits use subtle row-local highlight on the matched filename or parent-path context
  - the list remains flat; no tree, grouping, or alternate navigation mode was introduced

### Diff and Analysis shell

- `Diff` keeps the compact File Context Header and the explicit shell-driven state machine:
  - `no-selection -> loading -> unavailable/error -> detailed-ready|preview-ready`
- Single-side preview remains first-class:
  - `left-only`, `right-only`, and `equal` all use the preview path when appropriate
  - preview columns stay side-aware (`left/right`)
- `Diff` detail keeps the current ergonomics baseline:
  - selectable line content
  - double-click line-number / hunk-marker copy fallback
  - explicit `ScrollView` viewport for horizontal scrolling
  - mirrored header `viewport-x`
  - content-end scrollbar-safe spacer
- `Analysis` keeps its explicit state machine:
  - `no-selection -> not-started -> loading -> error|success`
- `Analysis success` remains a structured review-conclusion panel:
  - `Summary`
  - `Risk Level`
  - `Core Judgment`
  - `Key Points`
  - `Review Suggestions`
  - `Notes`
- `Analysis success` keeps `helper strip -> action strip -> scrollable content` as one workbench surface.

### Copy and menu boundaries

- `Compare Status` reuses the shared window-local non-input context-menu core:
  - `Copy Summary`
  - `Copy Detail`
  - `Copy Detail` remains available even when the tray is collapsed
- `Analysis success` section cards keep lightweight inline `Copy` actions and one `Copy All` action.
- `Analysis success` body text now uses native text-surface right-click on the current selected text only:
  - `Summary`
  - `Core Judgment`
  - `Key Points`
  - `Review Suggestions`
  - `Notes`
- That body-text path stays on Slint native text surface (`ContextMenuArea` + `TextInput.copy()/select-all()`).
- Section header / chrome stays on the existing window-local non-input context-menu core with `Copy` / `Copy Summary`.
- `Risk Level` stays explicit `Copy` button-only.
- Section-header labels remain explicitly left-aligned inside the narrowed header label lane, and that lane must not block the inline `Copy` action.
- The shared window-local non-input context-menu core remains intentionally narrow:
  - safe surfaces only
  - built-in actions remain limited to `Copy` and `Copy Summary`, with per-surface label override when needed
  - no fake `Paste` / `Cut` / `Select All`
  - no menu state in `AppState`
  - no controller ownership pushed into `Presenter`

### Editable-input integration

- `Compare Inputs`, `Filter / Scope -> Search`, and `Provider Settings` ordinary inputs use the native editable-input context menu from `slint 1.15.1`.
- `Provider Settings -> API Key` keeps one dedicated `ApiKeyLineEdit`:
  - hidden state: `Paste` only
  - visible state: `Select All`, `Copy`, `Paste`, `Cut`
  - hidden state still blocks hidden-state `Cmd/Ctrl+A/C/X`
- `Search` keeps its explicit `Clear` button because the current native `cupertino` `LineEdit` style still does not expose a stable clear affordance.

### Typography, scrolling, and feedback

- `SelectableDiffText` and `SelectableSectionText` share `UiTypography.selectable_content_font_family`.
- That shared token remains the accepted fix for the Slint `1.15.1` mixed Latin/CJK glyph fallback regression.
- Copy feedback remains lightweight and UI-local:
  - top-center overlay `toast`
  - no new global toast controller
- Loading feedback remains UI-local:
  - loading-mask scope is still derived from existing busy flags
  - timeout copy stays on one-shot timers driven by busy-phase transitions
  - no new global loading controller

### Runtime synchronization

- Background compare / diff / analysis work still completes off the UI thread.
- Completion now pushes a fresh snapshot back to the UI thread through presenter notifier + `slint::Weak::upgrade_in_event_loop`.
- Broad repeated `50ms` polling is no longer the main UI sync path.
- `Results / Navigator` and `Diff` row models stay initialized as persistent `VecModel` instances and update through `set_vec()` only when relevant source payload changes.
- Cache-aware projection and menu close-on-selection / busy-start behavior remain intact.

### Provider settings and persistence

- Provider settings remain a global modal launched from `App Bar -> Provider Settings`.
- Persistence stays in `settings.rs`.
- Saved settings still use `provider_settings.toml` with the existing config-dir override.
- The edition-2024 milestone did not change the product contract here; it only retained direct compatibility fixes around settings load/save lock lifetime and test-only directory override handling.

## What intentionally did not change

- No `Phase 16` work was mixed into the phase15 closeout.
- No new IA, tree mode, or Compare View mode was introduced.
- No new theme system, global loading controller, or global notification controller was introduced.
- No overlay interception, private pointer plumbing, or custom caret/selection/editing logic was added for editable inputs or selectable text.
- The large inline `slint::slint!` surface was not externalized because the cleanup benefit is still below the migration cost on the current baseline.

## Deferred architecture decisions

- `P1` Secure secret storage integration:
  - trigger: before remote provider becomes production-default.
- `P1` Provider profile management:
  - trigger: when rapid provider switching becomes a daily workflow need.
- `P2` Response caching and token / cost tracking:
  - trigger: when remote analysis usage becomes an operational concern.
- `P2` Multi-provider orchestration:
  - trigger: when fallback or provider routing becomes a reliability requirement.
- `P2` Optional `SelectableDiffText` row-level context menu:
  - trigger: only if mouse-driven row copy becomes a demonstrated productivity gap beyond the current keyboard copy and double-click fallback.
- `P2` Search clear-affordance convergence:
  - trigger: only if the native desktop style exposes a stable clear action or the application deliberately adopts a different widget style.
- `P2` Analysis shell-state selectable text:
  - trigger: only if revisited as an isolated pass that does not destabilize the shipped success-body scrolling and menu contract.
- `P2` Global loading orchestration:
  - trigger: only when broader multi-surface workflows require a shared loading model.
- `P2` Global toast orchestration:
  - trigger: only when broader save/export/report flows require a notification-center model.
- `P2` Sticky left-side line numbers:
  - trigger: only if the current `ScrollView` diff viewer stops being sufficient for review ergonomics.
- `P3` Tree explorer / dual-mode workspace:
  - trigger: only if file-view-only navigation becomes a demonstrated bottleneck.

## Next implementation priority

1. Continue the remaining `Phase 16` work on top of the current `0.2.18 + edition 2024 + rust 1.94.0 + slint 1.15.1 + Phase 16A` baseline.
   - acceptance: result navigation continues to improve without introducing tree mode or breaking the accepted workspace shell.
2. Keep the shipped `15.5` to `15.8 fix-1` contracts unchanged while `Phase 16` lands.
   - acceptance: editable-input context menus, the `API Key` secret contract, `Compare Status` summary-first boundary, non-input context-menu scope, `Analysis success` native text-surface right-click, section-header left alignment, event-driven sync, and persistent `VecModel` all remain intact.

## Documentation update contract

- Update this file whenever any current architecture fact, boundary, deferred decision, or next priority changes.
- Record current facts and boundaries, not upgrade-roadmap diary text.
- Keep terminology aligned with `docs/thread-context.md` and `docs/upgrade-plan-rust-1.94-slint-1.15.md`.
- Each update must state:
  - what is completed
  - what the current baseline is
  - what intentionally stays unchanged
  - why the next step is the remaining `Phase 16` work
