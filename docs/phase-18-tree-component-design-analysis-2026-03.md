# Phase 18 预研：tree component 技术设计分析报告

## 文档目的

- 本文档只做 `Phase 18` 的 tree component 技术设计分析，不直接进入实现。
- 分析基于当前项目真实基线，而不是最早 roadmap 的旧状态。
- 本文档的目标是帮助确认：
  - tree component 是否应独立实现
  - 现有平铺结果模型可以复用什么
  - tree node model 最小需要哪些字段
  - 搜索 / 状态过滤 / 定位并打开 / stale-selection 应如何接入
  - 哪些内容应放在 `Phase 18`，哪些应留给 `Phase 19+`

## 当前基线与前提

### 当前稳定基线

- `Sidebar` 四块 IA 已固定。
- `Results / Navigator` 当前稳定为 flat list。
- Results row 已采用 `filename-first + capability-first secondary + weak parent-path` 信息层级。
- Search contract 仍是 `path / name only`。
- `selection / stale-selection / unavailable / no selection` 语义已收口。
- `Settings` 已支持默认行为偏好第一轮模型，包含 `Hidden files`。
- tooltip 已收口为文本补全层。
- 当前 `Workspace / File View` 壳层稳定，不应在本次预研中顺手改动。

### 本次分析采用的已对齐产品前提

- 结果视图采用双视图并存：
  - 层级视图
  - 平铺视图
- 默认展示层级视图，但允许：
  - 在 `Settings` 中切换默认结果视图
  - 在 `Results / Navigator` 标题区即时切换
- 搜索结果不强行作用于树：
  - 搜索后进入独立平铺结果模式
  - 树视图主要负责比较与定位
- 状态过滤直接作用于树：
  - 只保留命中节点及必要祖先
  - 不显示完全没有命中的目录
- 右键菜单采用 `定位并打开`。
- 树结构采用 `Left / Right` 路径并集构建的合并树。
- 第一轮目录节点聚合信息保持极简：
  - 节点名
  - 展开/收起
  - 状态色 / pill
- tree view 应做成独立组件，不继续堆在 `app.rs`。
- 第一轮不做：
  - Kaleidoscope 式左右双树并排
  - Compare View / File View 双模式
  - 内容搜索
  - 字符级局部高亮
  - compare core 大改

## A. 当前基线 Review

### 1. Results / Navigator 的真实数据层

- UI 侧当前结果源模型是 `CompareEntryRowViewModel`：
  - `relative_path`
  - `status`
  - `detail`
  - `entry_kind`
  - `detail_kind`
  - `can_load_diff`
  - `diff_blocked_reason`
  - `can_load_analysis`
  - `analysis_blocked_reason`
- 该模型来自 `fc-core::CompareReport -> CompareEntry -> CompareEntryRowViewModel` 的 bridge 映射。
- 当前 flat 展示层并不是直接渲染 `CompareEntryRowViewModel`，而是再投影成 `NavigatorRowProjection`，补出：
  - `source_index`
  - `display_name`
  - `parent_path`
  - `secondary_text`
  - `tooltip_text`
  - `display_name_matches_filter`
  - `parent_path_matches_filter`

### 2. 当前搜索 / 过滤 / 隐藏文件 contract

- Search 仍然只基于 `relative_path.contains(query)`，也就是 `path / name only`。
- status filter 与 `Hidden files` 都作用于当前 visible set。
- 当前 `row_visible_in_results()` 是统一判断 visible set 的入口：
  - search
  - status filter
  - hidden-files preference
- 当前平铺列表的 scanability 由 `NavigatorRowProjection` 提供，而不是 compare core。

### 3. 当前 selection / stale-selection contract

- 当前选中 contract 是：
  - `selected_row: Option<usize>`
  - `selected_relative_path: Option<String>`
- `selected_row` 是 `entry_rows` 上的 source index。
- `selected_relative_path` 是 path-based anchor。
- 当当前 row 不再属于 visible set 时：
  - 清掉 `selected_row`
  - 保留 `selected_relative_path`
  - 进入 `stale-selection`
  - 不自动跳到其他 row
- compare rerun 后也只做同 path 的保守恢复；不可恢复时继续 stale。

### 4. 当前 compare core 对树的意义

- `fc-core` 当前已经按左右路径并集构建 compare entries，技术上已满足 merged tree 的基础输入。
- `fc-core` 也已经输出目录 entry，因此第一轮 tree 不需要先改 compare core 才能起结构树。
- 但当前 comparer 对“左右都存在的目录”直接标为 `Equal`，不会根据子节点差异向上汇总。
- 结论：
  - 树结构输入可直接复用 compare rows
  - 目录状态显示不能直接复用 compare core 的目录状态，必须在 presenter/state 层做聚合

## B. tree component 的建议拆分方案

### 1. 组件边界

- tree component 应作为独立 Slint 组件 / 组件组落地，而不是继续叠加在 `app.rs` 的内联 navigator 区块里。
- 但不建议把“递归树 + 聚合 + 过滤 + selection/stale-selection”直接塞进 Slint。
- 最稳的方案是分三层：
  - Rust canonical tree state
  - Rust visible tree row projection
  - Slint tree renderer

### 2. 推荐职责划分

- presenter/state 负责：
  - merged tree 构建
  - 目录状态聚合
  - 状态过滤剪枝
  - `expanded_paths`
  - view mode 决策（tree / flat / search-result flat）
  - locate-and-open
  - selection/stale-selection
- tree component 负责：
  - 渲染 tree rows
  - 缩进 / disclosure icon
  - hover / tooltip
  - 行级点击与右键入口
  - 向外抛出 `toggle(node_key)` / `select(source_index)` / `context_menu(node_key)`

### 3. 本地状态边界

- 适合留在 tree component 层的本地状态：
  - hover
  - pressed
  - tooltip anchor
  - 局部滚动位置
- 不适合留在 tree component 层的状态：
  - expanded/collapsed 持久状态
  - locate target
  - current selection
  - stale-selection
  - status filter 剪枝结果

### 4. 为什么要 flatten visible tree rows

- 当前 `MainWindow` 与 Rust 同步已经是多组平行 `VecModel` 数组，而不是嵌套模型。
- 当前 flat navigator 也是先在 Rust 侧投影，再在 Slint `ListView` 中渲染。
- tree 如果继续采用 Rust 先 flatten，再把可见 tree rows 交给 `ListView`，可以最大化复用当前窗口同步与测试方式。
- 结论：
  - canonical tree 可以是嵌套结构
  - UI 交给 Slint 的应是 flatten 后的 `TreeRowProjection`

## C. tree node model 建议

### 1. 推荐采用两层模型

- canonical tree node：持有结构和稳定状态
- visible tree row projection：持有当前渲染所需字段

### 2. canonical tree node 最小字段

```rust
struct TreeNodeVm {
    key: String,                    // normalized relative path; "" for synthetic root
    relative_path: String,
    kind: TreeNodeKind,             // Root | Directory | File | Symlink | Other
    display_name: String,
    parent_key: Option<String>,
    child_keys: Vec<String>,
    source_index: Option<usize>,    // back-reference into entry_rows
    expanded: bool,                 // presenter-owned
    base_status: TreeStatusToken,   // unfiltered aggregated status
}
```

### 3. visible tree row projection 最小字段

```rust
struct TreeRowProjection {
    key: String,
    source_index: Option<usize>,
    depth: u16,
    kind: TreeNodeKind,
    display_name: String,
    secondary_text: String,          // v1: leaf only, directory usually empty
    tooltip_text: String,            // full path completion
    display_status: TreeStatusToken, // filtered-view status
    matches_status_filter: bool,
    retained_for_descendant_match: bool,
    has_children: bool,
    is_expanded: bool,
    has_open_target: bool,
    is_locate_target: bool,          // transient, not persisted
}
```

### 4. 字段职责说明

- `key`
  - 稳定节点标识
  - 推荐直接使用 normalized relative path
- `source_index`
  - 连接回现有 `entry_rows`
  - file leaf 选中后可直接复用现有 `SelectRow + LoadSelectedDiff`
- `base_status`
  - 全量树下的 unfiltered aggregate status
- `display_status`
  - 当前过滤后的显示状态
  - 不建议和 `base_status` 混成一个字段
- `retained_for_descendant_match`
  - 目录在过滤下是否因为子节点命中而保留
  - 建议做成 derived flag，而不是新的业务状态机
- `is_locate_target`
  - 搜索结果回树时的瞬时定位强调
  - 建议做成 transient projection，不要持久化到 canonical node

### 5. 第一轮必要字段

- 必要：
  - `key`
  - `relative_path`
  - `kind`
  - `display_name`
  - `parent_key`
  - `child_keys`
  - `source_index`
  - `expanded`
  - `base_status`
  - `depth`
  - `display_status`
  - `retained_for_descendant_match`
  - `has_children`
  - `has_open_target`
- 可延后：
  - descendant counts
  - 目录统计摘要
  - 搜索高亮 spans
  - 目录 selection
  - dual-tree 对齐信息

## D. 关键行为设计建议

### 1. 默认展开策略

- 第一轮建议：
  - synthetic root 隐式展开
  - depth-1 目录默认展开
  - 更深层默认折叠
- 用户手动展开状态由 presenter/state 维护。
- `Locate and Open` 需要自动展开祖先链。
- active status filter 下，所有保留祖先链应自动展开，避免“命中了但用户看不见”。
- 不建议第一轮就做基于目录规模的智能展开策略。

### 2. 状态聚合规则

- 第一轮最小可用规则建议为：
  - 所有子状态一致，则目录沿用该状态
  - 子状态混合，则目录显示 `different`
  - 没有子节点时，回退到 direct/self status
- 这条规则最简单、最稳、最容易向后兼容。
- `pending / skipped` 可以继续保留为状态 token，但一旦和其他状态混合，直接落到 `different` 即可。

### 3. 状态过滤如何作用于树

- 先用 hidden-files preference 过滤掉基础输入里的隐藏路径。
- 再对树做 status filter：
  - file/special leaf 直接按自身状态命中
  - directory 用当前聚合状态判定
- 过滤后的可见规则：
  - 命中节点保留
  - 命中节点的祖先保留
  - 完全没有命中的目录不显示
- 建议目录在 projection 中带上 `retained_for_descendant_match`：
  - 便于后续做轻量视觉处理
  - 便于区分“目录自己命中”与“只是祖先保留”
- `display_status` 建议基于过滤后可见子树重新计算，而不是沿用 unfiltered `base_status`。

### 4. 搜索结果与树的关系

- 搜索应继续走 flat results mode，不应强行作用于树。
- 原因：
  - 当前搜索 contract 只有 `path / name only`
  - 当前搜索高亮是 row-local
  - 当前 tree 第一轮不做内容搜索、字符高亮、复杂祖先展开策略
- 继续复用当前 flat results 的好处：
  - 现有 `NavigatorRowProjection` 与 tooltip/highlight/scanability 逻辑可原样保留
  - 不需要给树额外引入搜索高亮与深层展开规则

### 5. “定位并打开” 的最小技术能力

- 从搜索结果返回树视图时，树至少需要：
  - `expand_to(relative_path)`
  - `select_by_source_index(relative_path/source_index)`
  - `mark_locate_target(relative_path)` 的瞬时能力
  - 触发既有 `LoadSelectedDiff`
- 第一轮推荐的动作顺序：
  - 退出搜索结果模式
  - 切回 tree view
  - 展开祖先链
  - 设置选中 leaf
  - 打开 file-level diff/view
- 右键菜单的 `Locate and Open` 最适合先加在搜索结果 flat mode 上。
- tree 内部本身已经有结构定位意义，第一轮不必给 tree row 再造第二个复杂右键语义。

### 6. 选中与 stale-selection 复用边界

- 当前 `selected_row + selected_relative_path` contract 基本可以原样复用。
- file leaf 选中时：
  - 写入 `selected_row`
  - 写入 `selected_relative_path`
  - 复用现有 `LoadSelectedDiff`
- directory node 第一轮不进入 file-view selection：
  - 点击目录只负责展开/收起
  - 不改变 `selected_row`
- 目录折叠不应触发 stale-selection：
  - stale 的定义应继续是“路径不属于当前 visible result set”
  - disclosure 折叠只是局部可视，不是过滤/作用域变化
- tree / flat 切换时应尽量复用既有 selection 逻辑，而不是重做新的状态机。

## E. 风险与复杂度判断

### 1. 最容易失控的点

- 把搜索语义强行塞进树。
- 试图在 Slint 里直接做递归 tree 数据结构与行为。
- 第一轮就让目录节点承载复杂摘要。
- 顺手大拆 `app.rs` 与整套 UI primitive。

### 2. 哪些内容会明显扩大 scope

- 搜索命中后自动展开整棵树并做树内高亮。
- 目录节点支持 selection 并进入右侧 file-view。
- 目录统计摘要、subtree counts、mixed-summary 文案。
- 左右双树并排。
- Compare View / File View 双模式。
- 任何 compare core 新 contract。

### 3. 建议明确 deferred

- 双树并排
- 内容搜索
- 字符级高亮
- 目录 selection / 目录详情面板
- 复杂目录统计
- compare core 扩张
- Kaleidoscope 式空间对齐与工作区演进

## F. 对 Phase 18A / 18B / 18C 的建议拆分

### 18A：正确性基线

- merged tree builder
- independent tree component
- flatten visible tree row projection
- hidden-files 兼容
- status filter 剪枝
- file leaf select/open
- search 非空时继续 flat fallback
- Results 区域 runtime tree/flat toggle

### 18B：模式联动与定位

- Settings 中默认结果视图持久化
- `Locate and Open`
- ancestor reveal
- tree / flat 切换时 selection 保持
- compare rerun 后 expanded-path pruning/restore

### 18C：收口与稳定性

- tooltip parity
- 定位后的滚动与可视区域收口
- presenter/state tests 补齐
- 大目录性能与视觉 polish

### 必要调整建议

- 如果现有草案把“状态过滤剪枝”放得比 tree 首版更后，建议前移到 `18A`。
- 如果现有草案打算在 `18A` 就连同 Settings 默认结果视图一起做，建议后移到 `18B`。
- tree correctness 不应被 Settings 扩张阻塞。

## G. 文档判断

- 本次没有更新 `docs/architecture.md`。
- 原因：
  - `docs/architecture.md` 当前刻意描述的是 `Stable Baseline before Phase 18`
  - 它仍然准确描述了当前真实代码基线
- 但需要明确：
  - 一旦 `Phase 18` 最终草案确认并进入实施
  - `architecture.md` 中关于 `Phase 18 Should Not Mix In tree/group navigation` 的表述需要同步改写
  - 否则未来线程会被旧边界误导

## H. 工作区状态

- 本轮执行前已检查工作区。
- 实际仓库根目录为 `/Users/xuwende/code/rust/compare_rs/folder-compare`。
- 分支为 `dev`。
- 执行前未发现未 commit 改动，因此本轮可以继续执行。

## 本轮结论摘要

- tree component 应独立实现，但树逻辑不应塞进 Slint 本地状态。
- 最小可落地路径是：
  - 继续复用 `CompareEntryRowViewModel` 作为源输入
  - 新增 Rust 侧 merged tree + visible row projection
  - Slint 侧只渲染 flatten 后的 tree rows
- 搜索继续走 flat mode。
- 状态过滤直接剪树。
- selection / stale-selection 继续复用现有 contract。
- 第一轮不应被双树、目录摘要、内容搜索、compare core 扩张带偏。

## 验证说明

- 本次仅新增文档。
- 本次未运行：
  - `cargo test --workspace`
  - `cargo run -p fc-ui-slint`
