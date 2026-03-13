# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本次初始化说明（2026-03-13）

- 改了什么：在 `Phase 15.1A fix-1` 基础上完成 `fix-2`，进一步稳定 `Diff` header 的上下节奏，把 `DiffStateShell` 收口为填充 detail 区的正式状态面，并补上按行复制与横向滚动安全区/提示的保底交互；同步更新 `docs/architecture.md` 到 `15.1A fix-2` 事实。
- 为什么影响下一线程：下一线程不应再以 `fix-1` 为验收基线；应先完成 `15.1A fix-2` 的最终视觉 smoke，再决定是否正式切入 `Analysis View` 产品化。
- 保持不变：IA 仍是 `App Bar + Sidebar + Workspace`；本轮仍未进入 `15.1B`；未引入 tree mode；`fc-core/fc-ai/fc-ui-slint` 边界不变。

## 快照（Snapshot）

- 日期：2026-03-13（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（`Phase 15.1A fix-2` 落地后，待最终视觉验收）
- 最近提交：
  - `1703032` phase summary: add comments/refactor docs and add thread context
  - `6d528cd` Phase 15.1A：File View shell 收敛 + Diff View 深化
  - `3a723c4` Phase 15.0 fix-4 + fix-5 consolidation
- 当前架构基线：`docs/architecture.md`（`Phase 15.1A fix-2` + `Phase 15.1A exit / 15.1B entry` priorities）

## 当前目标（Execution Focus）

1. 完成 `Phase 15.1A fix-2` 的 Diff 验收（header 节奏、状态面归属、长行横向阅读、滚动条不挡底部、复制保底、最小窗口/全屏无基础回归）。
2. 在视觉 smoke 通过后，再决定是否进入 `Phase 15.1B`：Analysis View 产品化（不改 IA）。
3. 保持结果导航效率与 provider hardening 在既有边界内，不提前扩 phase。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `fc-ui-slint` 中 `File View / Diff` 的 shell 收口、状态面强化、横向阅读与复制保底能力补足
  - `docs/architecture.md` / `docs/thread-context.md` 与当前 phase 事实对齐
  - `15.1A fix-2` 的视觉与交互验收
- Out of Scope：
  - IA 重置（`App Bar + Sidebar + Workspace` 保持不变）
  - Tree explorer / compare-view dual mode
  - Compare View 新模式或目录树扩展
  - `Phase 15.1B` Analysis 产品化与结构重排
  - 未经阶段决策的 phase logic 改写
  - 超出现有边界契约的 AI provider 架构扩展

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI/网络/provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration/presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，同一时刻仅一个主分支激活。
5. Compare Status 保持 summary-first，不演化为重型第二详情面板。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`：当前执行上下文与交接清单
2. `docs/architecture.md`：长期架构契约与 deferred decisions
3. `crates/fc-ui-slint/src/app.rs`：UI shell、modal、sync、callbacks
4. `crates/fc-ui-slint/src/presenter.rs`：状态编排与命令流
5. `crates/fc-ui-slint/src/state.rs`：UI state machine 与派生展示字段
6. `crates/fc-ui-slint/src/bridge.rs`：UI 与 core/ai API 的映射边界
7. `crates/fc-ui-slint/src/settings.rs`：provider 配置加载/保存边界
8. `crates/fc-ai/src/services/analyzer.rs`：analysis 编排
9. `crates/fc-core/src/api/compare.rs` 与 `crates/fc-core/src/api/diff.rs`：core API 契约

## 当前工作队列（Active Work Queue）

- Now：
  - `Phase 15.1A fix-2` 自检与视觉验收（Diff: header cadence / no selection / loading / detailed / preview / unavailable）
  - 确认文档契约与代码行为一致（`architecture.md` + `thread-context.md`）
- Next：
  - 视觉验收通过后，进入 `Phase 15.1B` Analysis View 产品化（不扩展 schema，不改 Sidebar IA）
  - 结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
- Later：
  - 承接 `docs/architecture.md` 中 deferred decisions 的 provider hardening 后续事项

## 已知风险与评审重点（Known Risks / Review Focus）

1. `Diff` header/state shell 在最小窗口与全屏窗口下出现密度或错位回归。
2. 新的状态面承载方式如果过度强调顶部带状区，可能重新形成“子卡片感”或弱化 detail 区归属。
3. 可选择文本、按行复制与横向滚动安全区引入交互回归（滚动冲突、长行裁切、底部内容被挡、复制行为不一致）。
4. 运行时同步回归（timer polling、model refresh 边界、状态抖动/过期）。
5. 在 `app.rs` 中混淆 tabs/modal/sync/events 职责导致跨 tab 互相污染。

## 验证命令（Verification Commands）

```bash
cargo check --workspace
cargo test --workspace
cargo test -p fc-ui-slint
```

UI 变更可加做：

```bash
cargo run -p fc-ui-slint
```

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 保持当前 IA 与 phase 边界。  
> 先确认 `Phase 15.1A fix-2` 是否已经完成最终视觉验收，再决定是否进入 `15.1B`。  
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
