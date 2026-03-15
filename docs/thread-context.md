# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-15）

- 轮次定义：`Phase 15.2A: toast-controller`（基线：`Phase 15.1B fix-3` 已验收代码）。
- 改了什么：
  - 在 `fc-ui-slint` 落地 window-local `toast-controller`（`banner` + 顶部偏中 `toast` 两种 placement）；
  - 支持 `success/warn/error/info` tone、可配置 duration（默认 `2000ms`）、`replace/queue/auto` 策略；
  - 将 `show_weak_feedback` / `copy_text_with_feedback` 抽象进 controller，copy 反馈继续走 `banner`；
  - 增加 `Provider settings saved` 的 overlay `toast` 实际接入；
  - 可测策略逻辑拆入独立 policy 模块并补最小单测。
- 为什么影响下一线程：此前文档中“global toast 仍 deferred”容易让人误判为“尚无本地通知基线”；现在已有可复用本地 controller，后续线程应在此基线上扩展而不是重造。
- 保持不变：IA 仍是 `App Bar + Sidebar + Workspace`；`Diff/Analysis` shell、connected tabs、copy baseline、helper/action strip 语义不回退；`status_text`/inline error/state shell 语义不改。

## 快照（Snapshot）

- 日期：2026-03-15（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（`fc-ui-slint` toast-controller + docs 对齐，待本线程提交）
- 最近提交：
  - `6afab36` phase 15.1B fix-3：Analysis selectable text（success sections only）
  - `8d932c1` phase 15.1B fix2: analysis success cannot scroll
  - `19388d5` Phase 15.1B fix1 ：Analysis View 产品化 收口
- 当前架构基线：`docs/architecture.md`（`Phase 15.2A` 本地 toast-controller 已落地；global route/persistence/cross-surface orchestration 仍 deferred）

## 当前目标（Execution Focus）

1. 以 `Phase 15.1B fix-3` 为稳定基线，维持 `Diff/Analysis` 现有 shell contract。
2. 在 `fc-ui-slint` 内维持并复用 window-local `toast-controller`，避免把短生命周期反馈状态塞回 `AppState/Presenter`。
3. 后续若扩展通知能力，优先在本地 controller 上增量演进（非全局事件总线），并继续保持最小影响面。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `fc-ui-slint` 内的本地 toast/banner 编排（策略、队列、计时、UI 渲染接线）
  - 保持 `Diff / Analysis` connected workbench shell 与 copy 反馈语义稳定
  - `docs/architecture.md` / `docs/thread-context.md` 与当前 phase 事实对齐
  - 最小回归验证（`cargo check --workspace`、`cargo test -p fc-ui-slint`、必要时 UI smoke）
- Out of Scope：
  - IA 重置（`App Bar + Sidebar + Workspace` 保持不变）
  - Tree explorer / compare-view dual mode
  - Compare View 新模式或目录树扩展
  - `fc-core` / `fc-ai` 合约改动
  - 全局通知中心、跨窗口/跨 surface 路由、持久化通知
  - 超出现有边界契约的 AI provider 架构扩展

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI/网络/provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration/presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是当前 accepted baseline，同一时刻仅一个主分支激活。
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
  - 维护 `Phase 15.2A` 本地 toast-controller 基线（banner/toast、策略、duration、timer）
  - 保持 copy feedback 走 `banner` placement，不扩散为全局漂浮提示
- Next：
  - 结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
  - 在现有 controller 上评估是否需要更多低风险 `toast` 接入点（仅非阻断成功/信息类）
- Later：
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `banner` placement 只能复用现有 helper/action strip 弱提示位，不能改写 Diff/Analysis 的阅读节奏。
3. overlay `toast` 必须是布局外层叠加，不得破坏现有滚动与主内容高度。
4. 双击行号复制与 Analysis section copy 的反馈时序不能回退（replace/auto 行为需可预期）。
5. 运行时同步回归（timer polling、model refresh 边界、状态抖动/过期）。
6. 在 `app.rs` 中混淆 tabs/modal/sync/events 职责导致跨 tab 互相污染。
7. `Results / Navigator -> Diff` 与 `Analysis -> Diff` 的链路一致性不能因后续 polish 再次分叉。

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
> 以当前 `Phase 15.1B fix-3` + `Phase 15.2A toast-controller` 版本为基线。  
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast-controller 重新塞进 `AppState/Presenter`。  
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
