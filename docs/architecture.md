# Folder Compare Architecture (Current Baseline after Phase 17D)

## Current status

- `phase15 summary` is complete as a documentation closeout.
- The following work is completed and closed:
  - `Phase 17D`
  - `Phase 17C-A`
  - `Phase 17C`
  - `Phase 17B fix-1`
  - `Phase 17B`
  - `Phase 17A fix-1`
  - `Phase 17A`
  - `Phase 16C fix-1`
  - `Phase 16C`
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
  - `Phase 17D` is complete
  - `Phase 17C-A` is complete
  - `Phase 17C` is complete
  - `Phase 17B fix-1` is complete
  - `Phase 17B` is complete
  - `Phase 17A fix-1` is complete
  - `Phase 17A` is complete
  - `Phase 16C fix-1` is complete
  - `Phase 16C` is complete
  - `Phase 16A` is complete
  - `Phase 16A fix-1` is complete
  - `Phase 16B` is complete
  - Sidebar IA remains unchanged, and `Phase 17B` keeps the accepted `Phase 16A + 16A fix-1 + 16B + 16C + 16C fix-1 + 17A + 17A fix-1` presentation contract while upgrading `App Bar -> Settings`, adding one first-round `Provider / Behavior` split, and introducing one persisted hidden-files visibility preference without reopening compare/core contracts
  - `Phase 17B fix-1` stabilizes the Settings modal container against section/provider-mode content changes, and narrows settings persistence to one authoritative `settings.toml` contract with one-time legacy migration
  - `Phase 17C` closes one low-risk workbench bug pass (`B/C/D` from `docs/ui-bug-root-cause-and-fix-plan-2026-03.md`) and finishes the Compare Inputs primary-action cleanup without reopening compare/core contracts
  - `Phase 17C-A` closes the remaining embedded `DiffStateShell` visual issue from the same bug plan and finishes the Compare Inputs action lane so the primary action now spans the full card content width instead of staying constrained to the input column
  - `Phase 17D` adds a macOS-only immersive title bar phase 1 in `fc-ui-slint`: startup now routes macOS through one explicit `window_chrome` facade, applies winit/macOS title bar attributes before window creation, and splits the top app bar into a platform-gated immersive strip vs. the existing legacy card so Windows/Linux stay on the old path byte-for-byte in behavior
- Why `Phase 17` now remains the active train:
  - the dependency-upgrade train and the edition milestone are already finished;
  - `Phase 16A`, `16A fix-1`, `16B`, `16C`, and `16C fix-1` closed the Sidebar expression, row-scanability, file-view state-consistency, and follow-up readability / typography regression pass without reopening old closeouts;
  - `Phase 17A` added a restrained tooltip-completion baseline for truncated text, and `Phase 17A fix-1` then stabilized that baseline without changing IA, row semantics, or input/menu contracts;
  - `Phase 17B` then generalized `Provider Settings` into `Settings`, added a minimal expandable preferences skeleton, and kept hidden-files visibility as a UI-side results preference instead of widening `fc-core`;
  - `Phase 17B fix-1` then tightened the Settings container and persistence contract without widening the scope into a full settings framework or compare/core policy change;
  - `Phase 17C` then used the same UI-side baseline to tighten the Compare action affordance and remove a few remaining workbench geometry inconsistencies without widening settings, search, or core behavior;
  - `Phase 17C-A` then finished the remaining embedded workbench-state-shell visual cleanup with a presentation-only pass, without widening `fc-core`, search, or settings scope;
  - `Phase 17D` then added the first macOS immersive title bar pass without adopting `no-frame`, without widening presenter/state contracts, and without forcing Windows/Linux into the new backend path;
  - future threads should therefore build on this baseline instead of reopening `15.3A` to `15.8 fix-1`, `16A` to `16C fix-1`, or edition-2024 tasks.

## Phase 15 summary

- `Phase 15.3A` aligned version ownership around the workspace manifest and packaging script.
- `Phase 15.3B` locked Rust to `1.94.0` and raised workspace `rust-version` to `1.94`.
- `Phase 15.4` moved the workspace to `slint 1.15.1` / `slint-build 1.15.1` without widening product scope.
- `Phase 15.5` shipped editable-input context-menu coverage on native Slint surfaces for `Compare Inputs`, `Filter / Scope -> Search`, and what is now `Settings -> Provider`, while keeping `Settings -> Provider -> API Key` on one dedicated `ApiKeyLineEdit` with a narrower secret contract.
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

### Platform window chrome

- `fc-ui-slint` now owns one small platform windowing facade in `src/window_chrome.rs`; `app.rs` no longer owns backend-selection details directly.
- macOS startup now performs one one-time `BackendSelector` installation before `MainWindow::new()` and uses Slint's `with_winit_window_attributes_hook()` to apply:
  - transparent title bar
  - full-size content view
  - hidden native title text
  - movable-by-window-background behavior
- Windows and Linux deliberately do not enter that path:
  - no forced `winit` backend selection
  - no platform title bar API customization
  - no top-level window-behavior fallback logic
- The root Slint window now receives read-only platform chrome properties (`immersive_titlebar_enabled`, `titlebar_visual_height`, `titlebar_leading_inset`) so presentation stays declarative while platform branching stays in Rust.
- The top app bar now has two intentionally separate render paths:
  - non-mac keeps the existing 36px `SectionCard` app bar unchanged
  - macOS renders one immersive title bar strip that visually merges into the top edge and reserves a fixed `86px` leading safety inset for traffic lights
- This phase intentionally keeps the fixed inset conservative and does not read or reposition traffic lights through AppKit.

### Compare workflow

- Compare entry flow remains state-driven:
  - left/right path input
  - validation
  - compare trigger
  - summary-first status update
- `Compare Inputs` keeps the same interaction model, with only a light presentation pass around input/browse/compare grouping.
- The `Compare` action now sits in one dedicated full-width action lane that spans the full `Compare Inputs` card content width, intentionally breaking out of the `Left / Right` label gutter instead of sharing space with an inline explanatory text label.
- The inline `Ready to compare / Running compare / Select left and right folders` text pattern is intentionally gone from `Compare Inputs`; that explanatory burden now lives in the existing summary-first compare status block and in one lightweight button tooltip only when the action is disabled or already running.
- `Compare Inputs` now add tooltip-only full-path completion on the existing left/right path inputs when the value is truncated and the field is not actively being edited, while the wrapped native `LineEdit` still clips long values inside the field instead of spilling into the rest of the Sidebar.
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
  - secondary information: concise capability-first summary for `diff / equal / left / right`
  - weak information: parent-path context for disambiguation only
  - one shared window-local tooltip now opens at the row level when the filename or weak parent-path context is visually truncated, and it only completes the full filename plus the full parent path
  - row secondary summaries no longer have their own tooltip hit target; the tooltip remains a completion aid, not a second explanation surface
  - path/name filter hits use subtle row-local label-level highlight on the matched filename or parent-path context
  - future match-span / substring highlight must come from lower-layer match positions or pre-split render segments; the Slint view layer should remain render-only and must not take on complex match parsing logic
  - the list remains flat; no tree, grouping, or alternate navigation mode was introduced
  - Search / Status filter changes keep the current selection only when the source row remains visible; otherwise the visible selection clears and the File View enters an explicit stale-selection state instead of auto-jumping to the first row
  - `Settings -> Behavior` now owns one persisted hidden-files visibility preference (`Show` / `Hide`) for dot-prefixed files and folders
  - that hidden-files preference applies immediately to the current visible Results / Navigator set and also affects future compare result presentation, but it does not widen the compare request / `fc-core` contract
  - when the hidden-files preference removes the selected row from the visible set, the application reuses the existing stale-selection contract instead of auto-restoring or auto-jumping
  - the collection summary explicitly reports when entries are currently hidden by `Settings`, so this preference does not blur into `Search` or status-scope filtering
  - compare rerun restoration stays intentionally simple: restore by the same relative path when it still exists and is still visible under the current filter; otherwise keep a stale-selection context and require an explicit reselection
  - row secondary summaries now lean toward current File View capability, especially for non-text / binary compare rows and common preview-unavailable file types, so the list better signals when the right side will land in an unavailable state
  - `Phase 16C fix-1` keeps filename-first scanning intact by shortening secondary summaries to capability-first phrases (`Text diff`, `Text-only preview`, `No text diff`, `No text preview`) and by letting weak parent-path context yield width earlier on narrow sidebars

### Diff and Analysis shell

- `Diff` keeps the compact File Context Header and the explicit shell-driven state machine:
  - `no-selection|stale-selection -> loading -> unavailable/error -> detailed-ready|preview-ready`
- `Diff` and `Analysis` now reserve the same visible top-stack rhythm in non-ready states:
  - title row
  - metadata/badge row
  - helper strip
  - shell/content body
- `Diff` now keeps the helper strip mounted in every state; ready rows keep the existing copy affordance, and non-ready rows reuse restrained contextual guidance instead of collapsing the strip.
- `DiffStateShell` now keeps two presentation modes:
  - standalone state card
  - embedded workbench shell
- The embedded shell mode now behaves as an internal workbench state layer rather than a second card:
  - a layout-driven compact badge lane replaces the old absolute badge placement
  - `neutral` stays `neutral` instead of being remapped to `info`
  - the left accent is reduced to a subdued `0-1px` edge that stays visually subordinate to the workbench border
  - the title/body block starts on the same restrained left padding rhythm as the rest of the workbench content
- Single-side preview remains first-class:
  - `left-only`, `right-only`, and `equal` all use the preview path when appropriate
  - preview columns stay side-aware (`left/right`)
- `Diff` detail keeps the current ergonomics baseline:
  - selectable line content
  - double-click line-number / hunk-marker copy fallback
  - explicit `ScrollView` viewport for horizontal scrolling
  - mirrored header `viewport-x`
  - content-end scrollbar-safe spacer
  - one shared column geometry contract between header and body separators, so horizontal scrolling no longer exposes header/body divider drift
- `Analysis` keeps its explicit state machine:
  - `no-selection|stale-selection -> waiting|ready|unavailable -> loading -> error|success`
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

- `Compare Inputs`, `Filter / Scope -> Search`, and `Settings -> Provider` ordinary inputs use the native editable-input context menu from `slint 1.15.1`.
- Those ordinary editable inputs now share one CJK-safe typography token (`UiTypography.editable_input_font_family`) so full-width punctuation and mixed Latin/CJK input stay stable on the `slint 1.15.1` baseline instead of relying on the default widget font chain.
- `Compare Inputs` left/right path fields and `Filter / Scope -> Search` now opt into the same window-local tooltip completion layer when long text is visually truncated and the input is in a non-editing state, while the custom wrapper keeps the editable control sized to the visible field so long values do not break the Sidebar boundary.
- `Settings -> Provider -> API Key` keeps one dedicated `ApiKeyLineEdit`:
  - hidden state: `Paste` only
  - visible state: `Select All`, `Copy`, `Paste`, `Cut`
  - hidden state still blocks hidden-state `Cmd/Ctrl+A/C/X`
- `ApiKeyLineEdit` now uses the same editable-input typography token as the ordinary inputs, so the secret field does not diverge from the rest of the input surfaces on glyph fallback behavior.
- `Search` keeps its explicit `Clear` button because the current native `cupertino` `LineEdit` style still does not expose a stable clear affordance.

### Typography, scrolling, and feedback

- `SelectableDiffText` and `SelectableSectionText` share `UiTypography.selectable_content_font_family`.
- That shared token remains the accepted fix for the Slint `1.15.1` mixed Latin/CJK glyph fallback regression.
- The tooltip overlay also uses the shared readable-content typography path, so long filename/path completion and the restrained Compare-action state hint do not introduce a second glyph-fallback policy.
- The shared tooltip overlay now prefers showing above its anchor surface and falls back below when top space is insufficient; no separate tooltip controller or native tooltip binding was introduced.
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

### Workspace shell simplification

- The outer workspace wrapper is now visually transparent, and the inner workbench host/panel remains the one primary visible surface on the right side.
- The workbench host now expands to the full transparent workspace wrapper instead of keeping an extra inner inset, so the right-side workbench and its loading mask use the same available height/width rhythm as the sidebar column without changing IA.
- The workspace loading mask is now mounted against the same workbench host surface users perceive as the actual file-view shell, so loading boundaries no longer expose the old outer-card box.
- The top border bridge under the workspace tabs now respects the workbench panel corner radius instead of drawing straight through the rounded outer corners, which removes the visible left/right seam at the tab-to-panel connection.

### Settings and persistence

- Settings now live in one global modal launched from `App Bar -> Settings`.
- The Settings modal now keeps one fixed desktop-tool footprint based on the current largest content state, so switching `Provider / Behavior` and `Mock / OpenAI-compatible` does not resize the outer container.
- The first-round Settings split is intentionally small:
  - `Provider`
  - `Behavior`
- `Provider` keeps the existing AI provider configuration flow and dedicated `API Key` secret-field contract.
- `Behavior` currently exposes one persisted preference: whether dot-prefixed files and folders are shown by default in `Results / Navigator`.
- `Hidden files` is currently a UI / presentation preference: it changes the default visible Results / Navigator set and its summary copy, but it does not change compare request semantics, compare-summary counts, or any `fc-core` API contract.
- The project should only reconsider moving hidden-file policy into `fc-core` when compare request building, shared statistics, export, cache behavior, or future tree-mode navigation all need the same policy source.
- Persistence stays in `settings.rs`.
- Saved settings now use `settings.toml` with the existing config-dir override.
- Legacy `provider_settings.toml` is no longer a standing fallback contract; it is only a one-time migration source when `settings.toml` is absent, and successful migration re-establishes `settings.toml` as the only active baseline.
- The edition-2024 milestone did not change the product contract here; it only retained direct compatibility fixes around settings load/save lock lifetime and test-only directory override handling.

## What intentionally did not change

- No `Phase 16` work was mixed into the phase15 closeout.
- No new IA, tree mode, or Compare View mode was introduced.
- No compare-level hidden-entry policy was pushed into `fc-core`; hidden-files visibility remains a first-round UI preference only.
- No new theme system, global loading controller, or global notification controller was introduced.
- No global tooltip controller, native-backend tooltip binding, or explanation-heavy hover system was introduced beyond the restrained Compare-action state hint.
- No character-level substring highlight was introduced; current results highlighting remains the low-cost label-level pass described above.
- No overlay interception, private pointer plumbing, or custom caret/selection/editing logic was added for editable inputs or selectable text.
- The large inline `slint::slint!` surface was not externalized because the cleanup benefit is still below the migration cost on the current baseline.
- No `no-frame` window mode, custom resize hit-testing, `objc2`, or raw AppKit `NSWindow` manipulation was introduced for the immersive title bar phase.
- Windows and Linux were not forced into the new backend-selection path; zero-behavior-change is preserved by keeping those platforms out of the new code path entirely.

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
- `P2` Compare-level hidden-entry policy:
  - trigger: only if users need `Compare Status` and compare-summary counts to exclude hidden entries at source rather than via the current UI-level visibility preference.
  - boundary: do not mix this with the first-round `Settings -> Behavior` skeleton; widening `fc-core` must be justified separately.
- `P2` Results match-span highlight:
  - trigger: only if the current label-level highlight is demonstrated to be insufficient for navigation speed on the flat list.
  - boundary: matching positions or pre-split render segments must come from a lower layer; the Slint view stays render-only and does not own substring parsing logic.
- `P2` Analysis shell-state selectable text:
  - trigger: only if revisited as an isolated pass that does not destabilize the shipped success-body scrolling and menu contract.
- `P2` Global loading orchestration:
  - trigger: only when broader multi-surface workflows require a shared loading model.
- `P2` Global toast orchestration:
  - trigger: only when broader save/export/report flows require a notification-center model.
- `P2` Sticky left-side line numbers:
  - trigger: only if the current `ScrollView` diff viewer stops being sufficient for review ergonomics.
- `P2` macOS title bar runtime fine-tuning:
  - trigger: only if manual smoke proves that the fixed leading inset or `with_movable_by_window_background(true)` is insufficient on supported macOS versions.
  - boundary: keep the follow-up inside `fc-ui-slint` first; do not jump to `objc2`, raw `NSWindow`, or traffic-light repositioning without a separate design pass.
- `P3` Tree explorer / dual-mode workspace:
  - trigger: only if file-view-only navigation becomes a demonstrated bottleneck.

## Next implementation priority

1. Continue the remaining `Phase 17` work on top of the current `0.2.18 + edition 2024 + rust 1.94.0 + slint 1.15.1 + Phase 16A + 16A fix-1 + 16B + 16C + 16C fix-1 + Phase 17A + Phase 17A fix-1 + Phase 17B + Phase 17B fix-1 + Phase 17C + Phase 17C-A + Phase 17D` baseline.
   - acceptance: later UI work does not regress the accepted macOS immersive title bar contract or the non-mac legacy app bar baseline.
2. Keep the shipped `15.5` to `15.8 fix-1` contracts plus the `Phase 17A` through `Phase 17D` settings/results/window-chrome boundary unchanged while later `Phase 17` work lands.
   - acceptance: editable-input context menus, the `API Key` secret contract, `Compare Status` summary-first boundary, non-input context-menu scope, `Analysis success` native text-surface right-click, section-header left alignment, event-driven sync, persistent `VecModel`, tooltip-as-completion-only, and the first-round `Settings -> Provider / Behavior` skeleton all remain intact.

## Documentation update contract

- Update this file whenever any current architecture fact, boundary, deferred decision, or next priority changes.
- Record current facts and boundaries, not upgrade-roadmap diary text.
- Keep terminology aligned with `docs/thread-context.md` and `docs/upgrade-plan-rust-1.94-slint-1.15.md`.
- Each update must state:
  - what is completed
  - what the current baseline is
  - what intentionally stays unchanged
  - why the next step is the remaining active phase work
