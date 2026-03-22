# Folder Compare Thread Context (Phase 18A Start)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前稳定事实、当前执行焦点、硬边界、下一线程入口。
- 当前主参考是 `docs/architecture.md` 的 stable baseline + `Phase 18` activation summary；本文件负责把它压缩成可直接开工的线程上下文。

## 本轮更新说明（2026-03-22）

- 本轮完成了 `Phase 18A` 启动前的文档对齐：
  - `docs/architecture.md` 已从单纯的 pre-`Phase 18` baseline summary 更新为 stable baseline + `Phase 18` activation 文档
  - 本文件已同步为 `Phase 18A` 启动入口
  - `README.md` 已做最小必要同步，避免继续把 flat-only 旧口径写成当前前进方向
- 本轮没有进行任何 Rust / Slint 代码实现。
- 已确认：早期关于“tree / grouping 不在范围内”的表述，只能作为 pre-`18A` 稳定基线说明，不能再作为当前实现边界。

## 快照（Snapshot）

- 日期：`2026-03-22`（Asia/Shanghai）
- 分支：`dev`
- 当前工作区：`Phase 18A` 启动前文档对齐
  - `docs/architecture.md`
  - `docs/thread-context.md`
  - `README.md`
- 当前真实稳定基线：`Phase 17D` 已收口完成，后续工作从该基线进入 `Phase 18A`

## 当前真实稳定基线（Through Phase 17D）

### 工具链与版本

- 稳定演进基线：`0.2.18` 后续
- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`

### 稳定产品结构

- Sidebar 四块 IA 已固定：
  - `Compare Inputs`
  - `Compare Status`
  - `Filter / Scope`
  - `Results / Navigator`
- Workspace 当前稳定结构：
  - `Tabs`
  - `Header`
  - `Content`
- `Diff / Analysis` 共用稳定 file-view shell。

### 稳定 contract

- Results row：
  - `filename-first`
  - `capability-first` 次信息
  - weak `parent-path`
- Search contract：`path / name only`
- tooltip：文本补全层，不是说明系统
- Settings：全局设置骨架，不是完整 settings framework
- `Hidden files`：UI / presentation preference，不改 compare core 语义
- `stale-selection / unavailable / no-selection / waiting / success` 基础状态语义已收口

### 平台窗口层基线

- macOS：immersive title bar
- non-mac：legacy App Bar
- 当前窗口层 contract 已稳定：
  - 不使用 `no-frame`
  - 平台分支收口在 `window_chrome`
  - 顶部拖拽只在 macOS immersive strip 空白区显式触发

## 当前执行焦点（Execution Focus）

- 当前准备启动的是 `Phase 18A`，不是继续回头清扫 `Phase 16` / `Phase 17`。
- `Phase 18` 的一句话定义：
  - 在当前稳定的平铺结果列表基础上，引入基于 Left / Right 路径并集构建的层级结果视图，使用独立 tree component 承载目录层级、节点展开与状态表达；同时保留平铺结果视图承载搜索结果与集中扫描，并为后续 Compare View 提供可复用的数据表达基础。
- 当前最重要的实现约束：
  - 不要把 tree 逻辑塞进 Slint 内部
  - Rust 侧构建 merged tree + flattened visible rows
  - 搜索仍走 flat mode
  - `fc-core` 语义不因 tree 而改写
  - Sidebar / Workspace / window-layer 不重做 IA

## 当前已确认的 `Phase 18A` 边界

### In Scope

- 双视图并存：
  - tree view
  - flat view
- 非搜索状态默认 tree mode
- 搜索非空时强制 flat results mode
- `Results / Navigator` 标题区提供 runtime tree/flat toggle
- merged tree 由 Left / Right 路径并集构建
- 状态过滤直接作用于树剪枝，并保留必要祖先
- 目录节点第一轮保持极简表达
- file leaf 继续复用既有 selected/open/load diff 链路

### Out of Scope

- `Settings` 中默认结果视图持久化
- `Locate and Open`
- auto reveal / auto scroll / locate highlight polish
- 目录节点进入右侧 file view
- 目录统计摘要、descendant counts、复杂 secondary text
- 内容搜索、tree 内高亮、match-span 语义
- Compare View / File View workspace redesign
- compare core contract widening

## `Phase 18A` 五个已确认决策

1. 非搜索状态默认 tree mode；搜索状态强制 flat mode。
2. 目录节点第一轮不进入 file-view selection；点击目录只负责展开/收起。
3. status filter 下目录 `display_status` 必须基于过滤后可见子树重算。
4. tree / flat 运行时切换遵循保守 selection contract：
   - 目标模式可见则保留打开并映射高亮
   - 不可见则进入既有 `stale-selection`
5. 目录节点第一轮不做 secondary text / 复杂统计摘要。

## 当前人工验收快照草案（Phase 18A Draft）

- 默认进入非搜索 tree mode 时：
  - synthetic root 隐式展开
  - depth-1 目录默认展开
- 目录节点点击：
  - 只展开/收起
  - 不改变右侧 file-view 选中态
- 文件叶子点击：
  - 沿用现有 `selected_row / selected_relative_path / load diff` 链路
- 搜索框非空：
  - 强制切到 flat results mode
  - 结果继续沿用当前 flat results 的 scanability / tooltip / highlight contract
- 清空搜索：
  - 返回当前非搜索运行时模式
- status filter：
  - 只保留命中节点及必要祖先
  - 目录显示状态基于过滤后可见子树重算
- tree / flat 运行时切换：
  - 可见则保留打开
  - 不可见则进入 `stale-selection`
  - 第一轮不要求 auto reveal / auto scroll

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不承载 compare core 语义改写。
4. Sidebar 继续保持四块 IA；Workspace 继续保持 attached `Diff / Analysis` shell。
5. `Compare Status` 保持 summary-first，不演化成第二个重型详情面板。
6. Results row 的 `filename-first + capability-first + weak parent-path` contract 继续有效。
7. 搜索 contract 继续是 `path / name only`；tree 第一轮不承接搜索表达。
8. tooltip 保持 completion-first / restrained-hint-first，不演化成说明系统。
9. `Hidden files` 继续是 UI preference，不推进到 compare/core policy。
10. 不回退 macOS immersive title bar / non-mac legacy top bar / `window_chrome` platform split 的当前 contract。
11. 不回退 event-driven sync、persistent `VecModel`、shared typography tokens 等 supporting baseline。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `docs/phase-18-tree-component-design-analysis-2026-03.md`
4. `README.md`
5. `crates/fc-ui-slint/src/app.rs`
6. `crates/fc-ui-slint/src/presenter.rs`
7. `crates/fc-ui-slint/src/settings.rs`
8. `crates/fc-ui-slint/src/window_chrome.rs`
9. `Cargo.toml`
10. `rust-toolchain.toml`

## 验证（Verification）

- 本轮是文档同步任务。
- 本轮未运行：
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo run -p fc-ui-slint`

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`，必要时再看 `docs/phase-18-tree-component-design-analysis-2026-03.md` 与 `README.md`。  
> 把 `Phase 16` 到 `Phase 17D`、以及 workspace `edition = "2024"` 里程碑，全部视为已完成稳定基线。  
> 把当前真实产品/UI/平台 contract 视为 `Phase 17D` 后稳定基线：Sidebar 四块 IA、attached `Diff / Analysis` shell、filename-first results rows、explicit stale-selection 语义、tooltip completion boundary、`Settings -> Provider / Behavior`、`Hidden files` UI preference boundary、macOS immersive title bar / non-mac legacy top bar。  
> 当前准备启动的是 `Phase 18A`：在 `Results / Navigator` 内引入基于左右路径并集的 tree view，并保留 flat view；搜索非空时强制 flat mode；tree logic 放在 Rust presenter/state，不要塞进 Slint 递归状态里。  
> 不要顺手重开 IA、window-system、full settings framework、compare-level hidden policy、或已接受 baseline。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件只记录当前事实和下一线程入口；不要把未经确认的未来能力写成已实现。
