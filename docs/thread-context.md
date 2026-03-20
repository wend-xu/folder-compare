# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的短周期交接，只记录当前事实、边界和下一步，不再把已完成的 phase train 当作待执行队列。

## 本轮更新说明（2026-03-19）

- 本轮执行 `Phase 16A`，只做 Sidebar 信息架构收口，不改 IA、不改 compare/diff/analysis 核心数据结构。
- 已完成并关闭：
  - `Phase 16A`
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
- 当前稳定基线：
  - workspace `version = "0.2.18"`
  - workspace `edition = "2024"`
  - `rust-toolchain = 1.94.0`
  - workspace `rust-version = 1.94`
  - `slint = 1.15.1`
  - `slint-build = 1.15.1`
  - `15.2E` 已在上述基线上落地
  - event-driven sync 已落地
  - persistent `VecModel` 已落地
  - `Diff` 显式 `ScrollView` 视口已落地
  - shared `UiTypography.selectable_content_font_family` 已落地
  - non-input context-menu visual polish 已落地
  - `Analysis success` native text-surface right-click 已落地
  - section header 左对齐修复已落地
  - `Compare Status` 块内 detail tray + `Copy Summary` / `Copy Detail` 已落地
  - `Compare Status` 折叠区与展开 tray 区的右键菜单覆盖已统一
  - `Results / Navigator` 顶部集合状态条已落地
- 保持不变：
  - `15.2D` 的 IA 与 shell contract 不变
  - connected tabs + attached workbench surface 不变
  - `Diff/Analysis` shell 不变
  - editable-input context-menu contract 不变
  - `Provider Settings -> API Key` secret contract 不变
  - window-local non-input context-menu core contract 不变
  - loading-mask / toast boundary 不变
  - UI 继续使用内联 `slint::slint!`
- 为什么下一步才是 `Phase 16`：
  - `phase15 summary` 的职责是把依赖升级、closeout 和独立 edition 里程碑统一收束成“当前事实”；
  - 只有先消除主文档里的旧 roadmap 叙事，下一线程才不会把已完成事项再次当成待执行；
  - 因此当前 closeout 完成后，下一步才进入 `Phase 16`，而不是继续重开 `15.3A` 到 `15.8 fix-1` 或 edition 兼容修复。

## 快照（Snapshot）

- 日期：`2026-03-19`（Asia/Shanghai）
- 分支：当前执行 `Phase 16A`
- 工作区：`fc-ui-slint` Sidebar 表达层、菜单接线与主文档同步改动
- 最近提交：
  - `1311f96` edition-2024 milestone
  - `7e59de3` Phase 15.8 fix-1: section title align
  - `51b28cd` Phase 15.8：Analysis success selectable-text native menu
- 当前主参考：
  - `docs/architecture.md`：长期“当前架构基线 + deferred decisions + next priority”
  - `docs/upgrade-plan-rust-1.94-slint-1.15.md`：升级与独立 edition 里程碑的归档背景

## 当前目标（Execution Focus）

1. `Phase 16A` 已完成；Sidebar 仍然保持 `Compare Inputs -> Compare Status -> Filter / Scope -> Results / Navigator`。
2. 后续线程继续 `Phase 16` 剩余子项时，不要重开 `Phase 15.3A` 到 `Phase 15.8 fix-1`，也不要重开独立 workspace `edition = "2024"` 里程碑。
3. 继续保持主文档与当前代码事实一致，不创建额外 phase checklist / summary 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `phase15 summary`
  - `docs/thread-context.md`、`docs/architecture.md`、`docs/upgrade-plan-rust-1.94-slint-1.15.md` 的口径统一
  - 用“当前事实”替换旧的升级路线叙事
- Out of Scope：
  - `Phase 16`
  - edition `2024` 兼容修复回合
  - 新的菜单 / 输入 / selectable-text / tree mode / theme / loading / toast / controller 方案
  - UI 或业务代码改动

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是 accepted baseline。
5. Compare Status 保持 summary-first，不演化为第二个重型详情面板。
6. `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框继续走 `slint 1.15.1` 原生 editable-input context menu。
7. `Provider Settings -> API Key` 继续保持专用 `ApiKeyLineEdit`：hidden=`Paste` only；visible=`Select All`、`Copy`、`Paste`、`Cut`。
8. `Analysis success` 正文文本继续走 Slint native text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`）；section header / chrome 继续走 window-local `Copy` / `Copy Summary` 菜单；`Risk Level` 继续保持显式 `Copy` 按钮-only。
9. 不重新引入 broad `50ms` polling，不回退 persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared `UiTypography.selectable_content_font_family`。
10. 不把 `Phase 16`、新菜单方案、或新的产品行为变更混入当前文档 closeout。

## 当前稳定事实（Stable Facts）

- 依赖与工具链：
  - `Cargo.toml` 已固定 workspace `version = "0.2.18"`、workspace `edition = "2024"`、workspace `rust-version = "1.94"`、`slint = "=1.15.1"`、`slint-build = "=1.15.1"`
  - `rust-toolchain.toml` 已固定 `channel = "1.94.0"`
  - `docs/macos_dmg.sh` 继续从 workspace manifest 派生 bundle / DMG / ZIP 版本
- UI / shell：
  - `15.2E` 已在当前基线上发货
  - `Diff` 与 `Analysis` 共用已验收的 workbench shell，不改 tabs / header / content 层次
  - `Diff` detail 长行横向滚动维持显式 `ScrollView` 视口，尾行通过 scrollbar-safe spacer 避免被横向滚动条遮挡
  - `SelectableDiffText` 与 `SelectableSectionText` 共用 `UiTypography.selectable_content_font_family`
  - `Compare Status` 继续保持 summary-first，并在块内支持 `Show details / Hide details` tray
  - `Compare Status` 右键菜单支持 `Copy Summary` / `Copy Detail`
  - `Compare Status` 折叠区与展开 tray 区都支持同一套上下文菜单
  - `Filter / Scope -> Search` 当前 contract 收口为 path/name 匹配
  - `Filter / Scope` 不再向用户重复显示单独的 `scope` 文案
  - `Results / Navigator` 顶部摘要现在表达当前结果集合状态（`Showing visible / total ...`）
  - Analysis success 正文文本支持 native text-surface `Copy` / `Select All` right-click
  - Analysis success section header 标题继续保持显式左对齐，且不会遮挡右上角 inline `Copy`
- 运行时：
  - compare / diff / analysis 后台完成态继续通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 回推 UI
  - `Results / Navigator` 与 `Diff` 行模型继续使用 persistent `VecModel`
  - loading-mask timeout copy 继续使用按 busy phase 调度的一次性 timer

## 下一步（Next）

- 唯一建议的下一步是继续剩余 `Phase 16` 工作。
- 后续实现应建立在当前 `0.2.18 + edition 2024 + rust 1.94.0 + slint 1.15.1 + Phase 16A` 基线上，不重开升级、edition 迁移或 `Phase 15` closeout。
- 继续保持当前 shell / menu / loading / toast / event-driven sync contract 不变。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `docs/upgrade-plan-rust-1.94-slint-1.15.md`（仅在需要升级归档背景时再读）
4. `crates/fc-ui-slint/src/app.rs`
5. `crates/fc-ui-slint/src/context_menu.rs`
6. `crates/fc-ui-slint/src/presenter.rs`
7. `crates/fc-ui-slint/src/settings.rs`
8. `crates/fc-ui-slint/src/ui_palette.slint`
9. `Cargo.toml`
10. `rust-toolchain.toml`

## 验证（Verification）

- 本轮验证重点是 `Phase 16A` Sidebar 表达与菜单接线：
  - `cargo check -p fc-ui-slint`
  - `cargo test -p fc-ui-slint`
- 本轮未运行 `cargo run -p fc-ui-slint`：
  - 原因：本轮未在此线程内做桌面 GUI smoke；真实视觉与交互验收仍保留给人工

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> `docs/upgrade-plan-rust-1.94-slint-1.15.md` 只在需要升级与独立 edition 里程碑归档背景时再阅读。  
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8`、`Phase 15.8 fix-1`，以及独立 workspace `edition = "2024"` 里程碑，全部视为已完成。  
> 把当前稳定基线视为：workspace `version = "0.2.18"`、workspace `edition = "2024"`、`rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`，且 `15.2E`、event-driven sync、persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared `UiTypography.selectable_content_font_family`、non-input context-menu visual polish、`Analysis success` native text-surface right-click、section header 左对齐修复均已稳定。  
> 当前默认进入 `Phase 16`，不要重开 phase15 summary、依赖升级 closeout、或 edition `2024` 兼容修复。  
> 保持现有产品行为、UI contract、shell / menu / loading / toast 边界不变。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `docs/upgrade-plan-rust-1.94-slint-1.15.md` 是否也需要更新。
- 每次更新都必须明确写出：
  - 什么已经完成
  - 当前稳定基线是什么
  - 什么保持不变
  - 下一步为什么是 `Phase 16`
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
