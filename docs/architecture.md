# Folder Compare Architecture (Phase 1-15.1B)

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

## `fc-ui-slint` current architecture snapshot (Phase 15.1B)

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
  - compact meta row: `Analysis` label + state badge + weak provider-readiness badge + one-line summary;
  - helper strip keeps provider/readiness/timeout/truncation as weak technical context rather than the primary message.
- Analysis success content is now a structured review-conclusion panel instead of raw text stacking:
  - `Summary`;
  - `Risk Level`;
  - `Core Judgment`;
  - `Key Points`;
  - `Review Suggestions`;
  - `Notes` for truncation/warning/mock-provider caveats.
- Analysis non-success states now reuse the same embedded `DiffStateShell` visual grammar as Diff, so shell hierarchy stays consistent without reintroducing detached cards.
- Workspace shell visual grammar is now part of the contract: tabs have no inter-tab gap, selected tab fill aligns with header surfaces, lower tab edges are straight, and tab/panel seam lines must remain continuous without detached pill styling.
- Neutral `no-selection` shell indicators stay within the cool workspace palette rather than using a warm generic neutral badge.
- Single-side preview remains first-class:
  - `left-only`, `right-only`, and `equal` all enter preview path;
  - equal preview is not blocked by detailed-diff eligibility;
  - preview table columns are side-aware (`left/right`) instead of always `old/new`.
- Diff content keeps the `column header -> rows` structure but now guarantees baseline review ergonomics:
  - line content is selectable/copyable;
  - the workbench panel owns `header -> helper strip -> state/table` as one surface, so Diff detail reads as a single workstation instead of nested cards;
  - the table can scroll horizontally for long lines with persistent in-surface guidance;
  - the diff body `ListView` owns vertical scrolling while the column header mirrors its horizontal viewport, keeping the vertical scrollbar visible without depending on horizontal position;
  - the diff body reserves a scrollbar-safe bottom inset so the last rows stay selectable/copyable;
  - lightweight row-copy fallback moved from always-visible per-row buttons to double-click-on-line-number/hunk-marker hotspots that copy the full underlying line text, not just the visible viewport fragment;
  - transient copy feedback is explicit but restrained (`Line N copied` / `Copy failed`);
  - fixed left-side line numbers remain deferred in this phase.
- `can_load_diff = false` and preview capability boundaries map to explicit `unavailable` (not generic failure).
- No new AI schema, sidebar IA, compare mode, or task orchestration layer was added for `15.1B`; the productization remains presentation/state-derivation only.

### Provider settings and persistence contract

- Provider settings are edited in a global modal (`App Bar -> Provider Settings`), not embedded in Analysis content.
- Persistence is owned by `settings.rs`, loaded at presenter initialize-time, and written to `provider_settings.toml` (with `FOLDER_COMPARE_CONFIG_DIR` override).
- Provider config model in scope:
  - `Mock` / `OpenAI-compatible`;
  - endpoint / api key / model / timeout;
  - API key visibility toggle semantics.

### Runtime synchronization contract

- Compare/diff/analysis work runs in background workers; UI uses periodic snapshot synchronization while work is active.
- Rebinding and refresh are bounded:
  - result/diff models rebuild only when relevant source state changes;
  - timer refresh is constrained by busy-state transitions;
  - row delegate local state reduces repeated indexed binding evaluation;
  - header stats avoid cloning filtered rows only for count computation.

### Boundaries and non-goals in this phase

- No IA reset.
- No Compare View/tree-mode expansion.
- No AI schema/provider capability expansion beyond UI-state orchestration.
- No deep Compare Status details expansion beyond summary-first intent.

## `fc-ui-slint` evolution highlights (Phase 13.1 -> 15.1B)

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
  - kept Diff shell, sidebar IA, and AI response schema unchanged.

## Deferred architecture decisions (after Phase 15.1B)

- `P1` Secure secret storage integration (Keychain/Credential Manager/Secret Service):
  - trigger: before remote provider is treated as production-default.
- `P1` Provider profile management (multiple saved provider presets):
  - trigger: when teams need rapid context/provider switching.
- `P2` Response caching + token/cost tracking:
  - trigger: when remote analysis usage and cost visibility become operational concerns.
- `P2` Multi-provider plugin orchestration:
  - trigger: when provider fallback/routing becomes a reliability requirement.
- `P2` Multi-line copy workflow:
  - deferred because the current fix-3 stabilization pass only needed a low-noise safety net for single-row extraction; full range copy, clipboard formatting, and selection semantics would expand Diff interaction scope beyond the current baseline.
- `P2` Sticky left-side line numbers:
  - deferred because the current Slint `ListView` viewer would need a split pinned gutter + horizontally scrollable content lane with synchronized vertical viewport, row-height parity, and hunk-row handling; that is medium-high complexity and too invasive for the accepted fix-3 stabilization shell.
- `P3` Tree explorer / compare-view dual-mode workspace:
  - trigger: when file-view-only navigation becomes a productivity bottleneck.

## Next implementation priority (after Phase 15.1B first-round productization)

1. Improve results navigation efficiency (sorting/quick-jump/filter ergonomics) without introducing tree mode.
   - acceptance: users can locate one target file in large result sets with fewer manual scroll steps.
2. Carry forward provider hardening deferred items only within the existing boundaries.
   - acceptance: reliability hardening does not expand IA, schema, or workspace mode count ahead of need.
3. Only do lightweight Analysis polish if runtime smoke exposes density or geometry regressions.
   - acceptance: fixes stay within the current shell contract and do not reopen Diff-shell or IA redesign.

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
