# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-16）

- 轮次定义：`Phase 15.2E assessment: editable input integration feasibility on top of stable 15.2D baseline`。
- 改了什么：
  - 先按要求复核 `15.2D` 已落地事实：shared `context-menu core` 仍保持 `window-local`，`Results/Navigator` 与 Analysis success `ScrollView` 滚动自动关闭菜单仍是当前基线，`AnalysisSectionPanel` anchor bug 已在现有代码中修复，`Risk Level` 仍只有显式 `Copy`；
  - 在锁定的 `slint = 1.8.0` 依赖上评估 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入与 `API Key` 的 editable context-menu 接入可行性；
  - 结论是当前版本没有稳定低风险的 editable-input menu hook：`LineEdit/TextInput` 有 `select-all/copy/paste/cut`，但没有等价于后续版本 `ContextMenuArea` 的右键接入面，因此没有落地任何 `15.2E` 代码；
  - 更新 `docs/architecture.md` 与本文件，把 `15.2E` 保持 deferred 的理由、`API Key` 的保守策略建议、以及下一线程不应尝试 overlay/private-event hack 的原因写成当前事实。
- 为什么影响下一线程：如果继续停留在 `slint = 1.8.0`，editable input integration 不应再尝试通过 `TouchArea`/私有事件链路硬接；主线可直接推进 phase 16，或等依赖升级后再回到 `15.2E`。
- 保持不变：`15.2D` 代码基线不变；IA 仍是 `App Bar + Sidebar + Workspace`；`Diff/Analysis` shell、connected tabs、loading scope boundary、`SelectableSectionText`/`SelectableDiffText`、所有输入绑定结构、modal draft 行为、以及本地 `toast/loading/menu` controller 边界均不改。

## 快照（Snapshot）

- 日期：2026-03-16（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（docs 对齐本次 `15.2E` assessment；代码保持 `15.2D` 基线）
- 最近提交：
  - `6afab36` phase 15.1B fix-3：Analysis selectable text（success sections only）
  - `8d932c1` phase 15.1B fix2: analysis success cannot scroll
  - `19388d5` Phase 15.1B fix1 ：Analysis View 产品化 收口
- 当前架构基线：`docs/architecture.md`（`Phase 15.2D` 为稳定代码基线；`15.2E` 已完成 feasibility assessment，editable/selectable input integration 继续 deferred）

## 当前目标（Execution Focus）

1. 以当前 `Phase 15.2D` stable baseline 为前提，确认 editable input integration 是否能在不破坏 typing/focus/selection contract 的情况下落地。
2. 若当前 Slint 版本缺少稳定 hook，则保持代码停留在 `15.2D` 基线，不为 `15.2E` 引入 overlay 拦截、私有事件链路或自写 caret/selection 逻辑。
3. 把可行性结论和后续队列写清楚，避免下一线程重复走高风险实现路径。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 基于当前 `15.2D` 代码确认 editable input integration 的真实依赖边界与风险
  - 评估 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入、`API Key`、以及 `SelectableSectionText`/`SelectableDiffText` 是否存在稳定低风险 hook
  - `docs/architecture.md` / `docs/thread-context.md` 与 assessment 结果对齐
  - 基线回归验证（`cargo check --workspace`、`cargo test -p fc-ui-slint`、`cargo run -p fc-ui-slint` smoke）
- Out of Scope：
  - IA 重置（`App Bar + Sidebar + Workspace` 保持不变）
  - runtime theme 切换、全量主题系统、全量 hex 清洗
  - `Provider Settings -> Settings` UI 升级
  - Tree explorer / compare-view dual mode
  - Compare View 新模式或目录树扩展
  - `fc-core` / `fc-ai` 合约改动
  - 全局 loading/theme/notification controller
  - 超出现有边界契约的 AI provider 架构扩展
  - 在当前 `slint = 1.8.0` 依赖上强行落地任何 editable input context-menu wiring
  - 通过 overlay `TouchArea`、私有事件链路、或自写 caret/selection/editing 逻辑硬接 `LineEdit` / `TextInput`

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
  - `15.2E` feasibility assessment 已完成：在 `slint = 1.8.0` 上保持 deferred，不落代码
  - 维护 `15.2D` stable baseline，不把 input/menu 生命周期或风险逻辑反向污染到主线
- Next：
  - 直接推进 phase 16，或仅在依赖升级后再重开 `15.2E`
  - 若未来重开 `15.2E`，优先验证是否已有类似 `ContextMenuArea` 的稳定 editable hook，再决定是否接入 `Compare Inputs` / `Search` / `Provider Settings`
  - 结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
- Later：
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `context-menu core` 必须保持 window-local；不要把 menu lifecycle 反向塞回 `AppState/Presenter`。
3. 在当前依赖版本下，不要再尝试通过 overlay `TouchArea`、私有事件拦截或自写编辑逻辑接 editable input surface。
4. 右键接线不能破坏 `Results / Navigator` 左键选择、Diff 行号双击复制、Analysis success 文本选择与滚动；`Risk Level` 保持 `Copy` 按钮-only，不再属于 menu safe surface。
5. `toast-controller` 仍是 overlay toast only；不要回退 15.2A 的边界。

## 验证命令（Verification Commands）

```bash
cargo check --workspace
cargo test -p fc-ui-slint
cargo run -p fc-ui-slint
```

UI 变更可加做：

```bash
cargo run -p fc-ui-slint
```

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 以当前 `Phase 15.1B fix-3` + `Phase 15.2A toast-controller overlay only` + `Phase 15.2B loading-mask(+sync projection fix)` + `Phase 15.2C ui_palette` + `Phase 15.2D menu core` 版本为基线。  
> 把 `15.2D` 视为当前稳定代码基线；`15.2E` 在 `slint = 1.8.0` 上已评估并保持 deferred。  
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
> 不要用 overlay `TouchArea`、私有事件链路或自写 caret/selection/editing 去硬接 editable inputs。  
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
