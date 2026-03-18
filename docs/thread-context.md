# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-18）

- 轮次定义：`Dependency upgrade executed through Phase 15.4; Phase 15.5 preparation on upgraded baseline`。
- 改了什么：
  - 实际完成 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`，不再停留在“升级路线已接受、尚未执行”的状态；
  - workspace 依赖与工具链现已收敛到 `rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`；
  - 版本号单一事实来源已落到 workspace `Cargo.toml`，`docs/macos_dmg.sh` 改为从 manifest 派生 bundle / DMG / ZIP 版本；
  - macOS arm64 人工 smoke 已通过，未发现回归；同时观察到 diff 加载性能体感明显提升；
  - 后续 phase 将按“执行同时更新主文档”的方式推进，临时 `docs/phase-15-upgrade-checklists.md` 不再保留。
- 为什么影响下一线程：如果下一线程仍按旧假设重复 `15.3A/15.3B/15.4`，或继续把当前基线当作 `slint = 1.8.0`，会重复做已完成工作并误判 `15.5` 的起点；如果跳过 `15.5/15.6` 直接做 `Phase 16`，会再次把输入菜单补票、同步清理、导航增强混成一轮。
- 保持不变：`15.2D` 的 IA 与 shell contract 不变；`15.2E` 仍未落代码；`Diff/Analysis` shell、connected tabs、loading scope boundary、`SelectableSectionText` / `SelectableDiffText`、modal draft 行为、以及本地 `toast/loading/menu` controller 边界均不改；workspace `edition` 仍是 `2021`；UI 仍使用内联 `slint::slint!`，`50ms` 轮询仍保留并留待 `Phase 15.6` 处理。

## 快照（Snapshot）

- 日期：2026-03-18（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（依赖升级已执行到 `Phase 15.4`，主文档同步到新基线）
- 最近提交：
  - `6afab36` phase 15.1B fix-3：Analysis selectable text（success sections only）
  - `8d932c1` phase 15.1B fix2: analysis success cannot scroll
  - `19388d5` Phase 15.1B fix1 ：Analysis View 产品化 收口
- 当前架构基线：`docs/architecture.md`（`15.2D` 行为已在 `rust 1.94.0 + slint 1.15.1` 基线上恢复等价；`Phase 15.5` 是下一默认执行目标）

## 当前目标（Execution Focus）

1. 以升级后的稳定基线为前提，下一步默认执行 `Phase 15.5`：完成 editable input context-menu integration（`15.2E`）。
2. 保持 phase train 分层：`Phase 15.5` -> `Phase 15.6` -> `Phase 16`，不回头重做 `15.3A/15.3B/15.4`，也不跳过 `15.5/15.6` 直接做导航增强。
3. 后续每个 phase 执行时同步更新 `architecture.md`、`thread-context.md`、`upgrade-plan-rust-1.94-slint-1.15.md`，不再创建额外 phase checklist 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 以 `rust 1.94.0 + slint 1.15.1` 为基线，推进 `Phase 15.5` 的输入菜单接入
  - 后续阶段执行时同步更新三份主文档
  - 继续维持 `15.x` 已收敛的 shell/menu/loading/toast 边界
- Out of Scope：
  - 重复执行 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`
  - 直接推进 `Phase 16`
  - 在同一轮里同时做 `Phase 15.5`、`Phase 15.6`、`Phase 16`
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
6. 依赖升级路线已完成到 `Phase 15.4`；后续必须按 `Phase 15.5` -> `Phase 15.6` -> `Phase 16` 推进，并在同一轮同步主文档。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`：当前执行上下文与交接清单
2. `docs/architecture.md`：长期架构契约与 deferred decisions
3. `docs/upgrade-plan-rust-1.94-slint-1.15.md`：升级执行结果、剩余阶段边界、Codex 提示词
4. `crates/fc-ui-slint/src/app.rs`：UI shell、modal、sync、callbacks
5. `crates/fc-ui-slint/src/context_menu.rs`：window-local context-menu core 与 safe-surface 边界
6. `crates/fc-ui-slint/src/presenter.rs`：状态编排与命令流
7. `crates/fc-ui-slint/src/state.rs`：UI state machine 与派生展示字段
8. `crates/fc-ui-slint/src/settings.rs`：Provider Settings 持久化与 API Key 相关约束
9. `Cargo.toml`、`rust-toolchain.toml`、`docs/macos_dmg.sh`：当前版本基线与打包版本来源

## 当前工作队列（Active Work Queue）

- Now：
  - `Phase 15.3A` / `15.3B` / `15.4` 已完成并通过 smoke
  - 当前稳定基线是 `15.2D` 行为等价 + `rust 1.94.0 + slint 1.15.1`
  - 下一默认工作是 `Phase 15.5`
- Next：
  - `Phase 15.5`：在新基线上重开并完成 `15.2E`
  - `Phase 15.6`：同步与 model churn 清理
- Later：
  - `Phase 16`：结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
  - `edition = "2024"`：单列里程碑，不并入当前 phase
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `context-menu core` 必须保持 window-local；不要把 menu lifecycle 反向塞回 `AppState/Presenter`。
3. `Phase 15.5` 必须优先使用 Slint 原生 editable-input surface；不要回退到 overlay 拦截、私有事件链路或自写编辑逻辑。
4. 输入菜单接线不能破坏 typing、focus、selection、paste、cut、select-all contract；`API Key` 继续保持保守策略。
5. 右键接线不能破坏 `Results / Navigator` 左键选择、Diff 行号双击复制、Analysis success 文本选择与滚动；`Risk Level` 仍保持 `Copy` 按钮-only，除非文档 contract 被显式更新。
6. `toast-controller` 仍是 overlay toast only，`loading-mask` 仍保持当前范围；不要因为输入菜单或同步清理而回退这些边界。
7. 不要把 `Phase 15.5`、`Phase 15.6`、`Phase 16` 混成同一轮；也不要重新开临时 checklist 文档绕开主文档同步。

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
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4` 视为已完成。  
> 把当前基线视为：`15.2D` 行为已在 `rust 1.94.0 + slint 1.15.1` 上恢复等价。  
> 下一步默认从 `Phase 15.5` 开始，不要回头重做升级，也不要直接推进 `Phase 16`。  
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
> 不要把 `Phase 15.5`、`Phase 15.6`、`Phase 16` 混在同一轮。  
> 不要用 overlay `TouchArea`、私有事件链路或自写 caret/selection/editing 去硬接 editable inputs。  
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
