# Folder Compare Architecture (Phase 1-15.0 fix-5)

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

## `fc-ui-slint` interaction/layout maturity after Phase 12

- Main window now supports:
  - left/right directory path input;
  - compare trigger button;
  - compare status, summary, warning, and error display;
  - compact compare summary/warning/error/truncated area with lower vertical overhead;
  - scrollable compare result list with stable row selection state;
  - lightweight compare row filtering by path/detail text;
  - structured list row rendering (`status`, `relative_path`, `detail`) instead of one raw line;
  - detailed diff panel driven by `fc-core::diff_text_file`, including:
    - selected path display;
    - scrollable unified-style viewer rows (`old_line_no/new_line_no/marker/content`);
    - visual separation for hunk headers and added/removed/context rows;
    - diff warning and diff truncated semantics;
    - diff-level error display isolated from compare-level errors.
- AI analysis panel now runs through `fc-ai` mock pipeline:
  - `Analyze` action from selected diff row;
  - bridge mapping from selected row + detailed diff to `AnalyzeDiffRequest` (`RiskReview`);
  - `Analyzer + MockAiProvider` invocation in presenter command flow;
  - analysis result card fields (`title`, `risk_level`, `rationale`, `key_points`, `review_suggestions`);
  - independent analysis loading/error/result state, separated from compare/diff states.
- AI provider mode and remote config are now exposed in UI with minimal settings:
  - provider mode switch (`Mock` / `OpenAI-compatible`);
  - OpenAI-compatible inputs (`endpoint`, `api key`, `model`);
  - analysis command dispatch chooses provider by `AiConfig.provider_kind`;
  - remote mode warns that diff excerpt is sent to configured endpoint;
  - remote mode requires complete config before analysis can start.
- Text/layout stability improvements for large-directory output:
  - compare warnings now use wrapped text with UI-side line splitting and a scrollable warning block to avoid overflow beyond container bounds;
  - selected path display now uses safe middle-ellipsis abbreviation for very long values;
  - result list `path/detail` lines use safe elide to avoid row layout breakage.
- Right-side details area now follows a clearer Phase 11-ready hierarchy:
  - selected path;
  - diff summary;
  - diff status block (loading/warning/truncated/error);
  - AI analysis panel (mock provider) between status and diff viewer;
  - detailed unified diff viewer as the primary lower section.
- Interaction/runtime improvements:
  - compare and detailed diff execution moved to background worker threads with a short startup defer so loading state can render first;
  - periodic UI snapshot refresh keeps view state synchronized while background work is running;
  - timer-driven refresh now uses a passive sync mode that updates display-only state and does not overwrite editable inputs (`left_root/right_root/filter`) while typing;
  - full input synchronization is limited to initialization/explicit submission paths, preventing cursor/content reset during user editing;
  - passive refresh now applies change-detection before syncing UI, so unchanged snapshots do not rebuild list models each timer tick;
  - detailed diff list no longer gets repeatedly rebound during idle timer cycles, preventing scroll position from being dragged back to top after release;
  - compare list and detailed diff panel now have independent scroll areas in a split layout;
  - window uses preferred/min size constraints and stretches key regions with resize.
- UI orchestration boundaries:
  - `commands`: user actions and compare execution trigger;
  - `presenter`: compare workflow plus filtering and selected-row detailed diff orchestration (including background task state transitions);
  - `bridge`: request construction and `CompareReport`/`TextDiffResult` to UI view-model mapping;
  - `state/view_models`: lightweight, UI-facing data and filter/viewer projection helpers.
- `fc-core` integration now includes:
  - `compare_dirs` for summary list;
  - `diff_text_file` for selected-row detailed diff.

## Architecture evolution (Phase 13.1 -> 15.0 fix-5)

This period moved `fc-ui-slint` from “feature accumulation” to a contract-driven desktop workspace architecture.  
The same IA was preserved end-to-end, but each layer (layout, state flow, semantics, preview behavior, runtime sync) was progressively hardened.

### 1) Information architecture and layout contract

- IA stayed stable as:
  - `App Bar`;
  - `Sidebar` (`Compare Inputs / Compare Status / Filter / Scope / Results / Navigator`);
  - `Workspace` (`Diff / Analysis` tabs, header, content).
- Layout behavior was tightened in two steps:
  - 14.x established sidebar width/overflow discipline and desktop-density component rhythm;
  - 15.0 unified Workspace into one continuous shell (`Tabs -> Header -> Content`) with conditional mode branches so only one mode participates in layout at a time.
- Result: main work area keeps stable height allocation across empty/loading/error/success states instead of fragmenting into independent boxes.

### 2) Input/filter/status workflow contract

- Compare entry workflow is complete and state-driven:
  - left/right path input + native browse + validation (`empty/missing/not-directory/unreadable`);
  - search + segmented status scope (`All / Diff / Equal / Left / Right`) with lockstep visual state.
- Compare Status is intentionally summary-first:
  - primary status + compact pills + key metrics;
  - details are weak/expandable instead of becoming a second heavy details pane.

### 3) Provider settings and persistence contract

- Provider configuration moved to a global modal boundary:
  - entry from `App Bar -> Provider Settings`;
  - explicit `Save / Cancel` lifecycle;
  - Analysis content no longer embeds full provider form.
- Persistence contract is localized in UI layer:
  - `settings.rs` owns load/save;
  - initialize-time hydration in presenter;
  - file-based persistence (`provider_settings.toml`) with `FOLDER_COMPARE_CONFIG_DIR` override.
- Provider config model is now complete for current scope:
  - `Mock` / `OpenAI-compatible`;
  - endpoint / api key / model / timeout;
  - API key visibility toggle semantics.

### 4) File View state and preview contract

- File View now follows one explicit state machine:
  - Diff: `no selection -> loading -> unavailable -> failed -> renderable`;
  - Analysis: `no selection -> not started -> loading -> failed -> result`.
- `WorkspaceStatePanel` became the shared state container so state transitions no longer change container grammar.
- Single-side preview is first-class:
  - `left-only`, `right-only`, and `equal` all enter File View preview path;
  - equal preview is not blocked by detailed-diff eligibility;
  - preview rows are rendered as neutral context with side-aware line numbering.
- Diff load behavior is clarified:
  - `can_load_diff = false` maps to explicit `unavailable` state;
  - repeated click on the same loaded/loading row is short-circuited.

### 5) Semantic and runtime synchronization contract

- Semantic color system is now role-based:
  - `different/equal/left-only/right-only` carry status semantics;
  - blue is reserved for selected/focus/active interaction;
  - unavailable uses neutral hierarchy (not blue-gray).
- Runtime sync path was incrementally hardened for scroll/interaction stability:
  - list models rebuild only on relevant source-state changes;
  - timer refresh stays polling-based but is constrained by busy-state transitions;
  - row delegate state is localized to reduce repeated indexed binding evaluation;
  - header stats avoid cloning filtered rows only to compute counts.

### 6) Guardrails kept through this evolution

- No IA reset.
- No Compare View/tree-mode expansion.
- No AI schema/provider capability expansion beyond UI-state orchestration.
- No deep Compare Status details expansion beyond summary-first intent.

## Still deferred after Phase 15.0 fix-5

- secure secret storage integration (Keychain/Credential Manager/Secret Service) is not implemented.
- provider profile management (multiple saved provider presets) is not implemented.
- response caching and token/cost tracking are not implemented.
- multi-provider plugin orchestration is not implemented.
- tree-based directory explorer and compare-view dual-mode workspace are not implemented.

## Next implementation priority

Post-15.0 priority should move to Phase 15.1 depth without breaking current IA:

1. refine File View information rhythm (`path/summary/state pills/content`) and visual stability under all state transitions;
2. improve results navigation efficiency (sorting/quick-jump/filter ergonomics) without introducing tree compare mode yet;
3. keep analysis panel in unified workspace rhythm while improving result readability/layout density;
4. continue provider hardening roadmap (secure secrets, diagnostics, reliability controls) outside core compare logic.
