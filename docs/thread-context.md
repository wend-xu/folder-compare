# Folder Compare Thread Context (Live)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前稳定事实、硬边界、下一线程入口，不再把已完成的 phase train 当作待执行队列。
- 当前主参考是 `docs/architecture.md` 的 Pre-Phase 18 stable baseline summary；本文件负责把它压缩成可直接接手的线程上下文。

## 本轮更新说明（2026-03-21）

- 本轮完成了 Pre-Phase 18 文档收口：
  - `docs/architecture.md` 已重组为 “stable baseline before Phase 18”
  - 本文件已同步为同一口径
  - `README.md` 已同步到当前真实产品事实
- 已完成并关闭：
  - `Phase 17D`
  - `Phase 17C`
  - `Phase 17B fix-1`
  - `Phase 17B`
  - `Phase 17A fix-1`
  - `Phase 17A`
  - `Phase 16C fix-1`
  - `Phase 16C`
  - `Phase 16A`
  - `Phase 16A fix-1`
  - `Phase 16B`
  - `Phase 15.3A`
  - `Phase 15.3B`
  - `Phase 15.4`
  - `Phase 15.5`
  - `Phase 15.5 fix-1`
  - `Phase 15.5 fix-2`
  - `Phase 15.5 fix-3`
  - `Phase 15.6`
  - `Phase 15.7`
  - `Phase 15.8`
  - `Phase 15.8 fix-1`
  - 独立 workspace `edition = "2024"` 里程碑
- 当前默认入口不再是“继续剩余 `Phase 17`”。
- 当前默认入口是：把现有产品/UI/平台 contract 视为稳定基线，在此之上开始 `Phase 18`。

## 快照（Snapshot）

- 日期：`2026-03-21`（Asia/Shanghai）
- 分支：`dev`
- 当前工作区：Pre-Phase 18 文档收口改动
  - `docs/architecture.md`
  - `docs/thread-context.md`
  - `README.md`
- 最近提交：
  - `5a00c3c` `Phase 17D`：macOS 沉浸式标题栏
  - `e189e81` `Phase 17C`：历史 UI bug 收口与 Compare Inputs 交互收尾
  - `2bb03b8` `Phase 17B fix-1`：Settings 容器稳定性与持久化 contract 收口
  - `c8e98ea` `Phase 17B`：Settings 入口与偏好模型第一轮演进

## 当前稳定基线（Stable Baseline）

### 工具链与版本

- workspace `version = "0.2.18"`
- workspace `edition = "2024"`
- `rust-toolchain = 1.94.0`
- workspace `rust-version = 1.94`
- `slint = 1.15.1`
- `slint-build = 1.15.1`
- `15.2E` 已在上述基线上发货

### 稳定产品结构

- 顶层结构稳定为 `Top Bar + Sidebar + Workspace`
- Sidebar 当前稳定为四块 IA：
  - `Compare Inputs`
  - `Compare Status`
  - `Filter / Scope`
  - `Results / Navigator`
- Workspace 当前稳定为一个 attached workbench shell：
  - `Diff`
  - `Analysis`
  - 共享 `Tabs -> Header -> Helper Strip -> Body` 节奏
- `Diff` / `Analysis` 共享 file-view shell；只有 ready content 不同，外围 workbench contract 不同。

### 稳定交互 contract

- `Compare Inputs`
  - 继续只负责路径输入、Browse、Compare 主动作
  - `Compare` 是 full-width primary action lane
  - 按钮右侧状态说明文案已移除
- `Compare Status`
  - 继续保持 summary-first
  - 块内保留 `Show details / Hide details`
  - 保留 `Copy Summary` / `Copy Detail`
- `Filter / Scope`
  - 继续只负责 `path / name` 搜索与状态 scope
  - 不改 compare source data 与 compare-summary source counts
- `Results / Navigator`
  - 保持 flat list，不引入 tree / grouping / alternate mode
  - row 信息层级稳定为：
    - 主信息：status pill + filename
    - 次信息：capability-first summary
    - 弱信息：parent-path disambiguation

### 稳定状态语义

- `no-selection`
  - 当前没有活动选中项
- `stale-selection`
  - 之前的相对路径不再属于当前可见 `Results / Navigator` 集合
  - 左侧清掉可见选中态
  - 右侧保留显式 stale context
  - 不自动跳到第一项
- `unavailable`
  - 当前 row 有效，但 viewer / analysis 无法为该 row 产出受支持内容
- Search / Status / Hidden-files 改变后，只要当前 row 不再可见，就复用同一套 stale-selection contract。
- compare 重跑后只做同路径的保守恢复；无法恢复就保持 stale，不自动跳转。

### `Diff / Analysis` 稳定 shell

- `Diff` 状态机稳定为：
  - `no-selection | stale-selection -> loading -> unavailable | error -> preview-ready | detailed-ready`
- single-side preview 继续是一等路径：
  - `left-only`
  - `right-only`
  - `equal`
- `Analysis` 状态机稳定为：
  - `no-selection | stale-selection -> waiting | ready | unavailable -> loading -> error | success`
- `Analysis success` 继续是结构化 review-conclusion panel：
  - `Summary`
  - `Risk Level`
  - `Core Judgment`
  - `Key Points`
  - `Review Suggestions`
  - `Notes`

### tooltip / Settings / Hidden files 边界

- tooltip 当前是一个 shared window-local overlay
- 它的职责只包括：
  - 截断文本 completion
  - disabled/running `Compare` 的 restrained state hint
- tooltip 不是 explanation-heavy hover system
- `App Bar -> Settings` 是单一全局设置入口
- Settings 当前只保留两个 section：
  - `Provider`
  - `Behavior`
- `Hidden files` 当前只是 UI / presentation preference：
  - 影响当前和后续 `Results / Navigator` 的默认可见集合
  - 影响顶部摘要文案
  - 不改 compare request
  - 不改 `fc-core`
  - 不改 `Compare Status` source counts

### 平台与窗口层基线

- 平台分支当前收口在 `fc-ui-slint::window_chrome`
- macOS：
  - 使用 immersive title bar strip
  - 通过 Slint winit hook 打开 transparent title bar / full-size content view / hidden native title
  - 顶部 strip 保持 full-bleed
  - 拖拽仅在 strip 内显式触发 `drag_window()`
- Windows / Linux：
  - 保持 legacy `SectionCard` top bar
  - 不进入新的窗口初始化路径
- 当前窗口层 contract 不包括：
  - `no-frame`
  - raw AppKit / `objc2`
  - traffic lights reposition
  - 非 macOS 标题栏统一方案

### 已稳定的 supporting contracts

- ordinary editable inputs 继续使用 `slint 1.15.1` native editable-input context menu
- `Settings -> Provider -> API Key` 继续使用专用 `ApiKeyLineEdit`
- `Analysis success` 正文继续使用 native text-surface right-click
- `SelectableDiffText` / `SelectableSectionText` 继续共用 `UiTypography.selectable_content_font_family`
- ordinary inputs / `ApiKeyLineEdit` 继续共用 `UiTypography.editable_input_font_family`
- UI 主同步路径继续是 event-driven sync
- `Results / Navigator` 与 `Diff` 行模型继续是 persistent `VecModel`
- settings persistence 继续以 `settings.toml` 为唯一活跃基线，`provider_settings.toml` 只承担一次性迁移输入

## 当前执行焦点（Execution Focus）

1. 当前默认入口是稳定基线，而不是继续清扫旧 phase。
2. `Phase 18` 可以开始聚焦：
   - 现有 `Diff / Analysis` shell 内的 file-view / analysis 工作
   - 基于当前 selection/state-shell contract 的 review-efficiency 改进
   - 不改变 Sidebar / Workspace / window-layer 的窄范围增量工作
3. `Phase 18` 不应混入：
   - 新 IA
   - tree / grouping / dual-mode workspace
   - window-system rework
   - full settings framework
   - compare-level hidden-entry policy
   - global tooltip / loading / toast / controller system

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 现有 `Diff / Analysis` shell 内的窄范围演进
  - 复用当前结果行层级、selection 语义、tooltip/Settings/Hidden-files 边界的功能增量
  - 以当前窗口层 contract 为前提的非破坏性迭代
- Out of Scope：
  - tree / hierarchy / grouping navigation
  - Compare View / File View 双模式 workspace
  - 新的窗口系统方案
  - full settings framework
  - compare-level hidden-entry policy
  - global loading / toast / tooltip controller
  - 重开 `Phase 15.x`、edition `2024`、或 `Phase 16A` 到 `Phase 17D` 已接受 baseline

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不承载 core 业务规则。
4. Sidebar 继续保持四块 IA；Workspace 继续保持 attached `Diff / Analysis` shell。
5. `Compare Status` 保持 summary-first，不演化成第二个重型详情面板。
6. `Results / Navigator` 保持 flat list 与 filename-first row hierarchy。
7. selection / stale-selection / unavailable 语义保持稳定，不重新引入自动跳首项。
8. tooltip 保持 completion-first / restrained-hint-first，不演化成 explanation-heavy hover system。
9. `Hidden files` 继续是 UI preference，不推进到 compare/core policy。
10. 不回退 macOS immersive title bar / non-mac legacy top bar / `window_chrome` platform split 的当前 contract。
11. 不回退 event-driven sync、persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared typography tokens。
12. 不把新的 phase roadmap 叙事写回本文件；本文件只记录当前事实和下一线程入口。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `README.md`
4. `docs/upgrade-plan-rust-1.94-slint-1.15.md`
   - 只在需要升级与独立 edition 里程碑背景时再阅读
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

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`，必要时再看 `README.md`。  
> 把 `Phase 15.3A` 到 `Phase 15.8 fix-1`、`Phase 16A`、`Phase 16A fix-1`、`Phase 16B`、`Phase 16C`、`Phase 16C fix-1`、`Phase 17A`、`Phase 17A fix-1`、`Phase 17B`、`Phase 17B fix-1`、`Phase 17C`、`Phase 17D`，以及独立 workspace `edition = "2024"` 里程碑，全部视为已完成。  
> 把当前产品/UI/平台 contract 视为 Pre-Phase 18 稳定基线：Sidebar 四块 IA、attached `Diff / Analysis` shell、filename-first results rows、explicit stale-selection 语义、tooltip completion boundary、`Settings -> Provider / Behavior`、`Hidden files` UI preference boundary、macOS immersive title bar / non-mac legacy top bar。  
> 后续工作默认从 `Phase 18` 入口开始，只在现有 `Diff / Analysis` shell 内推进需要的 file-view / analysis / review-efficiency 改进；不要重开 IA、window-system、full settings framework、compare-level hidden policy、或已接受 baseline。  
> 保持现有产品行为、UI contract、shell / menu / loading / toast / sync 边界不变。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件应优先记录：
  - 什么已经完成
  - 当前稳定基线是什么
  - 下一线程应该从哪里进入
  - 什么不应被重新混入
