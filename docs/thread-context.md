# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-18）

- 轮次定义：`Dependency upgrade executed through Phase 15.8; Phase 15 closeout is complete; Phase 16 returns to mainline afterwards`。
- 改了什么：
  - 实际完成 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8`；
  - workspace 版本继续保持 `0.2.17`；依赖与工具链继续保持 `rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`；
  - 版本号单一事实来源继续落在 workspace `Cargo.toml`，`docs/macos_dmg.sh` 继续从 manifest 派生 bundle / DMG / ZIP 版本；
  - `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框继续走 `slint 1.15.1` 原生 editable-input context menu；`Provider Settings -> API Key` 继续保持专用 `ApiKeyLineEdit` 的保守 secret contract；
  - `Phase 15.5 fix-1` / `fix-2` 的 read-only selectable content glyph fallback 保护与共享 `UiTypography.selectable_content_font_family` token 继续保留；
  - `Phase 15.5 fix-3` 的 `Diff` 显式 `ScrollView` 视口、横向滚动条恢复与 scrollbar-safe spacer 继续保留；
  - `Phase 15.6` 的 event-driven sync、loading-mask one-shot timeout copy、持久 `VecModel` 基线继续保留；当前仍不外置 `.slint`；
  - 本轮 `Phase 15.7` 已完成 non-input context-menu visual polish：菜单面板改为更紧凑的圆角、内边距和 item 高度，并增加分层阴影、顶部高光、inset hover surface、左侧 accent strip 与更柔和的 disabled 文本；
  - `Phase 15.7` 没有改 `context_menu.rs` 的 action build / dispatch / close contract，没有扩张到 `SelectableDiffText` 行级右键菜单、editable-input 菜单或平台原生菜单桥接；
  - 本轮 `Phase 15.8` 已完成：`Analysis success` 的 `SelectableSectionText` 现已直接使用 Slint native text surface，`Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 的正文文本支持最小化 `Copy` / `Select All` 右键菜单，菜单对象仍是当前选中文本；
  - `Phase 15.8` 保持了原约束：section header / chrome 继续走现有 window-local `Copy` / `Copy Summary` 菜单，`Risk Level` 继续保持显式 `Copy` 按钮-only，没有改 `context_menu.rs`，也没有扩张到 Analysis shell-state text、`SelectableDiffText` 或 editable inputs；
  - `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 右上角 inline `Copy` 按钮此前失效的根因已确认并修复：原先 `AnalysisSectionPanel` header 的整块右键命中层覆盖了按钮区域，本轮已把命中层收敛到 header label lane，不再遮挡 copy action；
  - `cargo check --workspace`、`cargo test --workspace` 已通过；`cargo run -p fc-ui-slint` 启动级 smoke 已进入运行态。真实 macOS arm64 最终人工视觉验收仍待人工确认。
- 为什么影响下一线程：如果下一线程仍把 `15.8` 当作未完成，就可能重复实现已落地的 native text-surface right-click，或者错误地再次把这条正文文本右键路径接回 window-local menu core；如果忽略“`Phase 15` 现已完成 closeout、`Phase 16` 已回主线”这一边界，就容易把导航增强、edition 升级或 phase15 总结重新混进同一轮；如果忽略“`.slint` 外置收益仍不足”这一结论，则容易再次打开高 churn 的 UI 结构迁移。
- 保持不变：`15.2D` 的 IA 与 shell contract 不变；`Diff/Analysis` shell、connected tabs、loading scope boundary、`SelectableSectionText` / `SelectableDiffText` 的可选中文本边界、modal draft 行为、以及本地 `toast/loading/menu` controller 边界均不改；普通输入继续走 Slint 原生菜单，`API Key` 继续 hidden=`Paste` only；workspace `edition` 仍是 `2021`；UI 继续使用内联 `slint::slint!`。

## 快照（Snapshot）

- 日期：2026-03-18（Asia/Shanghai）
- 分支：`dev-phase15_3_to_6_upgrade_plan`
- 工作区：有改动（`Phase 15.8` 代码与三份主文档同步更新，待提交）
- 最近提交：
  - `3b13629` phase 15.5
  - `e8bb75a` Phase 15.3A / 15.3B / 15.4 doc sync
  - `c90f746` Phase 15.3A / 15.3B / 15.4
- 当前架构基线：`docs/architecture.md`（`15.2E` 已在 `rust 1.94.0 + slint 1.15.1` 基线上落地，`Phase 15.5 fix-1` / `fix-2` / `fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8` 已完成；Analysis success 正文文本 right-click 已收敛为 native text surface，`Phase 16` 回主线）

## 当前目标（Execution Focus）

1. 把 `Phase 15.3A` 到 `Phase 15.8` 视为已完成；下一线程默认回主线 `Phase 16` 或其他独立里程碑，不要重开 phase15 closeout。
2. 保持 phase train 分层：不要回头重做 `15.3A/15.3B/15.4/15.5/15.5 fix-1/15.5 fix-2/15.5 fix-3/15.6/15.7/15.8`，也不要把 selectable-text menu、edition `2024` 升级或 phase15 总结混进同一轮。
3. 后续每个 phase / 里程碑执行时同步更新 `architecture.md`、`thread-context.md`、`upgrade-plan-rust-1.94-slint-1.15.md`，不再创建额外 phase checklist 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 以 `rust 1.94.0 + slint 1.15.1 + 15.5/15.5 fix-1/15.5 fix-2/15.5 fix-3/15.6/15.7/15.8 已完成` 为基线，推进后续 mainline 工作
  - 后续阶段执行时同步更新三份主文档
  - 继续维持 `15.x` 已收敛的 shell/menu/loading/toast 边界
- Out of Scope：
  - 重复执行 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`
  - 重复执行 `Phase 15.5`
  - 重复执行 `Phase 15.6`
  - 重复执行 `Phase 15.7`
  - 重复执行 `Phase 15.8`
  - `edition = "2024"` 迁移
  - phase15 总结 / `architecture.md` 去历史化清理
  - IA 重置、tree mode、Compare View 新模式
  - 全局 loading/theme/notification controller
  - 通过 overlay `TouchArea`、私有事件链路、或自写 caret/selection/editing 逻辑硬接 `LineEdit` / `TextInput`

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI/网络/provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration/presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是当前 accepted baseline，同一时刻仅一个主分支激活。
5. Compare Status 保持 summary-first，不演化为重型第二详情面板。
6. 依赖升级路线已完成到 `Phase 15.8`；`Phase 16` 与 edition 升级均在后续独立里程碑推进，并在各自轮次同步主文档。

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
  - `Phase 15.3A` / `15.3B` / `15.4` / `15.5` / `15.5 fix-1` / `15.5 fix-2` / `15.5 fix-3` / `15.6` / `15.7` / `15.8` 已完成并通过验证
  - 当前稳定基线是 `15.2E` 已落地 + `rust 1.94.0 + slint 1.15.1 + workspace version 0.2.17 + event-driven sync + context-menu visual polish + Analysis success native text-surface right-click`
  - Analysis success 的 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 正文文本现已原生支持 `Copy` / `Select All` 右键菜单，section inline `Copy` 按钮也已恢复可点击
  - `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 已通过
- Next：
  - `Phase 16`：结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
- Later：
  - `edition = "2024"`：单列里程碑，不并入当前 phase
  - phase15 总结 / `architecture.md` 去历史化整理
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
9. `Phase 15.8` 已把 Analysis success 正文文本 right-click 固定在 native text surface；不要把这条可选中文本路径接回 window-local menu core，也不要用 undocumented selection API、overlay interception 或自写 caret/selection 去追求“更严格”的 selection-aware contract。
10. 不要重新打开 `Phase 15.7` / `Phase 15.8` 的菜单 closeout 话题，也不要把 `Phase 16`、edition 升级、phase15 总结、残留输入 affordance 评估或新的菜单 controller 方案混成同一轮；也不要重新开临时 checklist 文档绕开主文档同步。

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
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8` 视为已完成。  
> 把当前基线视为：`15.2E` 已在 `rust 1.94.0 + slint 1.15.1` 上落地，read-only selectable content 的 glyph fallback 回归已修复并收敛到共享 `UiTypography` token，`Diff detail` 横向滚动条已切到显式 `ScrollView` 路径恢复稳定，UI 主同步路径已从 `50ms` polling 收敛为 event-driven sync，non-input context menu 的 visual polish 已完成，Analysis success `SelectableSectionText` 也已原生支持 text-surface `Copy` / `Select All` right-click。  
> 不要重开 `Phase 15.8`；默认进入 `Phase 16` 或其他独立主线里程碑。  
> 保持 Analysis success 正文文本继续走 Slint native text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`），不要把 selectable-text right-click 接回 window-local non-input menu core。  
> 保持 section header / chrome 继续使用现有 window-local `Copy` / `Copy Summary` 菜单；`Risk Level` 继续保持显式 `Copy` 按钮-only；Analysis success section inline `Copy` 按钮不要再被 header 右键命中层遮挡。  
> 接受无 selection 时保持 Slint / 系统一致的 disable 或 no-op 行为，不要为了“严格 selection-aware menu”引入 undocumented selection API、overlay `TouchArea`、私有事件链路或自写 caret/selection/editing。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
> 不要回退 `Phase 15.5` 已落地的输入菜单 contract。  
> 不要把 `Workspace Diff detail line` 的全角标点问题重新归因到编码或 `fc-core`；除非有新证据，否则把它视为已在 `fc-ui-slint` 字体回退层修复。  
> 不要重新引入 broad `50ms` polling，也不要在当前轮次顺手推进 `SelectableDiffText` 行级右键菜单、edition `2024` 升级或 phase15 总结；这些都必须保持独立范围。  
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
