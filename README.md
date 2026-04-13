# Folder Compare (Rust Workspace)

一个面向本地目录对比的 Rust workspace，包含确定性的目录/文本 diff 引擎、可选 AI 分析层，以及基于 Slint 的桌面 UI。

当前项目状态（2026-04-12）：

- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`
- `Phase 16A` 到 `Phase 17D` 的当前稳定基线已收口完成
- `Phase 18A` 到 `Phase 18C` 的 navigator 稳定化基线已落地并收口完成（含 `18C fix-1`）：
  - `Results / Navigator` 进入 `tree + flat` 双视图基线
  - 非搜索默认 mode 可由 `Settings -> Behavior -> Default view` 决定
  - 搜索非空强制 `flat results mode`
  - tree / flat 切换会把当前文件滚动回可见区域
  - flat results（搜索态与显式 flat）均支持 `Locate and Open`
- `Phase 19A` 已落地：
  - Rust-owned `workspace_mode`
  - 独立 `compare_focus_path`
  - 独立 `compare_row_focus_path`
  - 独立 `compare_foundation`
  - foundation -> navigator / legacy file-view projection 迁移方向
- `Phase 19B fix-2` 已成为 accepted baseline：
  - `Compare View` workspace mode
  - anchored compare tree workspace
  - stable `Base / Relation / Target` compare geometry
  - Compare View 跟随 `Hidden files`
- `Phase 19C fix-1` 已落地并成为当前稳定 shell baseline：
  - Rust-owned top-level `sidebar_visible`
  - sidebar toggle 已收口为 app bar / title bar 前导固定区的 glyph-only shell affordance
  - top bar 更轻、更低；继续保持 macOS immersive / non-mac legacy top bar contract
  - sidebar 收起后 workspace 自动吃满主 split 剩余宽度
  - Compare View compare tree 改为稳定的 semantic lane background 语言，并基本消隐 divider
  - Compare header 改为紧凑 bordered button + 压缩 roots context，并为外层 session 导航让出职责边界
  - 继续不做 auto-hide，也不进入 true `File Compare View`
- `Phase 19D` 已落地并成为当前稳定 compare workspace baseline：
  - 外层 workspace 现在是轻量 `session tabs -> session content`
  - 同一时刻只允许一个固定左侧的 `Compare Tree` tab
  - Compare Tree 中文件 leaf 会打开或复用 compare-originated `File` tabs
  - 关闭 Compare Tree tab 等于结束当前 compare session；如仍有派生 File tabs，会先确认再一起关闭
  - `Results / Navigator` 继续是全局结果浏览器；compare session 活跃时，从这里打开文件会先确认并关闭当前 compare session，然后进入标准 `File View`
  - 显式 `Open in Compare View` 在 compare session 已存在时会被定义为 reset 当前 compare session；如仍有 related compare file tabs，会先确认并一起清空
- `Phase 19E` 已落地并成为 compare-originated file tab 的 dedicated compare file view MVP 基线
- `Phase 19F` 已落地并成为当前稳定 compare file-content / workbench baseline：
  - 只改 compare-originated `File` tabs；标准 `Sidebar -> File View` 保持原有 `Diff / Analysis`
  - compare-originated `File` tab 现在使用 dedicated `Compare File View`
  - `Compare File View` 保留 `Back to Compare Tree`、roots / compare path / compare status context
  - compare 文件内容采用单一纵向滚动 + Rust-owned side-by-side row projection
  - compare 文件内容现在支持左右 horizontal scroll，且 gutter / relation lane 固定
  - compare 文本现在可选择，并支持系统复制；行号可直接复制对应侧整行
  - 当前仍不做 sync scroll / merge actions / compare search
- `Phase 19G` 已落地并成为当前稳定 compare tree navigation / scrolling baseline：
  - compare root 现在可以直接进入 Compare View
  - Compare Tree 顶部路径已升级为 breadcrumb-first 导航，`Up` 只保留为轻量父级动作
  - Compare Tree 现已支持左右内容 pane 的 horizontal scroll，relation lane 保持固定
  - Compare Tree 现已具备稳定的 viewport recovery / scroll-lock 基线
- `Phase 19H` 已落地并成为当前稳定 compare-tree affordance baseline：
  - `Results / Navigator` 现已提供主入口 `Open Compare Tree`
  - Compare Tree 目录行右键菜单现已提供 `Set as Current Level`
  - Compare Tree 现已提供 non-filter quick locate 与 `Prev / Next`
  - toolbar 现已收口为 icon-first scroll lock、`Reset Scroll`、`Center Row`
- `Phase 19I` 已落地并成为当前稳定 compare-tree/file coordination baseline：
  - 从 compare-originated `File View` 返回 `Compare Tree` 时，现可按设置 auto locate 当前文件
  - `Compare File View` compare-context header 现已提供 `Reveal in Compare Tree`
  - `Compare File View` horizontal scroll 现已与 `Compare Tree` 对齐为 shared `Lock / Unlock` 语义
  - `Settings -> Behavior` 现已新增 return locate 与 compare scroll lock default 两个持久化偏好
- `Phase 15.x` closeout 与独立 workspace `edition = "2024"` 里程碑已完成
- `15.2E` 已在当前基线上发货
- 当前 README 只维护“最新稳定事实”，不维护 phase-by-phase roadmap

![display](./docs/assets/display_0_2_18/display.gif)

## 1. Workspace 结构

- `crates/fc-core`
  - 核心比较引擎（纯本地、确定性）
  - `compare_dirs` / `diff_text_file`
- `crates/fc-ai`
  - 可选 AI 分析层
  - `Analyzer` + `AiProvider`
  - `MockAiProvider`
  - `OpenAiCompatibleProvider`
- `crates/fc-ui-slint`
  - Slint 桌面 UI
  - compare + diff + analysis 闭环
  - 平台窗口层集成（含 macOS immersive title bar facade）

## 2. 当前稳定产品基线

- 顶层 IA 保持 `Top Bar + Main Split`
- Main Split 继续是 `Sidebar + Workspace`，并新增 top-level 手动 sidebar hide / restore
- sidebar toggle 当前固定为 glyph-only top-level shell affordance，而不是 Compare View 私有控件
- Sidebar 当前稳定为四块：
  - `Compare Inputs`
  - `Compare Status`
  - `Filter / Scope`
  - `Results / Navigator`
- Workspace 当前稳定为：
  - 外层 `Session Tabs -> Session Content`
  - 标准 `File` session 内继续是 `Tabs -> Header -> Content`
  - compare-originated `File` session 使用 dedicated `Compare File View` surface
- 当前外层 workspace foundation 也已进入 Rust state：
  - `sidebar_visible`
  - `workspace_sessions`
  - `active_session_id`
  - `workspace_mode`
  - `compare_focus_path`
  - `compare_row_focus_path`
  - `compare_foundation`
- `Compare Status` 保持 summary-first，并支持块内 `Show details / Hide details` 与 `Copy Summary` / `Copy Detail`
- 当前 `Results / Navigator` 代码基线已进入双视图：
  - 非搜索默认 view 来自 `Settings -> Behavior -> Default view`
  - 搜索结果与集中扫描继续走 `flat mode`
  - compare browsing 主入口现已在这里提供 `Open Compare Tree`
- 层级结果视图仍然严格局限在同一 `Results / Navigator` block 内，不引入新 IA；详细边界见 `docs/architecture.md`
- Results row 信息层级当前稳定为：
  - 主信息：status pill + filename
  - 次信息：capability-first summary
  - 弱信息：parent-path disambiguation
- selection 语义当前稳定区分：
  - `no-selection`
  - `stale-selection`
  - `unavailable`
- Search / Status / Hidden-files 改变后，若当前 row 不再可见，则进入 explicit stale-selection，不自动跳到第一项
- compare 重跑只按同一路径做保守恢复；无法恢复则继续 stale

## 3. 当前 UI / 交互事实

- `Compare Inputs`
  - `Compare` 是 full-width primary action lane
  - 不再保留按钮右侧说明文案
  - disabled/running 说明由 restrained tooltip 承担
- `Filter / Scope`
  - 搜索 contract 当前为 `path / name`
  - 保留显式 `Clear` 按钮
- `Results / Navigator`
  - 顶部摘要使用集合状态文案（`Showing visible / total ...`）
  - 标题区提供 runtime `Tree / Flat` 切换
  - 非搜索默认 view 取决于 `Settings -> Behavior -> Default view`
  - 搜索高亮保持 label-level，不引入 match-span parsing
  - 搜索 contract 仍为 `path / name only`
  - 搜索非空时强制走 flat results mode
  - tree / flat 切换时，当前文件若仍有效，会自动滚动回目标视图可见区域
  - tree 中目录节点点击只负责展开/收起
  - tree 中文件 leaf 节点点击复用既有 file-view 打开链路
  - flat results 中 file leaf 支持 `Locate and Open` 回 tree
  - compare session 活跃时，Sidebar/Navigator 打开文件不会再偷偷创建 compare-originated `File` tab，而是先确认并回到标准 `File View`
  - 目录 compare target 可通过显式 `Open in Compare View` 动作进入 workspace `Compare View`
  - row tooltip 只做完整 filename + parent path completion
- Top-level shell
  - sidebar toggle 入口固定在 app bar / title bar 前导固定区，使用 glyph-only affordance + tooltip discoverability
  - 只做手动显隐
  - 不做 Compare View auto-hide
- `Compare View`
  - 基于 `compare_foundation` 的 anchored compare tree 投影渲染
  - 使用稳定 `Base | Relation | Target` 三列布局，header/body 共用同一套列几何
  - 目录主交互是树内 expand / collapse，不再以列表钻取作为主模型
  - compare root 现在可直接进入 Compare View
  - Compare Tree header 现已升级为 breadcrumb-first compare navigation；Compare / File session 切换仍由外层 tab strip 完成
  - `Up` 只保留为轻量父级动作，不再与 breadcrumb 分裂成两套导航
  - `Results / Navigator` 现已提供 `Open Compare Tree` 作为主 compare-browsing 入口
  - 目录行右键菜单现已提供 `Set as Current Level`
  - 轻量类型标识复用 navigator 风格，不再使用 compare tree 内的 pill 风格类型 badge
  - Compare tree 行背景按 `Diff / Equal / Left / Right` 复用 flat view 语义色，并补齐 Target 侧 disclosure 对称性
  - Compare Tree 现在支持左右内容 pane horizontal scroll，relation lane 保持固定
  - Compare Tree 现已提供 non-filter quick locate：`path / name only`、reveal / focus / ensure-visible、`Prev / Next`
  - Compare Tree 现在具备 icon-first scroll lock、`Reset Scroll`、`Center Row`
  - `Hidden files` on/off 会同步影响 Compare View visible rows
  - `Type mismatch` row 不可进入，只弹 restrained toast
- Workspace Session Tabs
  - `Open in Compare View` 会创建或激活唯一的 `Compare Tree` tab；若 compare session 已存在，则它会 reset 当前 compare session，并把 compare anchor 对准当前目录 target
  - Compare Tree tab 固定在最左侧；不会创建第二个 compare session tab
  - Compare Tree 中文件 leaf 会打开或复用同一路径的 File tab
  - compare-originated File tab 现在使用 dedicated `Compare File View`
  - Compare File View 保留 `Back to Compare Tree`、`Reveal in Compare Tree`、compare roots / path / status context
  - Compare File View 使用单一纵向 side-by-side 行投影，而不是双独立列表强同步
  - Compare File View 现在额外具备与 Compare Tree 对齐的 `Lock / Unlock` horizontal scroll、固定 gutter、可选择文本、以及行号复制整行
  - compare-originated File View 返回 Compare Tree 时，当前文件可按设置自动 reveal / focus / ensure-visible
  - compare session reset 时，全部 related compare-originated `File` tabs 会被一起清空；当前不做复杂保留策略
  - File tab 默认可直接关闭，不弹确认
  - Compare Tree tab 关闭等于结束当前 compare session；如仍有派生 File tabs，会先确认再一起关闭
- `Diff`
  - 状态机：`no-selection | stale-selection -> loading -> unavailable | error -> preview-ready | detailed-ready`
  - single-side preview 继续是一等路径
  - detail 横向滚动使用显式 `ScrollView`
  - header/body 共用列几何
- `Analysis`
  - 状态机：`no-selection | stale-selection -> waiting | ready | unavailable -> loading -> error | success`
  - success 面板继续包含：
    - `Summary`
    - `Risk Level`
    - `Core Judgment`
    - `Key Points`
    - `Review Suggestions`
    - `Notes`

## 4. Settings / Tooltip / Hidden Files

- 配置入口：App Bar / Title Bar -> `Settings`
- 当前 Settings 只保留两个 section：
  - `Provider`
  - `Behavior`
- `Behavior` 当前包含四个持久化偏好：
  - `Hidden files`
  - 默认结果视图 `Tree / Flat`
  - `Auto locate current file when returning to Compare Tree`
  - `Lock compare horizontal scrolling by default`
- `Hidden files` 当前只是 UI / presentation preference：
  - 影响 `Results / Navigator` 默认可见集合
  - 影响 `Compare View` visible rows
  - 影响顶部摘要文案
  - 不改 compare request
  - 不改 compare-summary source counts
  - 不改 `fc-core` contract
- tooltip 当前是 shared window-local overlay，只承担：
  - 截断文本 completion
  - disabled/running `Compare` 的 restrained hint
- tooltip 不是 explanation-heavy hover system

## 5. 平台与窗口层

- `fc-ui-slint` 当前通过 `window_chrome` 模块收口平台窗口层差异
- macOS：
  - 使用 immersive title bar strip
  - 启动前显式安装 winit backend selector
  - 通过 Slint winit hook 启用 transparent title bar / full-size content view / hidden title
  - blank-area drag 只在顶部 strip 内显式触发
- Windows / Linux：
  - 保持 legacy `SectionCard` top bar
  - 不进入新的窗口初始化路径
- 当前窗口层 baseline 不包括：
  - `no-frame`
  - raw AppKit / `objc2`
  - traffic lights reposition
  - 非 macOS 标题栏统一方案

## 6. 文本、菜单与运行时事实

- `Compare Inputs`、`Filter / Scope -> Search`、`Settings -> Provider` 普通输入框继续使用 `slint 1.15.1` 原生 editable-input context menu
- `Settings -> Provider -> API Key` 使用专用 `ApiKeyLineEdit`
  - hidden：`Paste` only
  - visible：`Select All`、`Copy`、`Paste`、`Cut`
- `Analysis success` 正文文本支持 native text-surface `Copy` / `Select All` right-click
- `Risk Level` 保持显式 `Copy` 按钮-only
- `SelectableDiffText` / `SelectableSectionText` 继续走 Slint 默认 generic family，由现有 macOS bootstrap 负责把系统字体接进来
- ordinary inputs / `ApiKeyLineEdit` 同样走 Slint 默认 generic family，由现有 macOS bootstrap 负责把系统字体接进来
- UI 主同步路径已切到 event-driven sync
- `Results / Navigator` 与 `Diff` 行模型使用 persistent `VecModel`
- `loading-mask` 与 `toast` 保持 UI-local boundary
- settings persistence 当前以 `settings.toml` 为唯一活跃基线；若只存在旧版 `provider_settings.toml`，启动时会一次性迁移

## 7. 运行方式

### 前置要求

- Rust `1.94.0`
- 推荐使用 `rustup`
- 仓库内已固定 `rust-toolchain.toml`
- macOS arm64 仍是当前主验证平台

### 启动 UI

```bash
cargo run -p fc-ui-slint
```

### 基础流程

1. 输入或 Browse 选择 Left / Right 目录
2. 点击 `Compare`
3. 在 `Results / Navigator` 中选择文件查看 `Diff`
4. 如需配置 provider 或 behavior：App Bar -> `Settings`
5. 切换到 `Analysis` 并点击 `Analyze`

## 8. Settings / OpenAI-compatible

### 持久化位置

- 配置入口：App Bar -> `Settings`
- 持久化文件名：`settings.toml`
- 配置目录优先级：
  - `FOLDER_COMPARE_CONFIG_DIR`
  - macOS：`~/Library/Application Support/folder-compare`
  - Windows：`%APPDATA%/folder-compare`
  - Linux：`$XDG_CONFIG_HOME/folder-compare` 或 `~/.config/folder-compare`

### 可用 provider

- `Mock`
- `OpenAI-compatible`

### OpenAI-compatible 必填配置

- `Endpoint`
- `API Key`
- `Model`

## 9. 常用验证命令

```bash
cargo check --workspace
cargo test --workspace
```

## 10. 文档入口

- `docs/thread-context.md`
  - 新线程交接、当前稳定事实、`Phase 19D` 的 handoff 入口
- `docs/architecture.md`
  - 当前稳定架构基线、`Phase 18` closeout 边界、deferred 与默认下一入口
- `docs/upgrade-plan-rust-1.94-slint-1.15.md`
  - 依赖升级与独立 edition 里程碑的归档背景

## 11. 当前开发入口

- 当前默认入口是 landed `Phase 19I`（继承 `19H / 19G`），不是继续滚动 `18C fix-*`，也不是回退到 `19B fix-*` / `19C fix-*`。
- 新工作应优先复用当前：
  - Sidebar 四块 IA
  - top-level manual sidebar hide / restore
  - attached `Diff / Analysis` shell
  - explicit stale-selection / unavailable 语义
  - tooltip / Settings / Hidden-files 边界
  - macOS immersive title bar / non-mac legacy top bar contract
- `Phase 18` 当前额外约束：
  - tree 与 flat 双视图并存
  - 搜索非空时强制 flat mode
  - tree logic 放在 Rust presenter/state，Slint 只渲染 visible rows
  - tree / flat 与 locate 的可视区域连续性已是当前基线，不再视为 deferred
- `Phase 19A` 当前额外事实：
  - `workspace_mode` 已进入 Rust state
  - `compare_focus_path` 已与 file selection state 分离
  - `compare_foundation` 已在 `fc-ui-slint` 内落地
  - 当前迁移方向已是 `compare_foundation -> navigator / legacy file-view projection`
- `Phase 19D` 当前额外事实：
  - 外层 workspace session tabs 已落地
  - 当前只允许一个固定左侧的 `Compare Tree` session
  - Compare Tree 中文件 leaf 会打开或复用 compare-originated `File` tabs
  - Compare session 的进入、切换、结束语义现在由外层 tab strip 承担
- `Phase 19G` 当前额外事实：
  - compare-originated `File` tab 现已升级为 dedicated `Compare File View`
  - 标准 `Sidebar -> File View` 继续保留既有内层 `Diff / Analysis`
  - Compare File View 使用单一纵向 side-by-side 行投影，并保留 `Back to Compare Tree`
  - Compare File View 现在支持 horizontal scroll、固定 gutter / relation lane、文本选择与系统复制、以及行号复制整行
  - Compare Tree 现在支持 compare root 直接进入、breadcrumb 导航、horizontal scroll、以及明确的 viewport recovery / scroll-lock 语义
- `Phase 19H` 当前额外事实：
  - `Results / Navigator` 现已提供主入口 `Open Compare Tree`
  - Compare Tree 目录行右键菜单现已提供 `Set as Current Level`
  - Compare Tree 现已提供 non-filter quick locate：
    - `path / name only`
    - reveal / focus / ensure-visible
    - `Prev / Next`
  - Compare Tree toolbar 现已收口为 icon-first scroll lock、`Reset Scroll`、`Center Row`
- `Phase 19I` 当前额外事实：
  - compare-originated `File View` 返回 `Compare Tree` 时可按设置 auto locate 当前文件
  - `Compare File View` header 现已提供 `Reveal in Compare Tree`
  - `Compare File View` horizontal scroll 现已与 `Compare Tree` 对齐为 shared `Lock / Unlock`
  - `Settings -> Behavior` 现已新增 return locate 与 compare scroll lock default 两个持久化偏好
- 当前仍未实现：
  - compare tree filtering search / search-results mode / 内容搜索
  - 目录 selection / 目录详情
  - narrow-width minimum-usable behavior beyond the current compare-file baseline
  - compare core widening
  - cross-surface sync scroll / compare-file reset-recenter / richer compare interaction
  - 超出当前 `19I` 的更深层 `Compare View / File View` 重构
- 后续只有在目标明确时才进入 `19J` 或更后阶段；不要把 landed `19I` 重新写回 proposal。
- README 下方保留长期 roadmap 参考；如需判断当前下一阶段可做什么，直接参考 `docs/architecture.md`。

## 12. 长期路线（参考）

- 本节用于保留产品长期方向，便于快速理解项目后续可能演进到哪里。
- 这是方向性 roadmap，主要保留历史演进脉络参考。
- 它不覆盖 `docs/architecture.md` 中的当前稳定事实，也不替代当前线程的实际执行入口。
- phase 编号按当前真实推进事实校正；已确认的 `Phase 17` 实际落点是 `Settings` 升级，因此原先其后的长期路线整体顺延。

- `Phase 16`
  - 结果视图增强（状态筛选 / 排序 / 更强过滤）
- `Phase 17`
  - `Settings` 升级（设置入口统一、Provider / Behavior 分区、首轮行为偏好）
- `Phase 18`
  - 层级结果视图 / tree component / flat results 双视图
- `Phase 19`
  - Compare workspace 演进（`19A` foundation 已落地，`19B fix-2` 已收口 compare tree MVP，`19C` 已完成 shell closeout，`19D` 已落地 outer session tabs）
- `Phase 20`
  - AI 分析增强（多任务 / hunk 关联 / 缓存）
- `Phase 21`
  - Diff / Analysis 高级交互
- `Phase 22`
  - 后台任务与性能体系
- `Phase 23`
  - 产品化收尾
