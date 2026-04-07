# Folder Compare Testing Guidelines

## Purpose

- This document defines the current testing practices for the repository.
- It is intentionally practical and repository-specific.
- It focuses on test placement, fixture construction, and source-of-truth discipline.
- It is not a generic Rust testing style guide.

## Current Testing Layout

- Keep narrow unit tests inline in the module that owns the logic.
- Keep cross-module contract tests in independent files under `crates/fc-ui-slint/src/tests/`.
- Use shared fixture builders when multiple test files need the same compare-state setup.
- Do not move internal `fc-ui-slint` contract tests into Cargo integration tests by default.
  - `fc-ui-slint` is currently a binary crate.
  - Many important tests still need access to crate-internal types and private helpers.
  - `src/tests/` keeps those tests organized without forcing visibility widening.

## Source-of-Truth Rule

- Tests must construct data from the current architectural source of truth.
- For compare-workspace, navigator, and selection semantics, the default source of truth is:
  - `fc_core::CompareEntry`
  - then `CompareFoundation`
  - then migration-era projections such as legacy `entry_rows`
- Do not build new tests around obsolete shortcuts just because they are convenient.
- If production behavior is foundation-first, the test fixture should also be foundation-first.

## Inline Module Tests

Keep tests inline when all of the following are true:

- The test validates logic that is local to one module.
- The test does not depend on a shared scenario builder.
- The test does not express a multi-step product contract.
- The test is still easy to read next to the implementation.
- Moving it out would add indirection without adding structure.

Typical inline examples in this repository:

- small helper/state-machine tests in `app.rs`
- local queue/replace behavior in `toast_controller.rs`
- persistence-local behavior in `settings.rs`
- internal structural invariants in `compare_foundation.rs`

## Independent Test Files

Move tests to independent files under `crates/fc-ui-slint/src/tests/` when any of the following is true:

- The test verifies a stable product contract rather than a small helper.
- The test crosses module boundaries such as `bridge -> foundation -> state -> presenter`.
- The test needs reusable fixtures or scenario builders.
- The implementation file is starting to accumulate large scenario tests.
- The test should be read as behavior documentation for a subsystem.
- The test must be constructed from the current source-of-truth path and that setup is non-trivial.

Typical independent examples in this repository:

- navigator projection and reveal behavior
- state-level filter, membership, and scroll semantics
- presenter-level mode/selection/focus transitions
- bridge-level compare mapping that now has both foundation and legacy projection responsibilities

## Fixture Rules

- Put shared fixture builders in `crates/fc-ui-slint/src/tests/fixtures.rs`.
- Shared fixtures should produce canonical setup with minimal ceremony.
- Prefer fixture builders that start from `CompareEntry` values.
- Build `CompareFoundation` from those entries, then project legacy rows only when needed by the current file-view pipeline.
- Keep fixture helpers small and composable.
- Do not hide important behavioral assumptions inside giant fixture constructors.

For current compare-related tests:

- Prefer `CompareEntry -> CompareFoundation -> AppState`.
- Use direct `CompareFoundation` fixtures when testing tree or compare-target semantics.
- Use direct legacy `CompareEntryRowViewModel` fixtures only when the test is explicitly about:
  - migration compatibility
  - legacy file-view bridge behavior
  - widget/view-model formatting that still legitimately depends on row projections

## Legacy Compatibility Tests

- Legacy projection tests are still allowed during the migration period.
- They must be clearly scoped as compatibility tests, not canonical-state tests.
- They should not redefine the primary mental model of the subsystem.
- Do not add new long-lived behavior tests that depend on test-only fallback reconstruction from `entry_rows`.
- If a fallback exists only to keep old tests alive, treat it as migration debt and reduce reliance on it over time.

## Naming and Organization

- Name independent test files by subsystem or contract domain, not by helper function.
- Good names describe architectural scope:
  - `state_foundation_tests.rs`
  - `presenter_foundation_tests.rs`
  - `navigator_tree_tests.rs`
  - `bridge_tests.rs`
- Use `legacy` or `compat` in the test name when the test is intentionally validating a migration-era compatibility layer.
- Keep each file cohesive; do not turn `src/tests/` into a catch-all dump.

## Review Checklist

Before adding or moving a test, check the following:

- Does the fixture start from the current source of truth?
- Is the test validating a local implementation detail or a subsystem contract?
- Will this test likely need shared fixtures with neighboring tests?
- Is the implementation file becoming harder to read because of scenario-heavy tests?
- Is the test accidentally relying on a test-only fallback that production does not use?
- If this is a compatibility test, is that made explicit in the name and assertions?

## Current Examples

- Keep inline:
  - `crates/fc-ui-slint/src/app.rs`
  - `crates/fc-ui-slint/src/settings.rs`
  - `crates/fc-ui-slint/src/toast_controller.rs`
  - `crates/fc-ui-slint/src/compare_foundation.rs`
- Prefer independent files:
  - `crates/fc-ui-slint/src/tests/navigator_tree_tests.rs`
  - `crates/fc-ui-slint/src/tests/state_foundation_tests.rs`
  - `crates/fc-ui-slint/src/tests/presenter_foundation_tests.rs`
  - `crates/fc-ui-slint/src/tests/bridge_tests.rs`

## Update Contract

- Update this guide when test placement policy materially changes.
- Update this guide when the architectural source of truth changes for workspace compare state.
- Update this guide when `fc-ui-slint` stops being a binary crate and the recommended test boundary changes.
