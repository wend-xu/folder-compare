# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的快速交接，定位是短周期执行上下文，不替代长期架构文档。

## 本轮更新说明（2026-03-17）

- 轮次定义：`Dependency upgrade roadmap alignment and Phase 15.3A preflight sequencing on top of stable 15.2D baseline`。
- 改了什么：
  - 在 `docs/architecture.md` 中正式落下依赖升级路线：`Phase 15.3A` -> `Phase 15.3B` -> `Phase 15.4` -> `Phase 15.5` -> `Phase 15.6` -> `Phase 16`；
  - 新增 `docs/upgrade-plan-rust-1.94-slint-1.15.md`，把目标版本、修改面、风险、人工验收、Codex 提示词拆成独立升级方案；
  - 把本文件从“`15.2E` feasibility assessment 线程”切换到“升级计划已接受、下一步先做 `Phase 15.3A` preflight”的执行上下文；
  - 明确当前主线不再默认“直接推进 phase 16”，而是先完成升级前收口、Rust 升级、Slint 升级，再在新基线上重开 `15.2E`。
- 为什么影响下一线程：如果下一线程仍按旧假设直接做 `Phase 16` 或在 `slint = 1.8.0` 上强行落 `15.2E`，会与当前 accepted roadmap 冲突，并再次把依赖问题和产品问题混在一起。
- 保持不变：`15.2D` 代码基线不变；IA 仍是 `App Bar + Sidebar + Workspace`；`Diff/Analysis` shell、connected tabs、loading scope boundary、`SelectableSectionText`/`SelectableDiffText`、所有输入绑定结构、modal draft 行为、以及本地 `toast/loading/menu` controller 边界均不改；本轮仍未实际执行 Rust/Slint 升级。

## 快照（Snapshot）

- 日期：2026-03-17（Asia/Shanghai）
- 分支：`dev`
- 工作区：有改动（docs 对齐依赖升级路线与 `Phase 15.3A` preflight；代码保持 `15.2D` 基线）
- 最近提交：
  - `6afab36` phase 15.1B fix-3：Analysis selectable text（success sections only）
  - `8d932c1` phase 15.1B fix2: analysis success cannot scroll
  - `19388d5` Phase 15.1B fix1 ：Analysis View 产品化 收口
- 当前架构基线：`docs/architecture.md`（`Phase 15.2D` 为稳定代码基线；依赖升级路线已接受但尚未执行；`15.2E` 保持 deferred，计划在 `Phase 15.5` 于 `slint = 1.15.x` 基线上重开）

## 当前目标（Execution Focus）

1. 以当前 `Phase 15.2D` stable baseline 为前提，先执行 `Phase 15.3A`：统一版本来源、升级 checklist、文档和交接口径。
2. 在 `Phase 15.3A` 与 `Phase 15.3B` 完成前，不直接推进 `Phase 16`，也不在 `slint = 1.8.0` 上重开 `15.2E`。
3. 保持升级路线分层：先 Rust `1.94.0`，再 Slint `1.15.x`，再在新基线上完成 editable input integration，并把后续清理与 `Phase 16` 拆开。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 依赖升级路线对齐：`architecture.md`、`thread-context.md`、`upgrade-plan-rust-1.94-slint-1.15.md`
  - `Phase 15.3A` preflight：版本来源统一、升级 checklist、smoke checklist、handoff 约束
  - 明确 `Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.6`、`Phase 16` 的边界与前后依赖关系
  - 文档与当前 `15.2D` 稳定代码基线对齐
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
  - 直接推进 `Phase 16`
  - 在同一轮里同时做 Rust/Slint 升级与 `edition = "2024"` 迁移
  - 通过 overlay `TouchArea`、私有事件链路、或自写 caret/selection/editing 逻辑硬接 `LineEdit` / `TextInput`

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI/网络/provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration/presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是当前 accepted baseline，同一时刻仅一个主分支激活。
5. Compare Status 保持 summary-first，不演化为重型第二详情面板。
6. 依赖升级路线必须按 accepted phase train 推进，不把 `Phase 15.4`、`Phase 15.5`、`Phase 15.6`、`Phase 16` 的目标重新糊成一个大版本。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`：当前执行上下文与交接清单
2. `docs/architecture.md`：长期架构契约与 deferred decisions
3. `docs/upgrade-plan-rust-1.94-slint-1.15.md`：独立升级方案、版本线、人工验收、Codex 提示词
4. `Cargo.toml` 与 `rust-toolchain.toml`：workspace 版本基线与工具链入口
5. `docs/macos_dmg.sh`：当前 bundle / DMG 版本来源
6. `crates/fc-ui-slint/src/app.rs`：UI shell、modal、sync、callbacks
7. `crates/fc-ui-slint/src/presenter.rs`：状态编排与命令流
8. `crates/fc-ui-slint/src/state.rs`：UI state machine 与派生展示字段
9. `crates/fc-ui-slint/src/bridge.rs`：UI 与 core/ai API 的映射边界

## 当前工作队列（Active Work Queue）

- Now：
  - `15.2E` feasibility assessment 已完成：在 `slint = 1.8.0` 上保持 deferred，不落代码
  - 依赖升级路线已接受：下一步先做 `Phase 15.3A` preflight，不直接写升级代码
  - 维护 `15.2D` stable baseline，不把 input/menu 生命周期或风险逻辑反向污染到主线
- Next：
  - `Phase 15.3A`：版本来源统一、文档对齐、升级 checklist、smoke checklist
  - `Phase 15.3B`：只升级 Rust 到 `1.94.0`，保持 `slint = 1.8.0`
  - `Phase 15.4`：升级到 `slint = 1.15.x`，先恢复 `15.2D` 行为等价
  - `Phase 15.5`：在新基线上重开并完成 `15.2E`
- Later：
  - `Phase 15.6`：同步与 model churn 清理
  - `Phase 16`：结果导航效率迭代（sorting / quick jump / filter ergonomics，限定在当前 IA）
  - 承接 `docs/architecture.md` 中 deferred 的 provider hardening 与 global notification orchestration

## 已知风险与评审重点（Known Risks / Review Focus）

1. 不要破坏已验收的 connected tabs / workbench seam / shell hierarchy。
2. `context-menu core` 必须保持 window-local；不要把 menu lifecycle 反向塞回 `AppState/Presenter`。
3. 在当前依赖版本下，不要再尝试通过 overlay `TouchArea`、私有事件拦截或自写编辑逻辑接 editable input surface。
4. 右键接线不能破坏 `Results / Navigator` 左键选择、Diff 行号双击复制、Analysis success 文本选择与滚动；`Risk Level` 保持 `Copy` 按钮-only，不再属于 menu safe surface。
5. `toast-controller` 仍是 overlay toast only；不要回退 15.2A 的边界。
6. 不要跳过 `Phase 15.3A` / `Phase 15.3B` 直接做 `Phase 15.4` 或 `Phase 16`；否则问题定位会重新混乱。
7. 不要在 Rust/Slint 升级同一轮里顺手切 `edition = "2024"`；edition 迁移应单列。

## 验证命令（Verification Commands）

```bash
cargo check --workspace
cargo test --workspace
cargo run -p fc-ui-slint
```

文档 / preflight 线程可按需降级为：

```bash
cargo check --workspace
```

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 再阅读 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。  
> 以当前 `Phase 15.1B fix-3` + `Phase 15.2A toast-controller overlay only` + `Phase 15.2B loading-mask(+sync projection fix)` + `Phase 15.2C ui_palette` + `Phase 15.2D menu core` 版本为基线。  
> 把 `15.2D` 视为当前稳定代码基线；依赖升级路线已接受，但尚未执行。  
> 下一步默认从 `Phase 15.3A` 开始，不要直接推进 `Phase 16`，也不要在 `slint = 1.8.0` 上重开 `15.2E`。  
> 保持当前 IA 与 phase 边界。  
> 不要回退 Diff/tabs/Analysis shell 收敛结果，也不要把本地 toast/loading/menu controller 重新塞进 `AppState/Presenter`。  
> 不要把 Rust/Slint 升级和 `edition = "2024"` 迁移混在同一轮。  
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
