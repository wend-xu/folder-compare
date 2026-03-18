# Folder Compare Architecture (Phase 1-15.2D stable baseline + dependency-upgrade closeout roadmap)

## Crate responsibilities

- `fc-core`: owns directory compare and text diff domain model, public API, and error contracts.
- `fc-ai`: owns AI-based interpretation for diff outputs through a provider abstraction.
- `fc-ui-slint`: owns desktop app entry, app state orchestration, and UI presentation.

## `fc-core` internal boundaries (Phase 7)

- `api/`: external entry points (`compare_dirs`, `diff_text_file`).
- `domain/`: pure domain types (requests/options/report/entry/diff/error).
- `services/`:
  - `scanner`: recursive traversal and indexed scan output per root;
  - `comparer`: left/right path alignment, node classification, and report entry assembly;
  - `large_dir`: soft/hard limit evaluation and policy planning for large-directory protection;
  - `hasher`: deterministic file-level content comparison (`size + bytes`) as fallback;
  - `text_loader`: text candidate detection + BOM/encoding-aware decode boundary;
  - `text_diff`: summary-level diff for `compare_dirs` plus detailed diff building for `diff_text_file`.
- `infra/`: path normalization and relative-path key generation plus thin fs helpers.

The dependency direction is kept as: `api -> services -> domain/infra`, and `domain` does not depend on `services`.

## Why `fc-core` must stay pure

`fc-core` is the foundation for correctness and determinism. It must stay independent from UI/runtime frameworks and remote AI services so that:

- core compare behavior remains testable and reusable;
- future CLI, UI, and service adapters can share the same engine;
- behavior does not depend on network state or provider variability.

## Why AI analysis is outside the core engine

AI analysis is optional, probabilistic, and provider-dependent. Core compare output must be available even when AI is disabled or unavailable. Keeping AI in `fc-ai` preserves a strict boundary between deterministic compare results and optional interpretation.

## Why UI only handles orchestration and presentation

UI should not embed compare business logic. `fc-ui-slint` translates user intent into calls to `fc-core`/`fc-ai`, then renders results. This keeps domain logic centralized and easier to test.

## `fc-core` API maturity after Phase 7

- `compare_dirs` now performs:
  - request validation and root normalization;
  - recursive scan and path alignment;
  - directory-level soft/hard limit evaluation for aligned entries and total bytes;
  - policy-based large-directory handling:
    - `Normal`: continue with warnings in large mode;
    - `SummaryFirst`: return summary-first output and allow truncation under hard limits;
    - `RefuseAboveHardLimit`: fail fast with structured error under hard limits;
  - text detail deferral in large mode and for oversized text files;
  - deterministic file-level comparison fallback when text path is deferred or unavailable.
- `diff_text_file` now performs:
  - full input validation and path normalization;
  - text loading via shared Phase 5 detection/decoding boundary;
  - per-file detailed diff input size guard (`max_file_size_bytes`) with structured boundary error;
  - structured detailed diff output with hunks, lines, line kinds, and line numbers;
  - local output limiting via `truncated + warning`.
- The report can now express:
  - left-only / right-only paths;
  - type mismatch between aligned paths;
  - aligned directories;
  - aligned files as `Equal` or `Different` from text summary, deferred text detail, or byte-level fallback;
  - large-mode and summary-first-mode flags in summary;
  - deferred-detail counters and oversized-text counters;
  - report-level truncation and warning messages for policy-triggered limits.
- `compare_dirs` remains summary-oriented and does not emit detailed hunk output.

## `fc-ai` API maturity after Phase 12

- DTO contract now includes:
  - request context (`task`, `relative_path`, `language_hint`, `diff_excerpt`, `summary`, `truncation_note`, `config`);
  - response fields for UI cards (`risk_level`, `title`, `rationale`, `key_points`, `review_suggestions`);
  - provider-neutral `PromptPayload`.
- Structured `AiError` now distinguishes:
  - invalid request;
  - prompt build failure;
  - input preparation failure;
  - provider execution failure;
  - response parse failure;
  - not implemented.
- Analyzer orchestration now performs:
  - request validation;
  - diff input preparation/truncation with note propagation;
  - provider-neutral prompt payload assembly;
  - provider invocation;
  - response normalization and parse-boundary checks.
- Mock provider now:
  - deterministic;
  - supports `Summary`, `RiskReview`, and `ReviewComments`;
  - produces stable `risk_level`, `title`, `rationale`, `key_points`, and `review_suggestions`.
- OpenAI-compatible provider now supports a minimal real execution path:
  - config fields: `endpoint`, `api_key`, `model`, `timeout`;
  - OpenAI-compatible `chat/completions` request construction;
  - response extraction from `choices[0].message.content`;
  - JSON contract mapping to `AnalyzeDiffResponse`;
  - structured provider execution failure kinds (`missing endpoint/key/model`, invalid endpoint, timeout, network failure, HTTP non-success);
  - structured response parse failure kinds (`invalid json`, missing content, invalid contract).

## `fc-ui-slint` current architecture snapshot (Phase 15.8 fix-1 shipped; Phase 16 back on mainline)

### IA and layout contract

- IA remains fixed as:
  - `App Bar`;
  - `Sidebar` (`Compare Inputs / Compare Status / Filter / Scope / Results / Navigator`);
  - `Workspace` (`Diff / Analysis` tabs + header + content).
- Workspace still behaves as one continuous shell (`Tabs -> Header -> Content`) with conditional mode branches, so only one mode participates in layout at a time.
- Workspace tab chrome now uses a dedicated connected tab strip rather than a segmented rail; the active tab attaches to the workbench surface with reduced overlap, shared seam repair, and header-aligned fill so Diff/Analysis read as one connected station.
- Sidebar width and overflow are constrained for desktop density; long text is handled by elide/wrap rules instead of intrinsic-width pushback.

### Compare workflow contract

- Compare entry flow is state-driven:
  - left/right path input + native folder browse;
  - validation (`empty/missing/not-directory/unreadable`);
  - compare trigger and summary-first status update.
- Filter flow is state-driven:
  - text search + segmented status scope (`All / Diff / Equal / Left / Right`);
  - segmented visual state stays in lockstep with filter state.
- Compare Status remains summary-first by design:
  - primary status + compact pills + key metrics;
  - details stay lightweight/expandable, not a second heavy pane.

### File View shell and preview contract

- Diff tab now uses a compact File Context Header instead of an always-expanded three-row stack:
  - primary: selected relative path remains the dominant line inside one embedded Diff workbench panel instead of a detached header strip;
  - compact meta row uses fixed-height rhythm so selected/unselected/loading/unavailable states keep the same vertical cadence;
  - compact meta row: mode (`Detailed Diff` / `Preview`) + result status + concise summary;
  - weak hints (`type`, preview source, truncation) stay inline and only escalate to a state pill when loading/unavailable/error needs stronger emphasis.
- Diff mode state machine is now explicit and shell-driven:
  - `no-selection -> loading -> unavailable/error -> detailed-ready|preview-ready`;
  - `detailed-ready|preview-ready` can still use shell fallback for empty line payloads.
- `DiffStateShell` is the unified container for non-renderable states in Diff mode and now fills the detail region as a formal state surface rather than a centered card.
- Analysis mode now shares the same outer workbench surface and connected tab chrome as Diff, but it has its own explicit product state machine:
  - `no-selection -> not-started -> loading -> error|success`;
  - `No Selection` and `Not Started` are distinct product states, not one empty placeholder.
- Analysis header now stays compact and shell-aligned:
  - primary: selected relative path remains the dominant line;
  - compact meta row follows the same fixed-height cadence as Diff header and uses `mode + state + weak provider-readiness + one-line summary`;
  - helper strip keeps provider/readiness/timeout/truncation as weak technical context rather than the primary message.
- Analysis success content is now a structured review-conclusion panel instead of raw text stacking:
  - `Summary`;
  - `Risk Level`;
  - `Core Judgment`;
  - `Key Points`;
  - `Review Suggestions`;
  - `Notes` for truncation/warning/mock-provider caveats.
- Analysis success keeps `helper strip -> action strip -> scrollable content` as one workbench surface:
  - header/helper/action remain fixed in the shell;
  - long review content scrolls independently inside the success body instead of stretching the whole workspace;
  - action strip owns low-noise global actions such as `Open Diff` and `Copy All`.
- Analysis copy baseline is intentionally lightweight:
  - section cards expose inline `Copy` actions instead of persistent button walls;
  - `Copy All` exports the current structured review conclusion;
  - success sections (`Summary/Core Judgment/Key Points/Review Suggestions/Notes`) support direct text selection and native system copy shortcuts;
  - the read-only selectable wrappers behind these success sections now consume one shared `UiTypography.selectable_content_font_family` token, so mixed Latin/CJK text does not regress to tofu glyphs after the `slint 1.15.1` text-engine migration without repeated prop plumbing;
  - explicit copy actions (`Copy` / `Copy All`) now trigger top-center overlay `toast` feedback;
  - native shortcut copy for selected success text currently keeps system-default behavior; toast feedback for this path is deferred until Slint exposes a stable native copy callback surface in this shell;
  - success-body `SelectableSectionText` now ships its own Slint-native text-surface right-click menu with `Copy` / `Select All` on the current selected text only, while section header/chrome keeps the existing window-local `Copy` / `Copy Summary` menu;
  - Analysis success section inline `Copy` actions remain lightweight buttons and are no longer blocked by the header right-click hotspot; the hotspot now stays on the header label lane instead of covering the whole header row;
  - Analysis success section header labels (`Summary/Core Judgment/Key Points/Review Suggestions/Notes`) remain explicitly left-aligned inside that narrowed header label lane; `Phase 15.8 fix-1` restored this after the `15.8` hotspot refactor accidentally dropped the label text geometry/alignment contract and let the text fall back to centered layout inside a plain container;
  - Analysis shell-state body text selection (`no-selection/not-started/loading/error`) is not a hard requirement in `Phase 15.1B fix-3` and remains out of scope for this round;
  - if shell-state text selection is revisited, evaluate it as an isolated pass and do not couple it with success-body scroll stabilization changes.
- Analysis non-success states now reuse the same embedded `DiffStateShell` visual grammar as Diff, so shell hierarchy stays consistent without reintroducing detached cards.
- Workspace shell visual grammar is now part of the contract: tabs have no inter-tab gap, selected tab fill aligns with header surfaces, lower tab edges are straight, and tab/panel seam lines must remain continuous without detached pill styling.
- Neutral `no-selection` shell indicators stay within the cool workspace palette rather than using a warm generic neutral badge.
- Results / Navigator selection remains the canonical `selected row -> Diff` entry path:
  - selecting any result row forces the workspace back to Diff and refreshes that file's diff context;
  - Analysis can return to the same selected file's Diff surface via tab switch or success action strip without introducing a second state path.
- Single-side preview remains first-class:
  - `left-only`, `right-only`, and `equal` all enter preview path;
  - equal preview is not blocked by detailed-diff eligibility;
  - preview table columns are side-aware (`left/right`) instead of always `old/new`.
- Diff content keeps the `column header -> rows` structure but now guarantees baseline review ergonomics:
  - line content is selectable/copyable;
  - read-only selectable diff rows use the same shared `UiTypography.selectable_content_font_family` token as Analysis success text, because `slint 1.15.1` changed `TextInput` font fallback behavior and otherwise rendered some full-width punctuation in mixed Latin/CJK lines as tofu on macOS;
  - the workbench panel owns `header -> helper strip -> state/table` as one surface, so Diff detail reads as a single workstation instead of nested cards;
  - the table can scroll horizontally for long lines with persistent in-surface guidance;
  - the diff body now uses an explicit `ScrollView` viewport instead of relying on `ListView` horizontal-scroll exposure, while the column header mirrors the same horizontal viewport;
  - the diff body keeps a content-end scrollbar-safe spacer so the last rows stay selectable/copyable and no longer sit under the horizontal scrollbar;
  - lightweight row-copy fallback moved from always-visible per-row buttons to double-click-on-line-number/hunk-marker hotspots that copy the full underlying line text, not just the visible viewport fragment;
  - transient copy feedback is explicit but restrained (`Line N copied` / `Copy failed`) and now uses top-center overlay `toast` for copy actions;
  - window-local `toast-controller` is toast-only (overlay placement), and copy-related helper/action-strip pill has been removed in this phase;
  - overlay `toast` also covers low-risk success/info notifications (for example, provider settings save confirmation);
  - fixed left-side line numbers remain deferred in this phase.
- `can_load_diff = false` and preview capability boundaries map to explicit `unavailable` (not generic failure).
- No new AI schema, sidebar IA, compare mode, or task orchestration layer was added for `15.1B`; the productization remains presentation/state-derivation only.

### Provider settings and persistence contract

- Provider settings are edited in a global modal (`App Bar -> Provider Settings`), not embedded in Analysis content.
- Persistence is owned by `settings.rs`, loaded at presenter initialize-time, and written to `provider_settings.toml` (with `FOLDER_COMPARE_CONFIG_DIR` override).
- Provider config model in scope:
  - `Mock` / `OpenAI-compatible`;
  - endpoint / api key / model / timeout;
  - API key visibility toggle semantics now live inside one field-local `ApiKeyLineEdit`, not a detached extra button;
  - hidden API key context-menu policy stays conservative (`Paste` only), while visible state can expose full edit actions.

### Runtime synchronization contract

- Compare/diff/analysis work still runs in background workers, but async completion now pushes a fresh `AppState` snapshot back to the UI thread through presenter notifier + `slint::Weak::upgrade_in_event_loop`, instead of relying on a repeated `50ms` polling timer.
- Rebinding and refresh are bounded:
  - result/diff models are initialized once as persistent `VecModel` instances and only receive `set_vec()` updates when relevant source state changes;
  - cache-aware sync still skips redundant property/model projection when the latest snapshot is unchanged;
  - loading-mask projection is still derived only from busy flags, but timeout copy is now scheduled by busy-phase transitions through one-shot timers rather than by a repeated watchdog tick;
  - row delegate local state reduces repeated indexed binding evaluation;
  - header stats avoid cloning filtered rows only for count computation.

### Local loading-mask baseline (Phase 15.2B)

- Loading-mask is a lightweight local UI component in `fc-ui-slint` only (no new global loading controller, no presenter/open API expansion).
- Mask lifecycle is still derived from existing busy flags only:
  - `running`: lock Sidebar `Compare Status / Filter / Scope / Results / Navigator` and lock the whole `Workspace`;
  - `diff_loading`: lock the whole `Workspace`;
  - `analysis_loading`: lock the whole `Workspace`.
- `App Bar` and `Provider Settings` modal are outside the loading-mask scope in this phase.
- Existing `enabled` bindings remain the primary control logic; loading-mask only adds overlay-level input interception.
- Mask overlay keeps corner-radius alignment with host surfaces to avoid seam/clip regressions in Sidebar and Workspace.
- Timeout handling remains UI-local behavior only:
  - before timeout: show normal loading copy;
  - after timeout: degrade copy to `Taking longer than expected...`;
  - timeout never mutates compare/diff/analysis business state and never auto-closes mask;
  - timeout scheduling is driven by busy-phase start/reset, not by a broad repeated polling loop.
- Minimal extra UI protection: while `diff_loading`, navigator row click is blocked to prevent selection/context drift caused by `SelectRow` racing with in-flight diff completion.

### Local semantic palette boundary (Phase 15.2C)

- `fc-ui-slint` now owns a local semantic palette + typography token file: `crates/fc-ui-slint/src/ui_palette.slint`.
- This extraction is intentionally narrow and reuses existing visual hierarchy:
  - semantic tones for `StatusPill`: `neutral/info/success/warn/error/different/equal/left/right`;
  - shared semantic surfaces for `DiffStateShell`, Analysis result/risk sections, and Results row status border/background/text;
  - one shared typography token for read-only selectable content (`UiTypography.selectable_content_font_family`);
  - Phase `15.2` infra colors for local `toast-controller` overlay and `loading-mask`;
  - `context-menu` core visual tokens for panel border/background/shadow plus item hover/disabled/accent state.
- Boundary remains local to Slint layer only:
  - no Rust-side color struct/model was introduced;
  - no runtime theme switching;
  - no Provider Settings visual-system upgrade;
  - no full-application hex cleanup outside semantic color contracts.

### Local context-menu core baseline (Phase 15.7 visual-polish accepted state)

- `fc-ui-slint` now owns one shared window-local context-menu core for non-input safe surfaces only.
- Lifecycle and short-lived menu state stay in UI orchestration:
  - menu open/close;
  - anchor position near the right-click point;
  - target/context token;
  - action dispatch and custom-action handler storage;
  - auto-close on outside click, tab switch, compare/analyze rerun, selected-row change, busy-start, and user scroll on `Results / Navigator` plus Analysis success `ScrollView`.
- Business state remains outside the menu core:
  - no menu state was added to `AppState`;
  - no presenter/core/ai contract was expanded for menu actions.
- Current public baseline is intentionally narrow:
  - built-in actions are `Copy` and `Copy Summary` only;
  - no fake `Paste/Cut/Select All` is exposed on non-input surfaces;
  - custom actions are supported locally with a hard cap of 10 entries per caller, truncated after the first 10.
- `Phase 15.7` 只提升了 visual layer：
  - menu panel 现已使用分层阴影、收敛圆角/边框/内边距和顶部高光；
  - item hover 现已是 inset surface + 左侧 accent strip；
  - disabled item 现已使用更柔和的禁用态文本；
  - lifecycle、action dispatch、anchor positioning、close-on-scroll/busy/selection contract 保持不变。
- Current safe-surface integration points:
  - `Results / Navigator` item;
  - `Workspace` file context header (`Diff` and `Analysis` header surface);
  - `Analysis` success section header/card chrome (`Summary`, `Core Judgment`, `Key Points`, `Review Suggestions`, `Notes`).
- `Risk Level` keeps explicit `Copy` button only in this phase and no longer participates in context-menu coverage.
- Even after `Phase 15.5`, this shared controller remains non-input only:
  - editable inputs do not route through the window-local `Copy / Copy Summary` controller;
  - `SelectableSectionText` and `SelectableDiffText` still remain outside this menu core;
  - `Phase 15.8` kept Analysis success text-body right-click outside this controller as well, and `Phase 15.8 fix-1` did not change that boundary.
- `15.2D` is designed to stand on its own:
  - safe surfaces already get reusable menu open/close/dispatch behavior now;
  - future input integration can reuse the same shared core without being required for phase-16 progression.

### Editable input integration status (post-15.8 fix-1 baseline)

- `15.2E` is now shipped on top of the upgraded dependency baseline without expanding the existing non-input menu controller.
- Dependency and packaging baseline after `Phase 15.8 fix-1`:
  - Rust toolchain is fixed at `1.94.0`;
  - workspace `rust-version = 1.94`;
  - workspace `slint` / `slint-build` are pinned to `1.15.1`;
  - workspace version remains `0.2.17` after `Phase 15.8 fix-1`;
  - release version, macOS bundle version, and DMG/ZIP artifact version now derive from the workspace manifest version.
- Product outcome:
  - `15.2D` shell/menu/loading/toast boundaries remained behavior-equivalent on the new dependency baseline;
  - editable-input menu coverage is now live on `Compare Inputs`, `Filter / Scope -> Search`, and `Provider Settings`;
  - non-input context menu has now completed one style-only visual polish pass without changing controller ownership or menu scope;
  - Analysis success `SelectableSectionText` now exposes native text-surface `Copy` / `Select All` coverage on the selected text only, without changing the window-local non-input menu controller;
  - Analysis success section header labels now keep explicit left alignment after the `15.8` header-hotspot narrowing; `15.8 fix-1` was a geometry-only regression repair, not a new menu-scope change;
  - read-only selectable content (`SelectableDiffText` / `SelectableSectionText`) now carries one shared `UiTypography` glyph-fallback token so full-width punctuation from real workspace text stays readable after the Slint upgrade without repeated view-level prop threading;
  - macOS arm64 manual smoke passed after `Phase 15.4`, and launch-level smoke also passed after `Phase 15.5`, `Phase 15.6`, `Phase 15.7`, `Phase 15.8`, and `Phase 15.8 fix-1`;
  - diff loading still feels perceptibly faster on the upgraded baseline, and `Phase 15.6` removed the old high-frequency sync path without adding new product scope.
- Rejected implementation paths remain rejected after `15.5`:
  - overlay-style `TouchArea` interception would risk left-click caret placement, drag selection, IME behavior, and native shortcut flow;
  - private/global pointer interception would leak menu lifecycle outside the current UI-local boundary and raise focus/passive-sync risk;
  - custom caret/selection plumbing would duplicate editor behavior and violate the “reuse Slint editing logic” constraint.
- Standard editable-input coverage now relies on Slint-native surfaces:
  - `Compare Inputs -> left/right` use the built-in `LineEdit` context menu from `slint 1.15.1`;
  - `Filter / Scope -> Search` uses the same native `LineEdit` context menu and keeps its explicit `Clear` button for now;
  - `Provider Settings -> Timeout / Endpoint / Model` use the same native `LineEdit` context menu;
  - no overlay interception, private pointer plumbing, or custom caret/selection logic was added.
- `API Key` is intentionally handled as a narrower local exception:
  - `Provider Settings -> API Key` now uses one dedicated `ApiKeyLineEdit` built on native `TextInput` + `ContextMenuArea`;
  - hidden state menu exposes `Paste` only and blocks hidden-state `Cmd/Ctrl+A/C/X` shortcuts before the default `TextInput` handler runs;
  - visible state re-enables `Copy`, `Cut`, `Paste`, and `Select All`;
  - the old detached `Show/Hide` button collapsed into one field-local reveal toggle so visibility changes no longer pull focus into a separate modal action button.
- Conservative `API Key` menu policy remains the accepted contract:
  - hidden state: `Paste` only;
  - visible state: `Select All`, `Copy`, `Paste`, `Cut`;
  - rationale: hidden state should not imply masked text is safely copyable/cuttable, while `Paste` remains the least surprising secret-entry action.
- What intentionally still did not change in `15.5`:
  - `Search` keeps its explicit `Clear` button because the current macOS native `cupertino` `LineEdit` style does not provide a stable built-in clear affordance;
  - `SelectableDiffText` still does not expose its own context-menu coverage on the shipped baseline;
  - Analysis shell-state text and editable inputs did not change scope when `SelectableSectionText` gained its Analysis-success-only native text-surface coverage; the window-local menu core remains unchanged.

### Boundaries and non-goals in this phase

- No IA reset.
- No Compare View/tree-mode expansion.
- No AI schema/provider capability expansion beyond UI-state orchestration.
- No deep Compare Status details expansion beyond summary-first intent.
- No runtime theme/settings upgrade or cross-surface theme controller.
- No selectable-text-wrapper context-menu expansion beyond the shipped editable-input scope.

## `fc-ui-slint` evolution highlights (Phase 13.1 -> 15.8 fix-1 shipped)

- 13.1 -> 14.2:
  - stabilized IA and desktop-density visual grammar;
  - finished Compare Inputs/Filter/Status interaction baseline;
  - moved provider settings into modal flow and introduced persistence boundary.
- 15.0 -> fix-5:
  - unified Workspace shell and explicit Diff/Analysis state grammar;
  - completed semantic status-color contract for Results;
  - promoted single-side/equal preview to first-class File View path;
  - hardened runtime refresh to reduce unnecessary model churn during interaction.
- 15.1A:
  - converged Diff File View shell into a stable `Header -> State Shell -> Content` contract;
  - elevated Diff context recognition (path/mode/status/reason) to product-grade hierarchy;
  - improved detailed diff readability with clearer column/hunk/line rhythm and preview-aware columns.
  - fix-1 tightened the Diff header, strengthened shell-state emphasis, and added selectable + horizontally scrollable line content for long-line review.
  - fix-2 stabilized header cadence, turned `DiffStateShell` into a detail-surface state panel, and added row-copy + scrollbar-safe affordances for long-line review.
  - fix-3 collapsed Diff detail into one workbench panel, moved copy fallback to double-click line-number hotspots with clearer transient feedback, decoupled the vertical scrollbar from horizontal scroll position, and completed the final visual convergence pass for connected workspace tabs / seam repair / neutral `No Selection` shell tone.
- 15.1B:
  - productized Analysis View inside the accepted File View shell instead of treating it as a raw AI text dump;
  - introduced an explicit five-state Analysis surface with separate `No Selection` and `Not Started` semantics;
  - rebalanced provider/readiness/timeout into weak helper context while promoting summary/risk/judgment/suggestions to the primary reading flow;
  - fix-1 aligned Analysis header cadence with Diff, landed lightweight section/whole-review copy actions, and routed copy feedback through a shared weak-feedback pill;
  - fix-2 stabilized independent success-body vertical scrolling with geometry-driven section stacking and dynamic scrollbar visibility;
  - fix-3 landed selectable text inside Analysis success sections (`Summary/Core Judgment/Key Points/Review Suggestions/Notes`) while keeping shell-state selectable text out of scope;
  - kept Diff shell, sidebar IA, and AI response schema unchanged.
- 15.2A:
  - narrowed local `toast-controller` responsibility to overlay toast only;
  - docked copy-action toast feedback to Diff row double-click copy and Analysis success `Copy`/`Copy All`, and removed the corresponding helper/action-strip copy pill;
  - kept Analysis success native shortcut copy (`selection + system copy`) on system-default path without toast hook due current callback boundary.
- 15.2B:
  - introduced reusable local `loading-mask` overlay with spinner/copy/interception, scoped to Sidebar lower controls + Workspace by busy-flag derivation;
  - added a local timeout watchdog that only downgrades UI copy and does not write back business state;
  - fixed short-lived diff loading visibility by projecting loading-mask immediately after state sync (not only on timer tick);
  - kept presenter/core/ai contracts unchanged and deferred any global loading orchestration API.
- 15.2C:
  - extracted a narrow Slint-layer semantic `ui_palette` for `StatusPill`, `DiffStateShell`, Results status rows, Analysis risk/section semantic surfaces, and local `toast/loading-mask`;
  - kept tone semantics and visual hierarchy unchanged while reducing duplicated semantic hex values;
  - explicitly deferred runtime theme switching, Provider Settings visual upgrade, and full layout/surface color cleanup.
- 15.2D:
  - added one shared window-local context-menu core with `Copy` / `Copy Summary` actions, right-click anchor positioning, outside-click close, scroll close on `Results`/Analysis success scroll hosts, and action dispatch by target token;
  - connected only non-input safe surfaces (`Results / Navigator`, `Workspace` file context header, `Analysis` success section chrome except `Risk Level`);
  - normalized `AnalysisSectionPanel` anchor coordinates so section menus open near the actual pointer location;
  - kept `SelectableSectionText` / `SelectableDiffText` / all editable inputs out of scope so the menu core remains independently stable and does not depend on `15.2E`.
- 15.2E assessment:
  - confirmed that the old `slint = 1.8.0` baseline was the blocker for low-risk editable-input menus;
  - kept `Compare Inputs`, `Filter / Scope -> Search`, `Provider Settings` inputs, and all selectable-text wrappers deferred until the upgraded baseline was stable.
- 15.3A -> 15.4:
  - unified version ownership around the workspace manifest and packaging-script derivation;
  - locked Rust `1.94.0` and Slint `1.15.1` while preserving accepted `15.2D` shell/menu/loading/toast boundaries;
  - kept the large inline `slint::slint!` surface and current sync design intact because the dependency migration compiled cleanly without a UI rewrite;
  - manual smoke on macOS arm64 passed and diff loading responsiveness improved perceptibly relative to the pre-upgrade baseline.
- 15.5:
  - shipped editable-input menu coverage by reusing Slint-native `LineEdit`/`ContextMenuArea` surfaces for `Compare Inputs`, `Filter / Scope -> Search`, and `Provider Settings`;
  - added one dedicated `ApiKeyLineEdit` local shell so `API Key` can keep hidden=`Paste` only while still relying on native `TextInput` editing behavior;
  - collapsed the old detached API-key `Show/Hide` button into a field-local reveal affordance and left `Search` clear explicit because the current native `cupertino` style still lacks a stable built-in clear action.
- 15.6:
  - replaced the old repeated `50ms` UI polling path with presenter-driven async completion pushes back to the UI thread;
  - narrowed loading-mask timeout handling to busy-phase-triggered one-shot timers instead of repeated watchdog ticking;
  - converted `Results / Navigator` and `Diff` row data to persistent `VecModel` instances updated via `set_vec()` only when the related payload changes;
  - evaluated `.slint` externalization and intentionally kept the large inline `slint::slint!` surface plus current `build.rs` boundary because cleanup benefit was not yet strong enough to justify migration churn.
- 15.7:
  - kept the existing window-local context-menu controller and safe-surface coverage untouched;
  - polished the menu visual layer with tighter panel geometry, layered shadow, inset hover surface, accent strip, and softer disabled-state text;
  - kept selectable-text row menus, native platform menu bridging, and any Phase 16 navigation work out of scope.
- 15.8:
  - shipped one narrow closeout: Analysis success `SelectableSectionText` now uses Slint-native text-surface `Copy` / `Select All` coverage for selected text only;
  - kept section header/chrome on the existing window-local menu and did not reopen controller ownership or menu-scope decisions;
  - fixed the accidental regression where Analysis success section inline `Copy` buttons were blocked by the header-wide right-click hotspot by narrowing that hotspot to the header label lane only;
  - kept `SelectableDiffText`, Analysis shell-state text, editable inputs, and Phase 16 navigation work out of scope.
- 15.8 fix-1:
  - fixed the follow-up layout regression where section header labels became centered after that hotspot narrowing;
  - restored explicit `Text` geometry plus `horizontal-alignment:left` inside the narrowed header label lane instead of changing any menu/controller boundary;
  - kept `SelectableSectionText` native text-surface right-click, header/chrome window-local menu coverage, and inline `Copy` action behavior unchanged.

## Dependency upgrade roadmap status (Phase 15.3A -> 15.8 fix-1 completed)

- Why this upgrade line was executed:
  - a meaningful share of deferred UI work was blocked by the old dependency baseline rather than by product uncertainty;
  - `15.2E` remained the clearest example because editable-input menus were too expensive/risky on `slint = 1.8.0`.
- Current upgraded baseline:
  - `15.2D` remains the stable UI contract;
  - `15.2E` is now shipped on the upgraded baseline;
  - workspace `edition` remains `2021`;
  - Rust toolchain is fixed at `1.94.0`;
  - workspace `rust-version = 1.94`;
  - `slint` / `slint-build` are pinned to `1.15.1`;
  - `fc-ui-slint` still uses inline `slint::slint!`;
  - release version ownership now lives in the workspace manifest and packaging derives bundle/DMG/ZIP version from that source;
  - main UI sync no longer relies on repeated `50ms` polling; async completion is pushed back to the UI thread on demand;
  - loading-mask timeout copy is phase-driven UI-local timer behavior only;
  - results/diff models now reuse persistent `VecModel` instances;
  - non-input context-menu visual layer now uses the accepted `15.7` polished panel/item treatment while keeping the same controller ownership and menu scope;
  - Analysis success `SelectableSectionText` now keeps selection + system-copy baseline and also exposes native text-surface `Copy` / `Select All` right-click on the selected text only;
  - Analysis success section header labels remain explicitly left-aligned inside the narrowed header label lane after `Phase 15.8 fix-1`;
  - macOS arm64 remains the primary validation platform.
- Completed phase train:
  - `Phase 15.3A`: version-source cleanup plus doc/checklist alignment, completed;
  - `Phase 15.3B`: Rust `1.94.0` migration with no `15.2D` regression, completed;
  - `Phase 15.4`: Slint `1.15.1` migration with behavior parity restored, completed;
  - `Phase 15.5`: editable-input context-menu integration shipped with conservative `API Key` secret handling, completed;
  - `Phase 15.5 fix-2`: read-only selectable-content typography cleanup moved from window/panel prop threading to a shared Slint global token, completed;
  - `Phase 15.5 fix-3`: diff detail horizontal scrollbar regression moved off the upgraded `ListView` path onto an explicit `ScrollView` viewport with bottom safe spacer, completed;
  - `Phase 15.6`: sync/model-churn cleanup shipped without expanding product scope, completed;
  - `Phase 15.7`: non-input context-menu visual polish shipped as a style-only pass on top of the existing window-local controller, completed;
  - `Phase 15.8`: Analysis success `SelectableSectionText` native text-surface right-click shipped without reopening menu-controller scope, completed;
  - `Phase 15.8 fix-1`: Analysis success section header left-alignment regression fixed without changing the `15.8` menu scope, completed.
- What intentionally still did not change after `15.8 fix-1`:
  - no `Phase 16` navigation work was mixed into the menu polish pass;
  - `Search` keeps its explicit `Clear` button because the current native `cupertino` style still lacks a stable clear affordance;
  - the large inline `slint::slint!` surface was not externalized because the cleanup benefit was still below migration cost;
  - no `edition = "2024"` pass was combined with the dependency diff;
  - no native platform menu bridge or new menu controller was introduced for `15.7` / `15.8` / `15.8 fix-1`;
  - `Phase 15.8` remains text-body-native-only scope and `15.8 fix-1` is not a reason to route `SelectableSectionText` back into the window-local menu core;
  - `SelectableDiffText` row-level context-menu coverage remains deferred.
- Remaining phase train:
  - `Phase 16` returns to mainline progression and is no longer part of this dependency-upgrade phase train.
- Detailed implementation planning now lives in:
  - `docs/upgrade-plan-rust-1.94-slint-1.15.md`

## Deferred architecture decisions (after Phase 15.8 fix-1 shipped)

- `P1` Secure secret storage integration (Keychain/Credential Manager/Secret Service):
  - trigger: before remote provider is treated as production-default.
- `P1` Provider profile management (multiple saved provider presets):
  - trigger: when teams need rapid context/provider switching.
- `P2` Response caching + token/cost tracking:
  - trigger: when remote analysis usage and cost visibility become operational concerns.
- `P2` Multi-provider plugin orchestration:
  - trigger: when provider fallback/routing becomes a reliability requirement.
- `P2` Multi-line copy workflow:
  - deferred because the current baseline now covers low-noise Diff row copy plus lightweight Analysis section/whole-review copy; full range selection, clipboard formatting, and richer clipboard semantics would still expand interaction scope beyond the accepted shell.
- `P3` Optional Workspace Diff detail line selectable-row context-menu (`SelectableDiffText`):
  - deferred and explicitly optional for now; current baseline keeps keyboard copy plus line-number/hunk double-click copy as the accepted low-risk path;
  - trigger: only revisit if mouse-driven row copy becomes a demonstrated productivity gap that existing selection + copy shortcuts do not cover;
  - do not mix with accepted `Phase 15.7` style-only polish or `Phase 16` navigation work, and do not revisit with overlay interception, private pointer plumbing, or custom caret/selection logic.
- `P2` Search clear-affordance convergence:
  - deferred because the current macOS native `cupertino` `LineEdit` style still lacks a stable built-in clear action, so the accepted `15.5` baseline keeps an explicit `Clear` button;
  - trigger: revisit only if Slint native style support changes or the application deliberately adopts a different desktop widget style.
- `P2` Analysis shell-state selectable text (non-success states):
  - not a hard requirement for `Phase 15.1B fix-3`;
  - deferred for a separate pass so shell-state interaction changes do not regress the stabilized success-body scrolling contract.
- `P2` Analysis streaming raw-response presentation with loading mask:
  - deferred to `Phase 19: AI analysis enhancement`;
  - local loading-mask baseline is already available in `15.2B`; deferred part is streaming raw-response orchestration and final structured hydration flow.
- `P2` Loading-mask timeout policy configuration:
  - local baseline currently uses a fixed UI-local timeout constant for copy downgrade only;
  - deferred because operation-level timeout customization would require extra presenter/open API surface and is not needed for current accepted scope.
- `P2` Global loading orchestration:
  - local baseline now exists via window-local loading-mask scope derivation (`running/diff_loading/analysis_loading`);
  - global route/cross-window loading coordination remains deferred until broader multi-surface workflows require a shared loading controller model.
- `P2` Global toast / feedback orchestration:
  - local baseline now exists via window-local `toast-controller` (overlay toast, tone/queueing/replace policy, per-request duration);
  - global routing, persistence, and cross-surface orchestration remain deferred until broader save/export/report flows require a notification center model.
- `P2` Sticky left-side line numbers:
  - deferred because the current `ScrollView`-backed diff viewer would still need a split pinned gutter + horizontally scrollable content lane with synchronized vertical viewport, row-height parity, and hunk-row handling; that is medium-high complexity and too invasive for the accepted `15.5 fix-3` stabilization shell.
- `P3` Tree explorer / compare-view dual-mode workspace:
  - trigger: when file-view-only navigation becomes a productivity bottleneck.

## Next implementation priority (after Phase 15.8 fix-1 shipped)

1. Resume `Phase 16` results navigation enhancement in mainline mode.
   - acceptance: users can locate one target file in large result sets with fewer manual scroll steps and without introducing tree mode or regressing the accepted workspace shell.
2. Keep `Phase 15.8` narrowness intact while future work lands around it.
   - acceptance: Analysis success body text stays on the shipped native text surface path, section header/chrome stays on the window-local menu path, section labels stay left-aligned inside the narrowed header lane, `Risk Level` remains explicit `Copy` button-only, and no broader selectable-text menu scope is reintroduced accidentally.
3. Revisit `.slint` externalization only when it delivers concrete maintainability or compile-time payoff beyond the now-accepted inline baseline.
   - acceptance: do not reopen this just for cleanup aesthetics; any future extraction must justify the new `slint-build` churn and preserve the current shell/runtime contracts.

## Documentation update contract (mandatory)

### Update triggers

Update this file in the same PR whenever any of the following changes:

- UI IA or workspace layout contract;
- Diff/Analysis state machine or preview eligibility contract;
- provider settings boundary/persistence model;
- runtime synchronization strategy that can affect responsiveness/stability;
- deferred architecture decisions or priority order.
- language/terminology policy that affects cross-thread handoff docs.

### Required sections to touch per trigger

- Always update `fc-ui-slint current architecture snapshot`.
- Update `evolution highlights` only when the change introduces a new architectural step.
- Update `deferred architecture decisions` when priority/trigger/status changes.
- Update `next implementation priority` when acceptance targets change.

### Writing rules

- Record architecture facts and boundaries, not implementation diary details.
- Language policy: keep this file in English; keep `docs/thread-context.md` in Chinese with key English terms preserved.
- For shared contracts, use the same canonical terms across this file and `docs/thread-context.md`.
- Each update must state:
  - what changed;
  - why the boundary/contract changed;
  - what intentionally did not change.
- Keep incremental updates concise (target: 8-20 lines per PR unless major refactor).

### Review checklist (Definition of Done)

- The changed code paths and this document describe the same contracts.
- New behavior is reflected in `current architecture snapshot`.
- Deferred/priority items remain ordered and have explicit triggers.
- No obsolete phase wording remains after contract changes.
