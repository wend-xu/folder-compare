# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-18）

- 轮次定义：`Dependency upgrade executed through Phase 15.6; Phase 16 is the next default target and Phase 15.7 stays optional`。
- 改了什么：
  - 实际完成 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`；
  - workspace 版本现收敛到 `0.2.17`；依赖与工具链继续保持 `rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`；
  - 版本号单一事实来源已落到 workspace `Cargo.toml`，`docs/macos_dmg.sh` 改为从 manifest 派生 bundle / DMG / ZIP 版本；
  - `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框现已直接走 `slint 1.15.1` 原生 editable-input context menu；
  - `Provider Settings -> API Key` 已收敛到专用 `ApiKeyLineEdit`：hidden=`Paste` only，visible=`Copy/Cut/Paste/Select All`，并额外阻断 hidden 状态下的 `Cmd/Ctrl+A/C/X`；
  - `API Key` 外置 `Show/Hide` 按钮已改为字段内 reveal toggle；`Search` 的手工 `Clear` 按钮则因当前 macOS native `cupertino` style 缺少稳定 clear affordance 而暂时保留；
  - `Workspace Diff detail line` 在升级后把原始文本中的全角冒号 `：` 渲染成方框；根因不是编码或 diff 数据变化，而是 `SelectableDiffText` / `SelectableSectionText` 这条 `TextInput` 渲染链在 `slint 1.15.1` 新字体引擎下对 mixed Latin+CJK 文本的 glyph fallback 发生变化；
  - `Phase 15.5 fix-1` 已在 UI 层为 read-only selectable content 增加 glyph fallback 保护，恢复 `Workspace Diff detail line` 的全角标点显示；
  - `Phase 15.5 fix-2` 已把这层保护从 `MainWindow` / `AnalysisSectionPanel` 透传收敛为共享 Slint global token：`UiTypography.selectable_content_font_family`，行为不变但实现更干净；
  - `Workspace Diff detail` 横向滚动条在升级后也出现回归；根因不是本轮字体修复，而是 `slint 1.15.1` 下继续依赖 `ListView` 承载这类“宽表 + selectable TextInput + 变高行”的自定义 diff viewer 时，横向滚动条露出不再稳定；
  - `Phase 15.5 fix-3` 已把 `Diff` body 改为显式 `ScrollView` 视口，header 继续镜像其 `viewport-x`，并把 scrollbar-safe inset 收敛为内容末尾 spacer，恢复横向滚动条且避免尾行被滚动条遮挡；
  - `Phase 15.6` 已移除 UI 主同步路径上的常驻 `50ms` 轮询：compare / diff / analysis 后台完成态现在通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 直接回推 UI 线程；
  - loading-mask timeout 文案已改为按 busy phase 切换调度的一次性 timer，不再依赖 repeated watchdog tick，但 scope / timeout copy contract 不变；
  - `Results / Navigator` 与 `Diff` 行模型已改为持久 `VecModel`，只在相关 payload 变化时 `set_vec()` 更新，不再反复 `ModelRc::new(VecModel::from(...))`；
  - `Phase 15.6` 已评估外置 `.slint`；当前收益不足以覆盖迁移 churn，因此继续保持内联 `slint::slint!` 和现有 `build.rs` 边界；
  - 右键菜单外观美化已在升级计划中登记为 `Phase 15.7`，并明确与 `Phase 15.6` 同步清理、`Phase 16` 导航增强分离；
  - `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 已通过；macOS arm64 人工 smoke 仍需最终人工确认，但目前未发现回归，同时 diff 加载性能体感仍明显优于旧基线；
  - 后续 phase 将按“执行同时更新主文档”的方式推进，临时 `docs/phase-15-upgrade-checklists.md` 不再保留。
- 为什么影响下一线程：如果下一线程仍把 `15.6` 当作未完成，会重复做已完成的去轮询和 model cleanup；如果忽略 `15.6` 已接受的 event-driven sync baseline，就可能重新把 loading-mask timeout、busy completion、menu close contract 混回 broad polling；如果忽略“已评估但未外置 `.slint`”这一结论，则容易在没有明确收益时再次打开高 churn 的 UI 文件迁移；如果把 `15.7` style-only polish 和 `Phase 16` 导航增强混在一起，又会重复制造 phase 边界噪音。
- 保持不变：`15.2D` 的 IA 与 shell contract 不变；`Diff/Analysis` shell、connected tabs、loading scope boundary、`SelectableSectionText` / `SelectableDiffText` 的可选中文本边界、modal draft 行为、以及本地 `toast/loading/menu` controller 边界均不改；普通输入继续走 Slint 原生菜单，`API Key` 继续 hidden=`Paste` only；workspace `edition` 仍是 `2021`；UI 继续使用内联 `slint::slint!`，本轮没有提前推进 `Phase 15.7` 或 `Phase 16`。

## 快照（Snapshot）

- 日期：2026-03-18（Asia/Shanghai）
- 分支：`dev-phase15_3_to_6_upgrade_plan`
- 工作区：有改动（`Phase 15.6` 已执行，代码与三份主文档已同步到 `0.2.17` 基线）
- 最近提交：
  - `3b13629` phase 15.5
  - `e8bb75a` Phase 15.3A / 15.3B / 15.4 doc sync
  - `c90f746` Phase 15.3A / 15.3B / 15.4
- 当前架构基线：`docs/architecture.md`（`15.2E` 已在 `rust 1.94.0 + slint 1.15.1` 基线上落地，且 `Phase 15.5 fix-1` / `fix-2` / `fix-3` 与 `Phase 15.6` 已完成；当前默认下一目标是 `Phase 16`，`Phase 15.7` 仅在明确要求 style-only polish 时再单开）

## 当前目标（Execution Focus）

1. 把 `Phase 15.6` 视为已完成；下一步默认执行 `Phase 16`，在当前 event-driven sync + persistent `VecModel` 基线上做结果导航增强。
2. 保持 phase train 分层：`Phase 15.7`（optional, style-only）与 `Phase 16` 必须分开执行；不回头重做 `15.3A/15.3B/15.4/15.5/15.6`。
3. 后续每个 phase 执行时同步更新 `architecture.md`、`thread-context.md`、`upgrade-plan-rust-1.94-slint-1.15.md`，不再创建额外 phase checklist 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 以 `rust 1.94.0 + slint 1.15.1 + 15.5/15.5 fix-1/15.5 fix-2/15.5 fix-3/15.6 已完成` 为基线，默认推进 `Phase 16`
  - 如用户显式要求，也可以单开 `Phase 15.7` 做 style-only 菜单美化
  - 后续阶段执行时同步更新三份主文档
  - 继续维持 `15.x` 已收敛的 shell/menu/loading/toast 边界
- Out of Scope：
  - 重复执行 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`
  - 重复执行 `Phase 15.5`
  - 重复执行 `Phase 15.6`
  - 在同一轮里同时做 `Phase 15.7`、`Phase 16`
  - `edition = "2024"` 迁移
  - IA 重置、tree mode、Compare View 新模式
  - 全局 loading/theme/notification controller
  - 通过 overlay `TouchArea`、私有事件链路、或自写 caret/selection/editing 逻辑硬接 `LineEdit` / `TextInput`

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI/网络/provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration/presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是当前 accepted baseline，同一时刻仅一个主分支激活。
5. Compare Status 保持 summary-first，不演化为重型第二详情面板。
6. 依赖升级路线已完成到 `Phase 15.6`；后续必须在该基线上推进 `Phase 15.7`（optional, style-only）或 `Phase 16`，并在同一轮同步主文档。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`：当前执行上下文与交接清单
2. `docs/architecture.md`：长期架构契约与 deferred decisions
3. `docs/upgrade-plan-rust-1.94-slint-1.15.md`：升级执行结果、剩余阶段边界、Codex 提示词
4. `crates/fc-ui-slint/src/app.rs`：UI shell、modal、sync、callbacks
5. `crates/fc-ui-slint/src/context_menu.rs`：window-local context-menu core 与 safe-surface 边界
6. `crates/fc-ui-slint/src/presenter.rs`：状态编排与命令流
7. `crates/fc-ui-slint/src/state.rs`：UI state machine 与派生展示字段
8. `crates/fc-ui-slint/src/settings.rs`：Provider Settings 持久化与 API Key 相关约束
9. `crates/fc-ui-slint/src/ui_palette.slint`：本地 semantic palette + typography token
10. `Cargo.toml`、`rust-toolchain.toml`、`docs/macos_dmg.sh`：当前版本基线与打包版本来源

## 当前工作队列（Active Work Queue）

- Now：
  - `Phase 15.3A` / `15.3B` / `15.4` / `15.5` / `15.5 fix-1` / `15.5 fix-2` / `15.5 fix-3` / `15.6` 已完成并通过验证
  - 当前稳定基线是 `15.2E` 已落地 + `rust 1.94.0 + slint 1.15.1 + workspace version 0.2.17 + event-driven sync cleanup`
  - `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 已通过
- Next：
  - `Phase 16`：结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
  - `Phase 15.7`：可选的 context-menu visual polish（style-only）
- Later：
  - `edition = "2024"`：单列里程碑，不并入当前 phase
  - `Search Clear` affordance：仅在原生 style surface 明确提供稳定 clear 能力时再单开小轮次收敛
  - `.slint` 外置：仅在维护收益或编译链路收益明确时再重开评估
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `context-menu core` 必须保持 window-local；不要把 menu lifecycle 反向塞回 `AppState/Presenter`。
3. 后续 phase 不得回退 `Phase 15.5` 已落地的原生 editable-input surface；不要把输入菜单重新改回 overlay 拦截、私有事件链路或自写编辑逻辑。
4. 输入菜单 contract 不能回退：普通输入继续走 Slint 原生菜单；`API Key` 继续保持 hidden=`Paste` only，并保留 hidden 状态下的 `Cmd/Ctrl+A/C/X` 阻断。
5. 右键接线不能破坏 `Results / Navigator` 左键选择、Diff 行号双击复制、Analysis success 文本选择与滚动；`Risk Level` 仍保持 `Copy` 按钮-only，除非文档 contract 被显式更新。
6. `toast-controller` 仍是 overlay toast only，`loading-mask` 仍保持当前范围；不要因为后续 phase 而回退这些边界。
7. 不要重新引入 broad `50ms` polling 作为 UI 主同步路径；若未来新增轮询，必须给出明确且局部的保留理由。
8. 不要移除 `UiTypography.selectable_content_font_family` 当前的 glyph fallback 收敛，也不要把 `Diff detail` 的显式 `ScrollView` 回退到升级后的 `ListView` 路径，除非已有真实样本验证新路径稳定。
9. 不要把 `Phase 15.7`、`Phase 16`、以及残留输入 affordance 评估混成同一轮；也不要重新开临时 checklist 文档绕开主文档同步。

## 验证命令（Verification Commands）

```bash
cargo check --workspace
cargo test --workspace
cargo run -p fc-ui-slint
```

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 再阅读 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。  
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6` 视为已完成。  
> 把当前基线视为：`15.2E` 已在 `rust 1.94.0 + slint 1.15.1` 上落地，read-only selectable content 的 glyph fallback 回归已修复并收敛到共享 `UiTypography` token，`Diff detail` 横向滚动条已切到显式 `ScrollView` 路径恢复稳定，UI 主同步路径也已从 `50ms` polling 收敛为 event-driven sync。  
> 下一步默认从 `Phase 16` 开始；除非本次明确要求 style-only 菜单美化，否则不要提前执行 `Phase 15.7`。  
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
> 不要把 `Phase 15.7`、`Phase 16` 和残留输入 affordance 评估混在同一轮。  
> 不要回退 `Phase 15.5` 已落地的输入菜单 contract，也不要用 overlay `TouchArea`、私有事件链路或自写 caret/selection/editing 去重做 editable inputs。  
> 不要把 `Workspace Diff detail line` 的全角标点问题重新归因到编码或 `fc-core`；除非有新证据，否则把它视为已在 `fc-ui-slint` 字体回退层修复。  
> 不要重新引入 broad `50ms` polling，也不要在当前轮次顺手推进 `SelectableDiffText` 行级右键菜单；这些都必须保持独立范围。  
> 执行同时同步更新 `docs/architecture.md`、`docs/thread-context.md`、`docs/upgrade-plan-rust-1.94-slint-1.15.md`，不要再创建额外 phase checklist 文档。  
> 仅执行本次任务范围内改动，并说明对 contract 的影响。

## 更新契约（Mandatory）

### Update triggers

同一 PR 内，以下任一变化发生时必须更新本文件：

1. 当前执行目标、队列顺序、短期 phase 约束发生变化。
2. 与当前推进相关的分支上下文变化（长期分支切换、里程碑切换）。
3. 风险画像、评审重点、验证命令发生变化。
4. 为避免新线程误判，handoff 指令需要调整。
5. 语言与术语策略发生变化（见 `Writing rules`）。

### Required sections to touch per trigger

- 编辑本文件时，必须更新 `快照（Snapshot）`。
- 优先级变化时，更新 `当前目标` 与 `当前工作队列`。
- 约束变化时，更新 `本阶段范围` 与 `硬契约`。
- 验证策略变化时，更新 `已知风险与评审重点` 与 `验证命令`。

### Writing rules

1. 以中文为主叙述，关键术语保留英文原词（如 `Workspace`、`Diff`、`Analysis`、`Provider Settings`）。
2. 保持短小、可执行、可交接，优先使用可操作条目。
3. 记录“当前事实与边界”，不复制冗长历史叙事。
4. 每次更新必须说明：改了什么、为什么影响下一线程、什么保持不变。
5. 术语命名应与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。

### Handoff Definition of Done

1. 新线程仅阅读本文件 + `docs/architecture.md` 即可开始实施。
2. 队列与约束与代码和评审意图一致。
3. `快照（Snapshot）` 中不存在过期分支/阶段假设。
