# Folder Compare Thread Context (Post-Phase 18A Round 1)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前真实事实、当前边界、下一线程入口。
- 当前主参考是 `docs/architecture.md`；本文件只做压缩版 handoff，不替代架构文档。

## 本轮更新说明（2026-03-22）

- `Phase 18A` 第一轮实现已落地到代码，不再是纯文档启动状态。
- `Results / Navigator` 已从 flat-only 稳定基线进入 `tree + flat` 双视图运行时基线。
- `docs/architecture.md` 已同步到“stable baseline + implemented `Phase 18A` baseline”口径。
- `README.md` 已做最小必要同步，避免继续写成“Phase 18A 启动前”状态。

## 快照（Snapshot）

- 日期：`2026-03-22`（Asia/Shanghai）
- 分支：`dev`
- 当前真实代码基线：
  - `Phase 17D` 稳定 shell / window / settings / tooltip / file-view contract
  - `Phase 18A` 第一轮 correctness baseline 已实现
- 本轮已运行验证：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`

## 当前真实稳定基线（Through Phase 17D + Phase 18A Round 1）

### 工具链与版本

- 稳定演进基线：`0.2.18` 后续
- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`

### 稳定 shell / IA / 平台 contract

- Sidebar 四块 IA 继续固定：
  - `Compare Inputs`
  - `Compare Status`
  - `Filter / Scope`
  - `Results / Navigator`
- Workspace 继续固定：
  - `Tabs`
  - `Header`
  - `Content`
- `Diff / Analysis` 继续共用稳定 file-view shell。
- macOS immersive title bar / non-mac legacy App Bar / `window_chrome` contract 继续稳定。

### 现已落地的 `Phase 18A` 事实

- `Results / Navigator` 现已支持双视图：
  - 非搜索状态默认 `tree mode`
  - `flat mode` 继续存在
- 搜索 contract 仍是 `path / name only`，搜索非空时强制 flat results mode。
- Tree 相关逻辑在 Rust presenter/state：
  - canonical merged tree
  - filtered visible tree rows flatten
  - expansion state
  - selection/stale-selection 决策
- Slint 侧只有独立 tree renderer，不持有递归 tree state。
- 目录节点点击只负责展开/收起，不进入 file-view selection。
- 文件 leaf 节点点击复用既有 `selected_row / load diff / load analysis` 链路。
- status filter 对 tree 做剪枝，并保留必要祖先。
- 目录 `display_status` 基于过滤后可见子树重算。
- `Hidden files` 继续是 UI / presentation preference，不下沉到 `fc-core`，并已与 tree mode 正确兼容。
- tree / flat 切换遵循保守 selection contract。
- 折叠包含当前打开文件的目录不会触发 false stale-selection；只有 membership 真变化时才 stale。

## 当前执行焦点（Execution Focus）

- 当前不再是“启动 `Phase 18A`”。
- 当前真实焦点是：
  - 以已落地的 `Phase 18A` round 1 baseline 为前提继续工作
  - 优先做 smoke、边界收口、必要的小修正
  - 或在确认边界后再进入 `18B`
- 不要回退到“flat-only baseline”叙事，也不要把本轮已实现内容继续当 proposal。

## 仍然明确未做（Out of Scope / Deferred）

- `Settings` 持久化默认结果视图
- `Locate and Open`
- ancestor reveal / auto reveal / auto scroll
- expanded-path restore / pruning 策略
- 目录进入右侧 file-view selection
- tree 内搜索 / 内容搜索 / match-span 高亮
- 目录 secondary text / descendant counts / summary
- Compare View / File View workspace 重构
- `fc-core` compare contract widening

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider concerns 隔离。
2. `fc-ai` 是可选解释层；compare 输出在无 AI 情况下也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不改写 compare core 语义。
4. Sidebar 继续保持四块 IA；Workspace 继续保持 attached `Diff / Analysis` shell。
5. `Compare Status` 继续 summary-first。
6. Flat results row 的 `filename-first + capability-first + weak parent-path` contract 继续有效。
7. 搜索 contract 继续是 `path / name only`；tree 不承接搜索表达。
8. tooltip 继续是 completion-first / restrained-hint-first，不演化成说明系统。
9. `Hidden files` 继续是 UI preference，不推进到 compare/core policy。
10. 不回退 macOS immersive title bar / non-mac legacy top bar / `window_chrome` split。
11. 不回退 event-driven sync、persistent `VecModel`、shared typography tokens 等 supporting baseline。

## 当前关键文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `docs/phase-18-tree-component-design-analysis-2026-03.md`
4. `README.md`
5. `crates/fc-ui-slint/src/navigator_tree.rs`
6. `crates/fc-ui-slint/src/navigator_tree.slint`
7. `crates/fc-ui-slint/src/state.rs`
8. `crates/fc-ui-slint/src/presenter.rs`
9. `crates/fc-ui-slint/src/app.rs`
10. `crates/fc-ui-slint/src/commands.rs`

## 验证（Verification）

- 本轮代码与文档同步完成后已运行：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 自动验证已覆盖 presenter/state/tree 的核心 contract。
- UI 侧仍建议人工 smoke：
  - tree / flat toggle
  - 搜索强制 flat fallback
  - tree 中目录 toggle 与文件 leaf open
  - status filter / hidden-files 在多层目录上的显示

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 把 `Phase 17D` 之前的 shell / settings / tooltip / file-view contract 视为稳定基线。  
> 把 `Phase 18A` 第一轮实现视为已落地事实，而不是 proposal：`Results / Navigator` 已有 tree + flat 双视图；非搜索默认 tree；搜索非空强制 flat；tree logic 在 Rust presenter/state；目录节点只 toggle；文件 leaf 才进入右侧 file-view。  
> 不要顺手重开 `fc-core` contract、window system、Settings persistence、directory selection、tree search、Locate and Open，除非当前线程目标明确要求。  
> 如果继续 `18A` 收口，优先检查 smoke 与小范围 contract 修正；如果进入 `18B`，保持 locate/persist/reveal 范围独立，不要和现有稳定 shell 混改。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件只记录当前事实和下一线程入口；不要把未经确认的未来能力写成已实现。
