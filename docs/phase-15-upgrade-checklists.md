# Phase 15 Upgrade Checklists

## Version Source

- Release version single source of truth: workspace `[workspace.package].version` in `Cargo.toml`
- `fc-core` / `fc-ai` / `fc-ui-slint` inherit the same crate version from the workspace manifest
- macOS bundle `CFBundleVersion` / `CFBundleShortVersionString` are generated from the same manifest version by `docs/macos_dmg.sh`
- DMG and ZIP artifact names are generated from the same manifest version by `docs/macos_dmg.sh`

## Phase Cutline

- `Phase 15.3A`
  - do not change Rust or Slint dependency versions
  - do not change `15.2D` product behavior
  - exit only after version ownership and upgrade smoke inputs are explicit
- `Phase 15.3B`
  - lock toolchain to Rust `1.94.0`
  - raise workspace `rust-version` to `1.94`
  - keep `slint = 1.8.0`
- `Phase 15.4`
  - migrate `slint` / `slint-build` to `1.15.1`
  - restore `15.2D` behavior parity before reopening any deferred input-menu scope

## Upgrade Checklist

- Confirm `Cargo.toml` owns the release version and packaging scripts do not hardcode another version
- Confirm `rust-toolchain.toml` and workspace `rust-version` match the intended Rust baseline
- Confirm `slint` and `slint-build` stay on the same exact patch version
- Run `cargo check --workspace`
- Run `cargo test --workspace`
- Run `cargo run -p fc-ui-slint`
- Record any compile warnings introduced by the new toolchain or Slint migration before leaving the phase

## Smoke Checklist

- Compare flow remains usable: input left/right paths, browse, validate, and run compare
- Results row selection still drives the active Diff context without selection drift
- Navigator / Results selection still closes the non-input context menu correctly
- Diff and Analysis tabs keep the connected workbench shell and seam continuity
- Analysis success scroll still auto-closes the non-input context menu
- Loading mask still stays out of the App Bar and Provider Settings modal
- Provider Settings can still save and reload configuration
- `Risk Level` still uses the explicit `Copy` button only
- Non-input context menu still exposes only the current safe-surface actions
