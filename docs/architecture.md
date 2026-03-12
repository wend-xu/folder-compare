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

## Phase 13.1 -> 14.2 fix-3 architecture evolution (from git log)

Commit range:

- start: `8a77ef8149eb820411427fe9380bbbde40dc8509` (`phase 13.1`)
- end: `2a7cfc0db2f3bd215dee0f86666f99e90aa048f8` (`Phase 14.2 fix-3 输入行布局稳定性与 modal 节奏统一`)

Tracked phase commits in this range:

- `8a77ef8` `phase 13.1`
- `6d2ec35` `Phase 13.1 fix-1: 宽度异常 + 状态区轻收口`
- `dd0969e` `phase 14 Provider Settings 与配置持久化`
- `13f0a8c` `Phase 14.1：Compare Inputs / Filter 轻交互微调`
- `8f161bc` `Phase 14.2：视觉目标图 + 设计语言收敛`
- `058fbd3` `Phase 14.2 fix-1：视觉实现收敛`
- `22a558c` `Phase 14.2 fix-2：组件对齐与状态控件收敛`
- `2a7cfc0` `Phase 14.2 fix-3 输入行布局稳定性与 modal 节奏统一`

### `fc-ui-slint` architecture maturity after Phase 14.2

- Window information architecture is now stable:
  - lightweight `App Bar` as global entry layer;
  - fixed `Sidebar` with four sections:
    - `Compare Inputs`
    - `Compare Status`
    - `Filter / Scope`
    - `Results / Navigator`
  - `Workspace` for file view with `Diff / Analysis` tabs, mode header, and content panel.
- Sidebar sizing and width behavior is hardened:
  - fixed sidebar width contract with stable left/right stretch behavior;
  - long text uses elide/wrapping constraints to avoid intrinsic-width pushback.
- Compare input architecture is now complete:
  - left/right path input + native folder browse flow;
  - compare action with clearer empty-path behavior;
  - basic path validation before compare (`empty`, `missing`, `not-directory`, `unreadable`).
- Filter architecture is now state-driven and stable:
  - search + segmented status scope (`All / Diff / Equal / Left / Right`);
  - active scope hint is retained as a weak signal;
  - segmented visual state now updates in lockstep with actual filter state.
- Compare status architecture has been reduced to summary-first:
  - primary status + compact badges;
  - key metrics line (`total/changed/left/right`);
  - details downgraded to weak expandable preview rather than embedded heavy details view.

### Provider settings architecture after Phase 14

- Provider configuration has moved from Analysis content into global modal flow:
  - entry: App Bar `Provider Settings`;
  - edit boundary: modal with explicit `Save / Cancel`;
  - Analysis panel no longer hosts full provider form.
- Persistence architecture is established in UI layer:
  - `settings.rs` owns load/save for provider settings;
  - startup loads persisted provider config in presenter `Initialize`;
  - save path writes local `provider_settings.toml`;
  - supports `FOLDER_COMPARE_CONFIG_DIR` override and OS-specific default config dirs.
- Provider model now includes:
  - `Mock` / `OpenAI-compatible` mode;
  - endpoint / api key / model / timeout;
  - API key password input semantics with show/hide toggle in modal UI.

### UI component-system direction after Phase 14.2

- A lightweight reusable UI kit is now present in Slint layer:
  - `SectionCard`, `ToolButton`, `SegmentedRail`, `SegmentItem`, `StatusPill`, `TextAction`.
- Visual iterations in `14.2 fix-1/2/3` focused on convergence:
  - reduce over-styled controls and heavy borders;
  - tighten radii and spacing toward desktop-tool density;
  - stabilize input-row alignment and modal rhythm across provider modes.

## Phase 15.0 -> 15.0 fix-5 architecture evolution (from git log + session trace)

Commit range:

- start: `57e6e6326bc9b23a2e626eb16df0c4db9980afb6` (`Phase 15.0：语义配色与 Workspace 一体化起步`)
- end: `3a723c473e033efdd6046a0127e07a60417c2b0a` (`Phase 15.0 fix-4 ... + Phase 15.0 fix-5 ...`)

Tracked phase commits in this range:

- `57e6e63` `Phase 15.0：语义配色与 Workspace 一体化起步`
- `2b60af3` `Phase 15.0 fix-1：Workspace 布局修复与语义配色校正`
- `9fdd3e3` `Phase 15.0 fix-2 Results 语义色收敛 + File View 行为补完 + Analysis 成功态统一`
- `83d72c8` `Phase 15.0 fix-3：Results 语义色定稿 + 预览模式收敛 + 性能回退排查`
- `3a723c4` `Phase 15.0 fix-4 状态色定稿 + equal 预览打通 + 滚动卡顿归因 Phase 15.0 fix-5 Results 状态色定稿 + 滚动卡顿进一步归因`

### `fc-ui-slint` architecture maturity after Phase 15.0 fix-5

- Workspace shell is now structurally unified while preserving IA:
  - `Tabs -> Header -> Content` is kept as one continuous workspace flow;
  - Diff/Analysis mode branches are conditionally rendered to avoid dual-branch layout contention;
  - main content area consistently consumes remaining height across empty/loading/error/success states.
- File View state machine is explicit and shared:
  - Diff mode states are normalized as `no selection -> loading -> unavailable -> failed -> renderable`;
  - Analysis mode states are normalized as `no selection -> not started -> loading -> failed -> result`;
  - `WorkspaceStatePanel` is used as the common visual/state container.
- Results/Navigator semantics are now contract-level, not ad-hoc styling:
  - semantic status coloring is fixed for `different`, `equal`, `left-only`, `right-only`;
  - interaction blue is reserved for selected/focus/active only;
  - unavailable rows are treated as a separate neutral state (not blue-gray), with downgraded text hierarchy.
- Single-side preview pipeline is unified and promoted to first-class capability:
  - `left-only`, `right-only`, and `equal` rows can all enter File View preview path;
  - equal preview is no longer blocked by detailed-diff eligibility checks;
  - preview rows are rendered with neutral context styling plus side-aware line numbers.
- Diff/analysis orchestration behavior is clearer:
  - `can_load_diff = false` is represented as explicit `unavailable` rather than generic load failure;
  - repeated click on the same already-loaded/loading row is short-circuited to avoid duplicate load chains.
- UI-side refresh and scrolling safeguards were added incrementally:
  - compare/diff list models rebuild only when relevant source state changes;
  - timer refresh remains polling-based but is constrained to busy-state transitions, reducing unnecessary full snapshots;
  - row delegate state is localized to reduce repeated indexed binding evaluation;
  - lightweight header stats computation avoids cloning filtered rows only for count purposes.

### Scope and boundaries kept during Phase 15.0

- No IA reset: Sidebar remains `Compare Inputs / Compare Status / Filter / Scope / Results / Navigator`.
- No Compare View/tree-mode expansion in this phase.
- No AI schema/provider capability expansion beyond UI integration and state-flow unification.
- No deep expansion of Compare Status details; it stays summary-first.

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
