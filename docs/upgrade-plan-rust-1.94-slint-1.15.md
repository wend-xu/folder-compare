# Folder Compare Upgrade Archive (`rust 1.94.0` / `slint 1.15.1` / `edition 2024`)

## 1. Purpose and status

本文件现在是归档背景，不再是当前主线执行计划。

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
- 独立 workspace `edition = "2024"` 里程碑

以上事项均已完成，并已被 `phase15 summary` 收束进当前架构基线。当前主线的下一步是 `Phase 16`，不要再把本文件当作待执行 roadmap。

## 2. Final shipped baseline

- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`
- release version、bundle version、DMG / ZIP version 统一从 workspace manifest 派生
- `15.2E` 已在该基线上落地
- event-driven sync、persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、read-only selectable content 继续走 Slint 默认 generic family（由现有 macOS bootstrap 接入系统字体）、non-input context-menu visual polish、`Analysis success` native text-surface right-click、section header 左对齐修复均已是稳定事实

## 3. Archived outcomes by phase

- `Phase 15.3A`
  - 统一版本号单一事实来源到 workspace manifest，并让 `docs/macos_dmg.sh` 从 manifest 派生 bundle / DMG / ZIP 版本。
- `Phase 15.3B`
  - 锁定 `rust-toolchain = 1.94.0`，提升 workspace `rust-version = 1.94`，验证 `15.2D` 行为不回退。
- `Phase 15.4`
  - 升级到 `slint = 1.15.1` / `slint-build = 1.15.1`，保持既有 shell / menu / loading / toast 边界不变。
- `Phase 15.5`
  - `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框切到 `slint 1.15.1` 原生 editable-input context menu。
  - `Provider Settings -> API Key` 落地专用 `ApiKeyLineEdit`，继续保持 hidden=`Paste` only、visible=`Select All`/`Copy`/`Paste`/`Cut`。
- `Phase 15.5 fix-1`
  - 修复 `SelectableDiffText` / `SelectableSectionText` 在 mixed Latin/CJK 文本中的 glyph fallback 回归。
- `Phase 15.5 fix-2`
  - 曾把 glyph fallback 收敛为共享 typography 中转层，避免继续通过多层 prop threading 传递。
- `Phase 15.5 fix-3`
  - `Diff` detail 横向滚动改到显式 `ScrollView` 视口，并保留 content-end scrollbar-safe spacer。
- `Phase 15.6`
  - UI 主同步路径切到 event-driven sync，loading-mask timeout copy 切到按 busy phase 调度的一次性 timer，`Results / Navigator` 与 `Diff` 行模型切到 persistent `VecModel`。
  - 评估后继续保留内联 `slint::slint!`，没有外置 `.slint`。
- `Phase 15.7`
  - non-input context-menu 只做 visual polish，不改变 controller ownership、action dispatch、safe-surface coverage。
- `Phase 15.8`
  - `Analysis success` 的 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 正文文本切到 Slint native text surface 的 `Copy` / `Select All` right-click。
  - section header / chrome 继续走 window-local non-input context-menu core；`Risk Level` 继续保持显式 `Copy` 按钮-only。
- `Phase 15.8 fix-1`
  - 恢复 `Analysis success` section header 标题左对齐，不改变 `Phase 15.8` 的正文文本 right-click 路径。
- 独立 workspace `edition = "2024"` 里程碑
  - 以 `cargo fix --edition --workspace` 为起点，保留最小必要兼容修复。
  - `fc-ui-slint/src/presenter.rs` 对 provider settings load/save 的 mutex 持有范围做了显式收敛，避免依赖 pre-2024 temporary tail-expression drop-order。
  - 测试侧 settings 目录覆写收敛到 `settings.rs` 内部的 test-only guard，不再写进程级环境变量。
  - workspace `version` bump 到 `0.2.18`，但不引入新的产品行为。

## 4. Lessons retained from the upgrade

- 版本号应继续以 workspace manifest 为单一事实来源。
- editable-input 菜单优先走 Slint native surface，不回退到 overlay 拦截、私有事件链路或自写 caret / selection / editing。
- `API Key` 应继续保持保守 secret contract，不把 masked 文本默认视为可复制内容。
- read-only selectable content 的字体策略继续走 Slint 默认 generic family，并由现有 macOS bootstrap 负责接入系统字体。
- `Diff` detail 这类“宽表 + selectable 文本 + 变高行”场景，显式 `ScrollView` 比依赖升级后的 `ListView` 横向滚动路径更稳定。
- event-driven sync 比 broad `50ms` polling 更符合当前 UI contract。
- `.slint` 外置仍然是 deferred decision；只有收益明确超过 churn 时才值得重开。
- edition `2024` 迁移适合保持为独立里程碑，而不是与产品功能迭代混在同一轮。

## 5. What stayed unchanged through the upgrade

- `15.2D` IA、workspace shell、connected tabs、workbench seam 继续保持 accepted baseline。
- Compare Status 继续保持 summary-first。
- loading-mask 继续是 UI-local overlay，不扩张成全局 loading controller。
- toast 继续是 UI-local overlay，不扩张成全局 notification controller。
- window-local non-input context-menu core 继续只覆盖 non-input safe surfaces。
- `Analysis success` 正文文本 right-click 继续走 Slint native text surface；section header / chrome 继续走 window-local non-input context-menu core；`Risk Level` 继续保持显式 `Copy` 按钮-only。
- `Search` 继续保留显式 `Clear` 按钮。
- `SelectableDiffText` 行级右键菜单继续 deferred。

## 6. Verification status

- 各 phase 在其实施轮次内已通过对应的 `cargo check --workspace` / `cargo test --workspace` 验证。
- `cargo run -p fc-ui-slint` 启动级 smoke 也已在对应实施轮次内通过。
- 本文件当前轮次不再重复跑这些命令，因为当前任务仅是文档归档整理，不涉及代码改动。
- 真实 macOS 桌面环境下的最终人工视觉验收仍属于人工责任，不应在文档里伪造为自动验证结果。

## 7. How to use this file now

- 当前主参考文件是：
  - `docs/thread-context.md`
  - `docs/architecture.md`
- 只有在需要以下背景时再阅读本文件：
  - 依赖升级为什么按 `15.3A -> 15.8 fix-1` 分轮执行
  - 为什么 edition `2024` 作为独立里程碑执行
  - 某个升级修复的根因、经验和“不回退什么”的边界
- 当前主线下一步只有一个：`Phase 16`。
