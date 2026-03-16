# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-16）

- 轮次定义：`Phase 15.2D: menu core + non-input safe surfaces`（基线：`Phase 15.2A toast-controller overlay only` + `Phase 15.2B loading-mask(+sync projection fix)` + `Phase 15.2C ui_palette` 已落地代码）。
- 改了什么：
  - 在 `fc-ui-slint` 内新增共享 `context-menu core`（window-local controller + shared menu surface），不把短生命周期菜单状态塞回 `AppState/Presenter`；
  - 接入 `Results / Navigator` item、`Workspace` file context header、`Analysis` success section chrome（`Summary` / `Core Judgment` / `Key Points` / `Review Suggestions` / `Notes`）的右键 `Copy / Copy Summary`；
  - `Risk Level` 在本阶段仅保留显式 `Copy` 按钮，不再提供右键菜单；
  - 增加 target/context token、anchor positioning、action dispatch 和自动关闭策略（tab 切换 / compare 重跑 / selected row 变化 / action 执行后 / 外部点击 / `Results` 与 Analysis success 滚动）；
  - 修正 `AnalysisSectionPanel` 的 anchor 坐标归一化，避免 Analysis section 菜单错误吸附到顶部；
  - 保持 `toast-controller` 仍为 overlay toast only，不回退为 banner 或全局通知中心。
- 为什么影响下一线程：`15.2D` 已提供未来 input integration 可复用的最小 menu core；如果跳过 `15.2E`，主线也可以直接推进 phase 16，不再依赖 editable input 方案。
- 保持不变：IA 仍是 `App Bar + Sidebar + Workspace`；`Diff/Analysis` shell、connected tabs、loading scope boundary（`running`/`diff_loading`/`analysis_loading`）、`SelectableSectionText`/`SelectableDiffText`、以及所有输入控件绑定结构均不改。

## 快照（Snapshot）

- 日期：2026-03-16（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（`fc-ui-slint` `context-menu core` + docs 对齐，待本线程提交）
- 最近提交：
  - `6afab36` phase 15.1B fix-3：Analysis selectable text（success sections only）
  - `8d932c1` phase 15.1B fix2: analysis success cannot scroll
  - `19388d5` Phase 15.1B fix1 ：Analysis View 产品化 收口
- 当前架构基线：`docs/architecture.md`（`Phase 15.2D` 已补齐本地 shared context-menu core baseline；editable input integration 仍 deferred）

## 当前目标（Execution Focus）

1. 以当前 `Phase 15.2A/15.2B/15.2C` 为稳定基线，维持 `Diff/Analysis` shell 与 local controller 边界。
2. 在 `fc-ui-slint` 内落地共享 `context-menu core`，仅覆盖 non-input safe surfaces。
3. 确保 `15.2D` 自身可独立成立；即使 `15.2E` 不做，也不阻塞 phase 16。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `fc-ui-slint` 内共享 `context-menu core`（open/close、action dispatch、anchor positioning、target token）
  - `Copy / Copy Summary` 公共动作与最小格式化 helper
  - safe surface 接入：`Results / Navigator` item、`Workspace` file context header、`Analysis` success section chrome（`Summary` / `Core Judgment` / `Key Points` / `Review Suggestions` / `Notes`）
  - `docs/architecture.md` / `docs/thread-context.md` 与当前 phase 事实对齐
  - 最小回归验证（`cargo check --workspace`、`cargo test -p fc-ui-slint`、`cargo run -p fc-ui-slint` smoke）
- Out of Scope：
  - IA 重置（`App Bar + Sidebar + Workspace` 保持不变）
  - runtime theme 切换、全量主题系统、全量 hex 清洗
  - `Provider Settings -> Settings` UI 升级
  - Tree explorer / compare-view dual mode
  - Compare View 新模式或目录树扩展
  - `fc-core` / `fc-ai` 合约改动
  - 全局 loading/theme/notification controller
  - 超出现有边界契约的 AI provider 架构扩展
  - 所有 editable input context-menu integration（`LineEdit` / `TextInput` / `SelectableSectionText` / `SelectableDiffText` / `Compare Inputs` / `Filter / Scope` / `Provider Settings`）

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
  - `Phase 15.2D` shared context-menu core 收口已完成，待确认是否直接推进 `15.2E`
  - 验证滚动自动关闭、Analysis section anchor 和 copy-family helper 不污染主线状态机
- Next：
  - 评估是否完全跳过 `15.2E` 直接推进 phase 16；若做 `15.2E`，仅作为 isolated editable-input pass
  - 结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
  - 评估 Analysis success 文本选择的 native copy 快捷键回调可行性（若 Slint 提供稳定 hook 再补 toast）
- Later：
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `context-menu core` 必须保持 window-local；不要把 menu lifecycle 反向塞回 `AppState/Presenter`。
3. 不要误接 editable input surface，尤其 `SelectableSectionText` / `SelectableDiffText` / `LineEdit` / `TextInput`。
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
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
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
