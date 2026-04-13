# Folder Compare Thread Context (Phase 19J landed baseline)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前真实事实、当前边界、下一线程入口。
- 当前主参考是 `docs/architecture.md`；本文件只做压缩版 handoff，不替代架构文档。

## 本轮更新说明（2026-04-13）

- 当前真实代码基线已明确包含：
  - `Phase 18` closeout（含 `18C fix-1`）
  - `Phase 19A` foundation-first 落地
  - `Phase 19B fix-2` compare-tree MVP 收口并通过当前验收
  - `Phase 19C fix-1` Compare workspace 收边 + top-level sidebar hide / restore 已落地
  - `Phase 19D` outer workspace session tabs 已落地
  - `Phase 19D fix-1` session reset semantics 已落地
  - `Phase 19E` true file compare view MVP 已落地
  - `Phase 19F` compare file view 可用性收口已落地
  - `Phase 19G` compare tree navigation / scrolling workbench 已落地
- `Phase 19H` compare tree 入口语义 / current level / quick locate 已落地
- `Phase 19I` compare tree / compare file 高级联动已落地
- `Phase 19J` compare-file 局部 locate + viewport tools 收口已落地
- 当前默认事实不再是“是否进入 `19C`”，而是“`19J` 已成为当前稳定 compare-file workbench baseline，`19I / 19H / 19F` 为其下层继承基线”。
- 后续线程默认不要重开 `19B fix-*` 或把 `19C` / `19D` / `19E` / `19F` / `19G` 重新当成 proposal；除非目标明确要求做 regression 或进入更后阶段。
- 当前 `19H` 已落地的新增事实：
  - `Results / Navigator` 现已承载主入口 `Open Compare Tree`
    - 当前以 icon-only 按钮呈现，tooltip 为 `Open Compare Tree`
  - Compare Tree 目录行右键菜单现已提供 `Set as Current Level`
  - Compare Tree 已具备 non-filter quick locate：
    - scope 继续限制在 `path / name only`
    - 行为是 reveal / focus / ensure-visible
    - 不改 compare tree visible set，不进入 search-results mode
    - header 现已提供 fixed-width search + `Prev / Next / Clear`
    - query 非空但当前 compare anchor 无匹配时，`Prev / Next` 会给出 restrained toast
  - Compare Tree header / toolbar 已收口为更明确的 viewport 语义：
    - header 当前为三行：roots/status、quick locate + 右侧 actions、breadcrumb
    - scroll lock 变为 icon-first
    - `Reset Scroll`
    - `Center Row`
- 当前 `19I` 已落地的新增事实：
  - compare-originated `File View` 返回 `Compare Tree` 时，当前文件现在可按设置 auto locate：
    - reveal 祖先链
    - ensure-visible
    - focus 当前 row
    - locate 失败时不改 compare anchor / quick locate / Hidden files，只给 restrained toast
  - `Compare File View` compare-context header 现已提供显式 `Reveal in Compare Tree`
  - `Compare File View` horizontal scroll 现已与 `Compare Tree` 对齐为同语义 `Lock / Unlock`
    - 每个 Compare Tree / compare-originated File tab 独立持有 lock 状态
    - 新开 tab 时默认值来自 `Lock compare horizontal scrolling by default`
    - gutter / relation lane 继续固定
  - `Settings -> Behavior` 现已新增两个持久化偏好：
    - `Auto locate current file when returning to Compare Tree`
    - `Lock compare horizontal scrolling by default`
- 当前 `19J` 已落地的新增事实：
  - `Compare File View` 现已提供 file-local locate：
    - 只作用于当前 compare-originated File tab
    - scope 只覆盖当前 compare file 渲染文本
    - 提供 `Prev / Next / Clear`
    - 无匹配时只给 restrained toast
    - 不复用 Compare Tree quick locate 状态
  - `Compare File View` 现已提供 viewport tools：
    - `Reset Scroll` 恢复当前 tab 的默认顶部 / 横向原点视口
    - `Recenter` 围绕当前命中或当前 compare row 做纵向居中
    - 两者都不改 lock 状态，也不改 Settings
  - `Compare File View` header 现已收口为 compare-file workbench 语言：
    - 右侧固定 locate 组件
    - 文件名过长时做截断，并通过 tooltip 展示完整文件名
    - compare-context 的 `Back / Reveal / badge / compare path` 保持不变
- 当前必须继承的 navigator 事实已固定：
  - `Results / Navigator` 已是 `tree + flat` 双视图基线
  - 非搜索默认 view 来自 `Settings -> Behavior -> Default view`
  - 搜索非空强制 `flat`
  - tree / flat 已具备 locate / ensure-visible / selection continuity
  - 目录节点只 toggle，不进入 file-view selection
- `Phase 19A` 已新增并固定：
  - Rust-owned `workspace_mode`
  - 独立 `compare_focus_path`
  - 独立 `compare_row_focus_path`
  - 独立 `compare_foundation`
  - foundation -> navigator / legacy file-view projection 迁移方向
- `Phase 19B fix-2` 当前已成立的事实：
  - 真实 `Compare View` workspace mode
  - anchored compare tree 三分工作面
  - Compare View 使用稳定 `Base / Relation / Target` 列几何
  - Compare View visible rows 已跟随 `Hidden files`
- `Phase 19C fix-1` 当前已成立的事实：
  - Rust-owned top-level `sidebar_visible`
  - app bar / title bar 前导固定区的 glyph-only sidebar toggle 已成为稳定 shell affordance
  - top bar 已进一步压低并弱化背景，但不改 macOS immersive / non-mac legacy top bar contract
  - sidebar 收起后 workspace 吃满主 split 剩余宽度
  - Compare View compare tree 已完成 disclosure / glyph / alignment / semantic lane background 收边，并基本消隐 divider；相邻同 relation rows 现在会动态消隐上下白缝，形成连续 semantic band
  - Compare header 已改为紧凑、左对齐的 bordered toolbar action + 压缩 roots context，为 `19D` 的外层 session 导航留出边界
  - Compare View / File View 在 sidebar visible/hidden 下继续维持同一 workbench 语言
  - 共享 `ToolButton` 现已默认 `horizontal-stretch: 0`；只有显式声明的 full-width action（例如 Compare）才允许铺满，避免 Settings / Analysis / modal action 的宽度爆炸回归
- `Phase 19D` 当前已成立的事实：
  - 外层 workspace 已升级为 Rust-owned session tabs：
    - `workspace_sessions`
    - `active_session_id`
    - 唯一 `Compare Tree` session
    - compare-originated `File` sessions
  - Compare Tree tab 固定在 session strip 最左侧，且同一时刻只允许一个
  - Sidebar `Results / Navigator` 继续是全局结果浏览器：
    - compare session 活跃时，从 Sidebar/Navigator 打开文件会先确认
    - 确认后关闭当前 compare session，再进入标准 `File View`
    - 不再偷偷打开 compare-originated file tab
  - `Open in Compare View` 现在是创建/激活该唯一 Compare Tree tab；若 compare session 已存在，则视为 reset 当前 compare session
    - 若当前存在 related compare file tabs，会先确认
    - 确认后保留唯一 Compare Tree tab，重设 compare anchor，并清空全部 related compare file tabs
  - `Open Compare Tree` 现已成为 `Results / Navigator` 附近的主 compare-browsing 入口：
    - 默认打开/激活 Compare Tree 于 compare root
    - 不承担 compare session reset 语义
    - 当前以 icon-only button 呈现，避免继续占用 navigator 标题区宽度
  - Compare Tree 中文件 leaf 会打开或复用外层 File tab；重复打开同一路径会切换到已有 tab
  - compare-originated File tab 现已升级为 dedicated `Compare File View`
    - 明确区分于标准 `Sidebar -> File View`
    - 保留 `Back to Compare Tree`、`Reveal in Compare Tree`、roots / compare path / compare status context
    - 使用单一纵向滚动 + Rust-owned side-by-side row projection
    - 当前已具备与 Compare Tree 对齐的 `Lock / Unlock` horizontal scroll、固定 gutter / relation lane、文本选择与系统复制、以及行号复制整行
    - compare file tab 的 lock 状态按 tab 独立，不与 Compare Tree 或其他 compare file tab 共享
    - 继续不做 sync scroll / merge actions / compare search
  - Compare Tree 现已支持 compare root 直接进入
  - Compare Tree header 已升级为 breadcrumb-first compare navigation：
    - breadcrumb segment 承担祖先目录导航语义
    - `Up` 只保留为轻量父级动作，不再与 breadcrumb 割裂
    - breadcrumb 超长时默认右对齐到当前尾部，优先展示最近目录
  - Compare Tree 现已具备左右内容 pane horizontal scroll，relation lane 继续固定
  - Compare Tree 现已具备 `Set as Current Level` 目录重锚定动作
  - Compare Tree 现已具备 header quick locate：`path / name only`、non-filter、fixed-width search、`Prev / Next / Clear`
  - query 非空但当前 compare anchor 无匹配时，`Prev / Next` 会 toast 提示，而不是静默失败
  - Compare Tree 现已具备 `Reset Scroll` / `Center Row`
  - Compare Tree horizontal scroll 当前已支持 icon-first scroll lock + tooltip 语义
  - compare/file header 当前已切到集中式 SVG 图标资源：
    - `crates/fc-ui-slint/src/icons.slint`
    - `crates/fc-ui-slint/src/assets/icons/`
    - 后续 header icon 调整优先改 SVG 资源，不再继续扩展内联 `Path` 方案
  - 关闭 Compare Tree tab 等于结束当前 compare session；存在派生 File tabs 时需确认，确认后一起清理
  - 关闭 File tab 默认直接关闭，不弹确认
- tree 内搜索、内容搜索、目录详情、compare-core widening 仍是 deferred。
- 字体方向维持当前集中式 macOS bootstrap shim 这一临时兼容基线；不要把它扩张成长期应用层字体策略。
- `docs/architecture.md`、`docs/thread-context.md` 与 `README.md` 现已统一到同一 handoff 入口。

## 快照（Snapshot）

- 日期：`2026-04-12`（Asia/Shanghai）
- 分支：`dev`
- 当前真实代码基线：
  - `Phase 17D` 稳定 shell / window / settings / tooltip / file-view contract
  - `Phase 18` navigator baseline 已收口完成（含 `18C fix-1`）
  - `Phase 19A` compare workspace foundation 已落地
  - `Phase 19B fix-2` 已成为 accepted baseline：compare tree MVP、stable compare geometry、Hidden files 接入 Compare View 已通过当前验收
  - `Phase 19C fix-1` 已成为当前稳定 shell baseline：top-level sidebar hide / restore、轻量 top chrome、Compare workspace semantic lane 语言、Compare/File 头部语言继续统一，并补齐按钮宽度约束与 relation band 连片收边
  - `Phase 19D fix-1` 已成为当前稳定 compare workspace session-shell baseline：外层 session tabs、唯一 Compare Tree tab、compare-originated File tabs、明确 compare-session close / reset 语义、Sidebar/Navigator 与 compare session 的边界收口
  - `Phase 19E` 已成为 inherited compare file renderer MVP baseline：compare-originated File tab 使用 dedicated Compare File View、单一纵向 side-by-side row projection、Back to Compare Tree 与 compare-context 保留、标准 File View 不变
  - `Phase 19F` 的单一纵向 compare file-content baseline 继续成立，其上 Compare File View 现已支持 horizontal scroll，gutter / relation lane 固定，compare 文本可选择并支持系统复制，行号可复制对应侧整行
  - `Phase 19G` 已成为当前稳定 compare tree navigation/workbench baseline：compare root 可直接进入 Compare View、顶部 path 已升级为 breadcrumb、breadcrumb overflow 默认优先展示当前最近目录、tree surface 已支持 horizontal scroll、viewport recovery/lock 已具备稳定语义
  - `Phase 19H` 已成为当前稳定 compare-tree affordance baseline：`Results / Navigator -> Open Compare Tree` 成为主入口，Compare Tree 目录行提供 `Set as Current Level`，header 提供 non-filter quick locate，toolbar 已改为 `Reset Scroll` / `Center Row`
  - `Phase 19I` 已成为当前稳定 compare tree/file coordination baseline：compare-file return auto-locate、`Reveal in Compare Tree`、Compare File View per-tab scroll lock/unlock、以及对应 Behavior settings 已落地
  - `Phase 19J` 已成为当前稳定 compare-file workbench baseline：file-local locate、`Reset Scroll`、`Recenter`、以及 compare-file header locate 收口已落地
  - compare/file header 当前已完成一轮图标资源收口：header action 与 breadcrumb nav 图标已迁到集中式单色 SVG 资源层，后续应优先维护 `icons.slint + assets/icons`
  - macOS 字体兼容当前由集中式 bootstrap shim 承担
- 当前线程已完成完整代码验证：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 当前代码验证已通过，但 `19J` 仍应重新做人工 smoke：
  - Compare Tree tab 创建 / 激活 / 固定左侧
  - `Results / Navigator -> Open Compare Tree` 是否稳定打开 compare root
  - icon-only `Open Compare Tree` 在窄宽度下是否仍可清晰辨识且不挤压 title 区
  - `Compare Status` 是否已退回 summary-first，不再承担主入口
  - Compare Tree -> File tab 打开与重复文件复用
  - Compare Tree / File tab 切换
  - Compare Tree breadcrumb 导航与 `Up` 的融合是否自然
  - `Set as Current Level` 后 breadcrumb / focus / ensure-visible 是否合理
  - quick locate 是否为 locate 而非 filtering visible-set
  - quick locate 的 `Prev / Next / Clear` 顺序、焦点保持、以及无匹配 toast 是否符合预期
  - header 第二行在最小宽度下是否仍能完整容纳 search 与右侧三个 icon-only actions
  - Compare Tree horizontal scroll 是否能覆盖长目录名 / 深层路径 / 单侧超宽树
  - `Reset Scroll` / `Center Row` 是否符合预期
  - icon-first scroll lock 与 tooltip 语义是否符合预期
  - compare-originated File tab 的 side-by-side Compare File View 是否成立
  - compare-file local locate 是否只作用于当前 compare file tab
  - compare-file `Prev / Next / Clear` 顺序与 wrap 是否符合预期
  - compare-file no-match toast 是否克制且不改 compare session 状态
  - 长行 horizontal scroll 是否可完整查看左右内容
  - Compare File View `Lock / Unlock` 是否与 Compare Tree 语义一致
  - compare-file `Reset Scroll` 是否只回顶部 / 横向原点且不改 lock 与 query
  - compare-file `Recenter` 是否围绕当前命中或当前 compare row 做纵向居中
  - compare-file 标题截断与 tooltip 是否稳定
  - gutter / relation lane 在 horizontal scroll 下是否保持固定
  - compare 文本选择与系统复制是否可用
  - 行号复制整行是否工作
  - CJK 混排行高、字符显示、左右对齐是否稳定
  - padding 行 / 删除行 / 新增行 / 字符级强调是否可扫读
  - `Back to Compare Tree` 是否稳定且能按设置 locate 当前文件
  - `Reveal in Compare Tree` 是否稳定切回并 focus 当前文件 row
  - locate 失败路径是否只给 restrained toast，且不改 compare anchor / quick locate / Hidden files
  - compare scroll lock default / return locate settings 是否重启后保持一致
  - Compare Tree tab 关闭确认与 compare-session 联动清理
  - compare session 活跃时，Sidebar/Navigator 打开文件是否先确认并回到标准 File View
  - `Open in Compare View` reset compare session 后，related file tabs 是否被一起清空
  - compare rerun / hidden-files / stale-selection 在 tab 模型下的稳定性
  - 长文件滚动是否仍可接受
- 字体链路后续收口阶段另已验证：
  - `cargo test -p fc-ui-slint`
  - `rg -n "UiTypography" crates/fc-ui-slint/src` 为 `0` 命中
  - macOS 15.x 已做人工验证，当前 bootstrap 基线可恢复中文与全角字符显示

## 当前真实稳定基线（Through Phase 17D + Phase 18 Closeout + Phase 19A Foundation）

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
- Workspace 继续固定为：
  - 外层 `Session Tabs`
  - `Session Content`
- 标准 `File` session 内继续固定：
  - `Tabs`
  - `Header`
  - `Content`
- compare-originated `File` session 现已改为 dedicated `Compare File View` 内容层：
  - `Back to Compare Tree`
  - compare-context header
  - 单一纵向 side-by-side row surface
- Sidebar 现在额外具备 top-level shell 能力：
  - `sidebar_visible`
  - 手动 hide / restore only
  - restore 入口始终留在 top-level app bar / title bar 的前导固定区
- `Diff / Analysis` 继续共用稳定 file-view shell。
- 外层 workspace product state 现已在 Rust 中持有：
  - `sidebar_visible`
  - `workspace_sessions`
  - `active_session_id`
  - `workspace_mode`
  - `compare_focus_path`
  - `compare_row_focus_path`
- macOS immersive title bar / non-mac legacy App Bar / `window_chrome` contract 继续稳定。

### 现已落地的 `Phase 18A + 18B + 18C` 事实

- `Results / Navigator` 现已支持双视图：
  - 非搜索状态默认 mode 由 `Settings -> Behavior -> Default view` 决定
  - `flat mode` 继续存在
- 搜索 contract 仍是 `path / name only`，搜索非空时强制 flat results mode。
- Tree 相关逻辑在 Rust presenter/state：
  - canonical merged tree
  - filtered visible tree rows flatten
  - expansion state
  - selection/stale-selection 决策
- Slint 侧只有独立 tree renderer，不持有递归 tree state。
- 文本面当前统一回到 Slint 默认 generic-family 路径；不再保留 `UiTypography` 或其他 runtime font-family 中转层。
- macOS 文本兼容由启动期 bootstrap shim 集中处理：
  - CoreText 查找并注册 `PingFang SC`
  - 将其接回 Slint shared font collection 的当前 generic/fallback 路径，以同时兜底已知 discovery 问题与 mixed-text 显示问题
  - 该 shim 是对当前依赖栈问题的临时兼容，不是长期应用字体策略
- 目录节点点击只负责展开/收起，不进入 file-view selection。
- 文件 leaf 节点点击复用既有 `selected_row / load diff / load analysis` 链路。
- status filter 对 tree 做剪枝，并保留必要祖先。
- 目录 `display_status` 基于过滤后可见子树重算。
- tree row disclosure 已切到绘制型 chevron，状态文案收口为 trailing lightweight text。
- `Hidden files` 继续是 UI / presentation preference，不下沉到 `fc-core`，并已与 tree mode 正确兼容。
- `Settings -> Behavior` 当前持久化四项 presentation preference：
  - `Hidden files`
  - 默认结果视图 `Tree / Flat`
  - `Auto locate current file when returning to Compare Tree`
  - `Lock compare horizontal scrolling by default`
- tree / flat 切换遵循保守 selection contract，并会把当前文件 ensure-scroll 回目标视图的可见区域。
- `Locate and Open` 现覆盖 flat results 全部入口：
  - 搜索态 flat
  - 非搜索显式 flat
  - 只针对 file leaf，继续复用既有 `selected_row / load diff / load analysis` 链路
- locate 完成后，tree 中目标 leaf 会被展开祖先链并滚动到当前可见区域。
- 搜索态 locate 完成后，`Filter / Scope` 中的 search 文本会被同步清空；不会再出现 tree 已切回但 search 仍残留的状态错位。
- compare rerun 会按新树 prune / restore expanded-path overrides；不再无条件清空。
- 折叠包含当前打开文件的目录不会触发 false stale-selection；只有 membership 真变化时才 stale。

### 现已落地的 `Phase 19A` 事实

- `fc-ui-slint` 内现已新增独立 `compare_foundation.rs`。
- compare 结果现在先进入结构化、side-aware 的 `compare_foundation`，而不是继续把字符串化 row VM 当长期 source of truth。
- foundation 当前至少承载：
  - compare root / relative path / parent path
  - entry kind
  - side presence
  - base status
  - structured detail / capabilities
  - immediate-children derivation基础
- 当前迁移方向已明确为：
  - `compare_foundation -> navigator tree projection`
  - `compare_foundation -> legacy entry_rows`
  - `legacy entry_rows -> 当前 Diff / Analysis file-view pipeline`
- `entry_rows` 仍保留，但角色已退回迁移期投影层，而不是未来 Compare workspace 的长期 source of truth。
- `compare_focus_path` 与当前文件选择状态明确分离：
  - `compare_focus_path` 是未来 Compare View 的 compare target 锚点
  - `selected_row / selected_relative_path` 继续承担当前 File View 文件锚点
- 当前 UI 可见行为现已进入双模式 workspace：
  - 外层 session tabs 当前只承载 compare side：
    - 一个固定左侧的 `Compare Tree` session
    - 若干 compare-originated `File` sessions
  - `File` session 内 attached `Diff / Analysis` shell 继续保留
  - `Compare View` 当前实现目标已校正为 anchored compare tree surface
  - Sidebar/Navigator 仍是标准 file-browsing 入口，而不是 compare session 子入口
  - tree 目录仍只 toggle，不接入 file-view selection
  - Compare Tree header 现已升级为 breadcrumb-first compare navigation；session 切换仍不由 header back button 承担
  - Compare View 已使用稳定 `Base / Relation / Target` 三列几何、轻量类型 glyph、Target 侧 disclosure 对称位、semantic lane background，以及相邻同 relation row 的动态连片
  - Compare View visible rows 已跟随 `Hidden files`
- 共享按钮基线现已固定：
  - 普通 `ToolButton` 默认不参与 `HorizontalLayout` 剩余空间拉伸
  - 只有显式声明的主操作才使用 full-width lane
  - `Settings`、`Analyze`、modal `Cancel / Save` 已回到固定/内容驱动宽度

## 当前执行焦点（Execution Focus）

- 当前不再是“启动 `18B`”或“准备 `18C`”。
- 当前真实焦点是：
  - 把 `Phase 18` 视为已收口完成的 navigator 基线
  - 把 `Phase 19A`、`19B fix-2`、`19C fix-1`、`19D`、`19E`、`19F`、`19G`、`19H`、`19I`、`19J` 视为已实现事实
  - 后续如有新线程，默认应从 `19J` 已成立 compare tree/file/session contract 出发，而不是重复 `19B` / `19C` / `19D` / `19E` / `19F` / `19G` / `19H` / `19I` / `19J` 收口，或把这些阶段写回 proposal
  - 除非遇到新回归，否则不要继续把 `18C fix-*` 当作默认主线
  - 若没有 concrete regression，下一入口优先是 `19J` 之后的明确新阶段，而不是继续默认滚动 `19G fix-*` / `19H` / `19I` 收口
  - 不要回退到 “可视区域/locate 仍 deferred” 的旧叙事
  - 不要把 compare workspace 的长期数据基础重新拉回到 `entry_rows` 字符串链路
  - 不要把 landed `19J` 顺手扩张成隐藏版 `19K+` / 多 compare session / compare-core widening
- 字体方向上的当前焦点不是继续扩展应用层字体策略，而是：
  - 维持现有集中式 macOS bootstrap shim
  - 等待可验证的上游版本升级窗口
  - 避免在应用层继续分散引入新的字体中转逻辑
- 不要回退到“flat-only baseline”叙事，也不要把本轮已实现内容继续当 proposal。

## 当前不要做什么

- 不要把继续滚动 `18C fix-*` 当成默认主线；只有明确 regression 才回去做补丁。
- 不要把 landed `19J` 顺手扩写成隐藏版 `19K+` 或更后阶段。
- 不要把 `NavigatorTreeRowProjection` 直接再升级成 Compare View foundation。
- 不要继续强化 `CompareEntryRowViewModel` 为长期 compare source of truth。
- 不要顺手重开字体策略讨论；当前边界是维持 `macos_font_bootstrap.rs` 这一临时 shim，并等待上游升级窗口。
- 不要把 `19K+` 或更后阶段的 richer compare surface、目录详情、dual-tree、cross-surface sync scroll 或 compare-core 变更写成已实现事实。
- 不要在没有明确目标的情况下，顺手重开 directory selection、directory detail pane、tree search、content search 或 compare-core widening。

## 仍然明确未做（Out of Scope / Deferred）

- locate 动画反馈 / 额外一次性强调效果（当前只有 ensure-visible + selection highlight）
- 目录进入右侧 file-view selection
- 目录 selection / 目录详情面板
- tree 内搜索 / 内容搜索 / match-span 高亮
- 目录 secondary text / descendant counts / summary
- narrow-width minimum-usable behavior beyond the current compare-file baseline
- deeper Compare View / File View redesign beyond the current session-shell baseline
- cross-surface sync scroll / compare-file reset-recenter / deeper advanced compare interaction
- 多 compare session 并发
- `fc-core` compare contract widening
- 依赖层字体栈的本地私有 fork / 长期 patch 维护

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider concerns 隔离。
2. `fc-ai` 是可选解释层；compare 输出在无 AI 情况下也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不改写 compare core 语义。
4. Sidebar 继续保持四块 IA；Workspace 外层保持轻量 session tabs，`File` session 内继续保持 attached `Diff / Analysis` shell。
5. `Compare Status` 继续 summary-first。
6. Flat results row 的 `filename-first + capability-first + weak parent-path` contract 继续有效。
7. 搜索 contract 继续是 `path / name only`；tree 不承接搜索表达。
8. tooltip 继续是 completion-first / restrained-hint-first，不演化成说明系统。
9. `Hidden files` 继续是 UI preference，不推进到 compare/core policy。
10. 不回退 macOS immersive title bar / non-mac legacy top bar / `window_chrome` split。
11. 不回退 event-driven sync、persistent `VecModel`、以及当前集中式 macOS font bootstrap shim 这一临时兼容基线；也不要重新引入 `UiTypography` 一类 runtime 字体中转层。

## 当前关键文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `docs/phase-18-tree-component-design-analysis-2026-03.md`
4. `docs/macos-pingfang-tofu-root-cause-and-fix-2026-03.md`
5. `README.md`
6. `crates/fc-ui-slint/src/macos_font_bootstrap.rs`
7. `crates/fc-ui-slint/src/main.rs`
8. `crates/fc-ui-slint/Cargo.toml`
9. `crates/fc-ui-slint/src/navigator_tree.rs`
10. `crates/fc-ui-slint/src/navigator_tree.slint`
11. `crates/fc-ui-slint/src/compare_foundation.rs`
12. `crates/fc-ui-slint/src/state.rs`
13. `crates/fc-ui-slint/src/presenter.rs`
14. `crates/fc-ui-slint/src/app.rs`
15. `crates/fc-ui-slint/src/compare_view.slint`
16. `crates/fc-ui-slint/src/commands.rs`

## 验证（Verification）

- 当前线程已完成完整代码验证：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 字体相关后续收口阶段另已验证：
  - `cargo test -p fc-ui-slint`
  - `rg -n "UiTypography" crates/fc-ui-slint/src` 为 `0`
- 自动验证已覆盖 presenter/state/tree 的核心 contract。
- UI 侧仍建议人工 smoke：
  - Settings 中默认结果视图保存、重启恢复、搜索 fallback 不回退
  - tree / flat toggle
  - tree -> flat 切换时当前文件 selection 保持且 row 自动回到可见区域
  - flat -> tree 切换时当前文件 selection 保持且 row 自动回到可见区域
  - flat results（搜索态与非搜索态）的 `Locate and Open`
  - locate 后 tree 中目标 leaf 是否稳定可见
  - 搜索强制 flat fallback
  - Compare Tree tab 创建 / 激活 / 固定左侧
  - Compare Tree 中目录 toggle 与文件 leaf open
  - 重复文件打开是否复用已有 File tab
  - Compare Tree / File tab 切换
  - Compare Tree tab 关闭确认与 compare-session 联动清理
  - compare session 活跃时 Sidebar/Navigator 打开文件是否确认后切回标准 File View
  - `Open in Compare View` reset compare session 后 related File tabs 是否一起清空
  - File tab 关闭是否直接生效
  - compare rerun 后 expanded-path restore / pruning
  - status filter / hidden-files 在多层目录上的显示
  - macOS 15.x 下 `中`、`Ａ`、`（`、`中Ａ（`、左树文件行、左树目录行、diff 正文是否仍正确显示

## Phase 19G 后入口

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 把 `Phase 17D` 之前的 shell / settings / tooltip / file-view contract 视为稳定基线。  
> 把 `Phase 18A + 18B + 18C` 已落地实现视为事实，而不是 proposal：`Results / Navigator` 已有 tree + flat 双视图；非搜索默认 mode 来自 `Settings -> Behavior -> Default view`；搜索非空强制 flat；tree logic 在 Rust presenter/state；目录节点只 toggle；文件 leaf 才进入右侧 file-view；tree / flat 切换会在目标视图 ensure-scroll 当前文件；flat results（搜索态或显式 flat）都有 `Locate and Open`；compare rerun 会 prune / restore expanded-path overrides。  
> 把 `Phase 19A` 也视为已落地事实，而不是 proposal：Rust state 中已有 `workspace_mode`；`compare_focus_path` 已与 `selected_row / selected_relative_path` 分离；`compare_foundation` 已在 `fc-ui-slint` 中成为 compare 数据基础；当前迁移方向已明确为 `compare_foundation -> navigator / legacy file-view projection`。  
> 文本链路的当前事实是：`UiTypography` 已删除，Slint 文本面回到默认 generic-family 路径；macOS 兼容逻辑集中在 `macos_font_bootstrap.rs`，它是临时 shim，不是长期应用字体策略。  
> 把 `Phase 19C fix-1` 视为已成立 shell 基线，把 `Phase 19D fix-1` 视为已成立的 outer session-shell 基线：workspace 现在有一个固定左侧且唯一的 `Compare Tree` tab，以及若干 compare-originated `File` tabs；Sidebar/Navigator 仍是全局结果浏览器，compare session 活跃时从这里打开文件必须先确认并回到标准 File View；`Open in Compare View` 会创建或激活该 Compare Tree tab，并在 compare session 已存在时把它视为 reset 当前 compare session；Compare Tree 中文件 leaf 会打开或复用 File tab；session 切换依赖外层 tab strip 而不是 header back button。不要把当前事实写回成 “`19D` 仍是 proposal”。  
> 把 `Phase 19E` / `19F` 也视为已成立事实：compare-originated `File` tab 已是 dedicated Compare File View；它继续使用单一纵向 side-by-side row projection，但现在已经具备 horizontal scroll、固定 gutter / relation lane、文本选择与系统复制、以及行号复制整行。标准 `Sidebar -> File View` 仍保持原有 `Diff / Analysis`。  
> 把 `Phase 19G` 也视为已成立事实：compare root 现在可以直接进入 Compare View；Compare Tree 顶部 path 已升级为 breadcrumb-first 导航；`Up` 已退化为轻量父级动作；Compare Tree 现已支持 horizontal scroll 与稳定的 viewport recovery / scroll-lock 基线。不要把这些能力再写回 proposal。  
> 把 `Phase 19H` 也视为已成立事实：`Results / Navigator` 现已提供 `Open Compare Tree` 主入口；Compare Tree 目录行右键菜单现已提供 `Set as Current Level`；Compare Tree 现已提供 non-filter quick locate；toolbar 现已收口为 icon-first scroll lock、`Reset Scroll`、`Center Row`。不要把这些能力再写回 proposal。  
> 把 `Phase 19I` 也视为已成立事实：compare-originated `File View` 返回 `Compare Tree` 时现在可按设置 auto locate 当前文件；header 现已提供 `Reveal in Compare Tree`；Compare File View horizontal scroll 已与 Compare Tree 对齐为同语义 `Lock / Unlock` 且 lock 状态按 tab 独立；`Settings -> Behavior` 已新增 return locate 与 compare scroll lock default。不要把这些能力再写回 proposal。  
> 把 `Phase 19J` 也视为已成立事实：Compare File View 现已具备 file-local locate、`Prev / Next / Clear`、no-match toast、`Reset Scroll`、`Recenter`、以及固定右侧 locate header。不要把这些能力再写回 proposal。  
> 当前默认下一入口不再是“是否进入 `19C` / `19D`”，而是明确判断是否真的需要进入 `19K` 或更后阶段；否则继续在 `19D` / `19E` / `19F` / `19G` / `19H` / `19I` / `19J` 稳定基线上做回归修复。  
> 当前不要顺手重开 `fc-core` contract、window system、directory selection、tree search，或继续扩散应用层字体策略，除非当前线程目标明确要求。也不要把 `19G` 或更后阶段的 richer Compare View surface 写成已实现。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件只记录当前事实和下一线程入口；不要把未经确认的未来能力写成已实现。
