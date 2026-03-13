# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本次初始化说明（2026-03-12）

- 改了什么：已完成 Phase 15.1A 的 `Diff` shell 收敛与 `detailed diff` 可读性深化，并同步更新 `docs/architecture.md` 到 `Phase 15.1A` 基线。
- 为什么影响下一线程：下一线程不应再重复做 Diff 壳层搭建，应直接转入 `Phase 15.1B`（Analysis View 产品化）与剩余回归验证。
- 保持不变：IA 仍是 `App Bar + Sidebar + Workspace`；未引入 tree mode；`fc-core/fc-ai/fc-ui-slint` 边界不变。

## 快照（Snapshot）

- 日期：2026-03-12（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（Phase 15.1A 落地中）
- 最近提交：
  - `1703032` phase summary: add comments/refactor docs and add thread context
  - `6d528cd` Phase 15.1A：File View shell 收敛 + Diff View 深化
  - `3a723c4` Phase 15.0 fix-4 + fix-5 consolidation
- 当前架构基线：`docs/architecture.md`（`Phase 15.1A` + `Phase 15.1B entry` priorities）

## 当前目标（Execution Focus）

1. 完成 Phase 15.1A 的收口验证（状态覆盖、最小窗口、全屏窗口、无基础回归）。
2. 将后续实现重心转向 `Phase 15.1B`：Analysis View 产品化（不改 IA）。
3. 继续推进结果导航效率与 provider hardening，保持在既有边界内。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `fc-ui-slint` 的 Workspace shell 一致性、状态切换稳定性、可读性增强
  - Analysis 展示产品化（限于现有 `File View` shell）
  - 结果导航效率优化（仅限当前 IA）
  - provider 可靠性控制（UI/`fc-ai` 层）
- Out of Scope：
  - IA 重置（`App Bar + Sidebar + Workspace` 保持不变）
  - Tree explorer / compare-view dual mode
  - Compare View 新模式或目录树扩展
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
  - Phase 15.1A 自检与收口（Diff: no selection/loading/detailed/preview/unavailable）
  - 确认文档契约与代码行为一致（`architecture.md` + `thread-context.md`）
- Next：
  - `Phase 15.1B` Analysis View 产品化（不扩展 schema，不改 Sidebar IA）
  - 结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
- Later：
  - 承接 `docs/architecture.md` 中 deferred decisions 的 provider hardening 后续事项

## 已知风险与评审重点（Known Risks / Review Focus）

1. 在 15.1B 推进时误改 15.1A 已收敛的 Diff shell contract。
2. 运行时同步回归（timer polling、model refresh 边界、状态抖动/过期）。
3. 在 `app.rs` 中混淆 tabs/modal/sync/events 职责导致跨 tab 互相污染。
4. Provider Settings 生命周期回归（校验、持久化、modal 开关）。

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
