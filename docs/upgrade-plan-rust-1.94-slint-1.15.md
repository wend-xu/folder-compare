# Folder Compare Upgrade Plan (`rust 1.94.0` / `slint 1.15.x`)

## 1. Purpose

本文件记录依赖升级方案与执行结果。截止 `2026-03-18`，`Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8`、`Phase 15.8 fix-1` 已完成；依赖升级 closeout 至此收束，`Phase 16` 与后续里程碑回归主线推进模式，不再作为本升级计划的默认执行阶段。

原始升级目标是把当时基线：

- Rust `1.75.0`
- Slint `1.8.0`

升级到：

- Rust `1.94.0`
- Slint `1.15.x`

当前不再是“纯计划、未执行”状态；后续阶段将按“执行同时更新主文档”的方式继续推进。

## 2. Planning Principles

- 保持 `15.2D` 作为当前已发货稳定基线，升级期间先追求行为等价，再兑现升级收益。
- `macOS` 与 Apple Silicon 优先，不为 Intel Mac 兼容性增加额外成本。
- 不把依赖升级、`15.2E` 补票、`Phase 16` 新功能混在一个版本内。
- 先维持 workspace `edition = "2021"`，不在同一轮引入 edition 迁移噪音。

## 3. Baseline Transition Snapshot

### Original baseline (before `Phase 15.3A`)

- `Cargo.toml`
  - `rust-version = "1.75"`
  - `slint = "=1.8.0"`
  - `slint-build = "=1.8.0"`
- `rust-toolchain.toml`
  - `channel = "stable"`，未锁定精确工具链
- UI 基线
  - `fc-ui-slint` 仍使用 `src/app.rs` 内联 `slint::slint!`
  - editable input context menu 因 `slint = 1.8.0` 缺少稳定 hook 而 deferred
  - UI 同步仍依赖 `50ms` 轮询

### Current baseline (after `Phase 15.8 fix-1`)

- `Cargo.toml`
  - workspace `version = "0.2.17"`
  - `rust-version = "1.94"`
  - `slint = "=1.15.1"`
  - `slint-build = "=1.15.1"`
- `rust-toolchain.toml`
  - `channel = "1.94.0"`
- 打包版本来源
  - crate version、bundle version、DMG / ZIP version 已统一从 workspace manifest 派生
- UI 基线
  - `fc-ui-slint` 仍使用 `src/app.rs` 内联 `slint::slint!`
  - `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框已直接使用 `slint 1.15.1` `LineEdit` 自带 editable-input context menu
  - `Provider Settings -> API Key` 已落地专用 `ApiKeyLineEdit`，hidden=`Paste` only、visible=`Copy/Cut/Paste/Select All`
  - `API Key` hidden 状态额外阻断 `Cmd/Ctrl+A/C/X`，避免 masked secret 走复制/剪切捷径
  - `API Key` 外置 `Show/Hide` 按钮已收敛为字段内 reveal toggle
  - `Search` 手工 `Clear` 按钮暂时保留，因为当前 macOS native `cupertino` `LineEdit` 没有稳定 clear affordance
  - `SelectableDiffText` / `SelectableSectionText` 现已通过共享 Slint global token `UiTypography.selectable_content_font_family` 消费同一套 selectable-content `font-family`，优先落到 `PingFang SC`，用于修复 `slint 1.15.1` `TextInput` 在 mixed Latin+CJK 文本里把全角标点渲染成 tofu 的回归
  - `Workspace Diff detail` 的 body 现已切到显式 `ScrollView` 视口：column header 镜像 `viewport-x`，内容末尾保留 scrollbar-safe spacer，恢复长行横向滚动条并避免尾行被滚动条遮挡
  - compare / diff / analysis 后台完成态现已通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 回推 UI，主同步路径不再依赖常驻 `50ms` 轮询
  - loading-mask timeout 文案现已改为按 busy phase 切换调度的一次性 timer，不再依赖 repeated watchdog tick
  - `Results / Navigator` 与 `Diff` 行模型现已初始化为持久 `VecModel`，只在相关 payload 变化时 `set_vec()` 更新，避免重复 `ModelRc::new(VecModel::from(...))`
  - non-input context menu 现已完成 `Phase 15.7` visual polish：菜单面板使用分层阴影、收敛圆角与内边距，hover item 改为 inset surface + accent strip，disabled item 使用更柔和的禁用态文本
  - `Analysis success` 的 `SelectableSectionText` 现已直接复用 Slint native text surface：`Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 的正文文本支持 `ContextMenuArea` 原生右键菜单，动作最小化为 `Copy` / `Select All`
  - Analysis success section header / chrome 继续使用现有 window-local `Copy` / `Copy Summary` 菜单；`Risk Level` 继续保持显式 `Copy` 按钮-only
  - `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 右上角 inline `Copy` 按钮现已恢复可点击；根因是原先 `AnalysisSectionPanel` header 的整块右键命中层覆盖了按钮区域，本轮已把命中层收敛到 header label lane，不再遮挡 copy action
  - `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` section header 标题现已恢复左对齐；`Phase 15.8` 中把标题 `Text` 挪入 `header_context_lane` 普通 `Rectangle` 时丢失了显式 `width/height + horizontal-alignment:left`，导致文本退回居中，`Phase 15.8 fix-1` 已补回这组 geometry/alignment contract
  - `Phase 15.6` 已评估外置 `.slint`；当前收益不足以覆盖 churn，因此继续保留内联 `slint::slint!`，`build.rs` 不变
  - `15.2D` 行为已在新依赖下恢复等价

### Mainline follow-up

- 依赖升级 closeout 已完成到 `Phase 15.8 fix-1`
- `Phase 16` 回归主线推进模式，不继续在本升级计划模式中展开

## 4. Why This Upgrade Is Worth Doing

- `15.2E` 的阻断来自 `slint = 1.8.0` 缺少稳定 editable-input context-menu surface。
- 新版 Slint 已提供更适合输入控件的能力，可以显著降低 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 的实现成本。
- 当前 UI 里为旧版本保留的局部手工能力可以收敛：
  - `API Key` 的手工 Show/Hide 按钮
  - `Search` 的手工 Clear 按钮
  - 对输入菜单继续“只做非输入表面”的分裂策略
- 升级后再推进 `Phase 16`，能避免在旧基线下继续堆临时实现。

## 5. Files And Surfaces Changed / Remaining

### Actually changed in `Phase 15.3A` - `Phase 15.8 fix-1`

- 根级工具链与依赖
  - `Cargo.toml`
  - `Cargo.lock`
  - `rust-toolchain.toml`
- 文档与打包
  - `docs/architecture.md`
  - `docs/thread-context.md`
  - `docs/upgrade-plan-rust-1.94-slint-1.15.md`
  - `docs/macos_dmg.sh`
  - `README.md`
- UI 输入菜单补票
  - `crates/fc-ui-slint/src/app.rs`
- UI typography token 收敛
  - `crates/fc-ui-slint/src/ui_palette.slint`
- UI context-menu visual polish + Analysis success native text-surface closeout
  - `crates/fc-ui-slint/src/app.rs`
  - `crates/fc-ui-slint/src/ui_palette.slint`
- Diff detail scroll stabilization
  - `crates/fc-ui-slint/src/app.rs`
- UI sync / model churn cleanup
  - `crates/fc-ui-slint/src/app.rs`
  - `crates/fc-ui-slint/src/presenter.rs`
  - `crates/fc-ui-slint/src/bridge.rs`
- 兼容性清理
  - `crates/fc-core/src/services/classifier.rs`

### Notably unchanged during / after the migration train

- `crates/fc-ui-slint/src/context_menu.rs`
- `crates/fc-ui-slint/build.rs`

### Likely hotspots after `Phase 15.8 fix-1`

- `crates/fc-ui-slint/src/app.rs`
  - `Phase 15.8` 已通过 `SelectableSectionText` 上的 Slint native text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`）落地；后续不要把这条正文文本右键路径回退到现有 non-input menu core
  - `AnalysisSectionPanel` header 现已把 window-local 右键命中层限制在 header label lane，以避免继续遮挡右上角 inline `Copy` action；后续不要通过 overlay 热区重新覆盖该按钮
  - `Phase 15.8 fix-1` 已把 section label 的 `Text` 几何和 `horizontal-alignment:left` 显式写回 `header_context_lane`；后续若继续调整 header hit target，不要再依赖普通 `Rectangle` 下的默认文本布局
  - `Phase 16` 若执行，继续沿用当前 event-driven sync + persistent `VecModel` 基线
- `crates/fc-ui-slint/src/context_menu.rs`
  - non-input safe surfaces 与 editable-input 分层保持不回退；`15.7` 已完成 style-only polish，后续不要把 visual layer 反向升级成新 controller
- `crates/fc-ui-slint/build.rs`
  - 除非后续 `.slint` 外置收益明确，否则继续不接入额外编译链路 churn

## 6. Versioned Delivery Plan

### `Phase 15.3A` - Upgrade preflight

执行状态：

- 已完成（`2026-03-18`）

目标：

- 统一版本来源
- 固化升级范围和人工验收项
- 不改依赖版本，不改产品行为

要做的事：

- 明确 crate version、bundle version、DMG version 的单一事实来源
- 补齐升级 checklist 与 smoke checklist
- 明确 `Phase 15.3A -> Phase 15.4` 的阶段切线
- 文档对齐：`architecture.md`、`thread-context.md`、独立升级计划文档

人工验收标准：

- 现有 `cargo check --workspace`、`cargo test --workspace` 继续通过
- 文档中不存在“升级后做什么”与“当前基线是什么”的矛盾描述
- 打包版本来源清晰，不再同时存在多套人为维护的版本号口径

实际结果：

- workspace `Cargo.toml` 成为 crate / bundle / DMG / ZIP 版本的单一事实来源
- `docs/macos_dmg.sh` 改为从 manifest 派生版本，不再硬编码发布号
- `cargo check --workspace`、`cargo test --workspace` 通过

### `Phase 15.3B` - Rust `1.94.0` only

执行状态：

- 已完成（`2026-03-18`）

目标：

- 只升级 Rust，不升级 Slint
- 证明编译器升级本身不会破坏 `15.2D`

要做的事：

- `rust-toolchain.toml` 锁到 `1.94.0`
- workspace `rust-version` 提升到 `1.94`
- 更新 lockfile
- 修复新编译器下的 warning、lint、细微 API 兼容问题

人工验收标准：

- `cargo check --workspace` 通过
- `cargo test --workspace` 通过
- `cargo run -p fc-ui-slint` smoke 可启动
- 现有 `15.2D` 交互不回退：
  - Compare
  - Results/Navigator 选中
  - Diff/Analysis 切换
  - Provider Settings 保存
  - 现有 non-input context menu

实际结果：

- `rust-toolchain.toml` 已锁到 `1.94.0`
- workspace `rust-version` 已提升到 `1.94`
- 旧的 dead-code warning 已清理，便于后续阶段判断新增噪音
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` smoke 通过

### `Phase 15.4` - Slint `1.15.x` migration

执行状态：

- 已完成（`2026-03-18`）

目标：

- 完成 Slint 主升级
- 恢复 `15.2D` 行为等价
- 不在这个版本顺手加新产品范围

要做的事：

- `slint` / `slint-build` 升到精确 patch，建议固定到 `1.15.1`
- 修复 Slint DSL 编译错误
- 清理 `slint::slint!` 导入路径和 layout 语法
- 验证 `ui_palette`、`loading-mask`、`toast-controller`、`context-menu core` 在新版本下保持现有边界
- 必要时先做兼容层，不强推大规模 UI 重写

人工验收标准：

- `cargo check --workspace` 通过
- `cargo test --workspace` 通过
- `cargo run -p fc-ui-slint` smoke 可启动
- 下列行为与 `15.2D` 等价：
  - connected tabs / workbench seam 不回退
  - `Results / Navigator` 与 Analysis success scroll 自动关闭菜单
  - loading-mask 范围不扩张到 App Bar
  - `Risk Level` 仍保持显式 `Copy` 按钮，不错误并入通用菜单

实际结果：

- `slint` / `slint-build` 已精确锁定到 `1.15.1`
- `Cargo.lock` 已更新到新依赖图
- 现有 UI 代码对 `slint 1.15.1` 直接兼容，本轮不需要修改 `app.rs` / `context_menu.rs` / `presenter.rs`
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` smoke 通过
- macOS arm64 人工 smoke 通过，且 diff 加载性能体感明显提升

### `Phase 15.5` - Reopen and ship `15.2E`

执行状态：

- 已完成（`2026-03-18`）

目标：

- 基于新 Slint 基线正式完成 editable input context-menu integration
- 把旧版本导致的局部手工实现收敛掉

要做的事：

- 为以下输入表面接入稳定菜单：
  - `Compare Inputs`
  - `Filter / Scope -> Search`
  - `Provider Settings`
  - `API Key`
- 优先走 Slint 原生 editable-input surface
- 维持 `API Key` 的保守策略
- 收敛本地手工 affordance：
  - 密码显示切换优先改用控件原生能力
  - Search clear 优先改用控件原生能力

人工验收标准：

- 右键菜单不依赖 overlay 拦截、私有事件链路或自写 caret/selection
- 输入菜单不会破坏 typing、focus、selection、paste、cut、select-all contract
- `API Key` 在 hidden/visible 状态下行为符合预期
- 原有 non-input context menu 行为不回退

实际结果：

- `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 普通输入框继续使用 `slint 1.15.1` `LineEdit` 原生 `ContextMenuArea`，未引入 overlay、私有事件链路或自写 caret/selection/editing
- `Provider Settings -> API Key` 改为专用 `ApiKeyLineEdit`：
  - hidden 状态菜单仅保留 `Paste`
  - visible 状态提供 `Copy`、`Cut`、`Paste`、`Select All`
  - hidden 状态额外阻断 `Cmd/Ctrl+A/C/X`
- `API Key` 原有外置 `Show/Hide` 按钮已收敛为字段内 toggle，并在切换后保持输入焦点
- `Search` 手工 `Clear` 按钮本轮保留；原因是当前 macOS native `cupertino` `LineEdit` 还没有稳定 clear affordance，因此不强行引入非原生替代
- 现有 non-input context menu core 保持 window-local，`Risk Level` 仍保持 explicit `Copy` button-only
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 启动级 smoke 通过

### `Phase 15.5 fix-1` - Workspace Diff detail line glyph fallback stabilization

执行状态：

- 已完成（`2026-03-18`）

目标：

- 修复依赖升级后 `Workspace Diff detail line` 把原始全角标点渲染成方框的问题
- 明确根因属于 `fc-ui-slint` 文本渲染层，而不是 `fc-core` 编码/解码边界

要做的事：

- 用真实异常样本确认原始文本未损坏、编码未损坏、diff 数据未损坏
- 对照 `slint 1.8.0` -> `1.15.1` 的文本/字体运行时变化，定位回归层级
- 在不破坏 `SelectableDiffText` / `SelectableSectionText` 可选中、可复制 contract 的前提下做最小 UI 修复

人工验收标准：

- `Workspace Diff detail line` 中原始全角冒号 `：` 不再显示为方框
- 现有 diff 行文本选择、系统复制快捷键、行号双击复制不回退
- Analysis success sections 的 selectable text 不因同一修复产生高度/选择/滚动回退
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` smoke 通过

实际结果：

- 使用原始样本确认异常字符本体是文件内容里的全角冒号 `：`，不是乱码或编码破坏
- 升级提交 `c90f746` 本身没有改 `Diff` 面板代码；回归来自 `slint 1.15.1` 的 `TextInput` 文本引擎/字体回退路径变化
- `SelectableDiffText` 与 `SelectableSectionText` 现已显式绑定 window-local selectable-content `font-family`，优先使用 `PingFang SC`，并保留 Slint 默认 generic fallback
- 修复保持在 `fc-ui-slint/src/app.rs`，未改 `fc-core` 文本加载、解码、diff 构造逻辑
- workspace 版本随本轮收敛到 `0.2.16`

### `Phase 15.5 fix-2` - Selectable content typography token cleanup

执行状态：

- 已完成（`2026-03-18`）

目标：

- 在不改变 glyph fallback 修复效果的前提下，把 read-only selectable content 的字体策略从多层 prop threading 收敛为共享 token
- 为后续 `Phase 15.6` 清理保留更小、更稳定的 UI 边界

要做的事：

- 把 selectable content 的 `font-family` 单一事实来源提到共享 Slint global
- 删除 `MainWindow` / `AnalysisSectionPanel` 对同一字体策略的中转属性
- 同步更新三份主文档，明确 `Phase 15.7` 是菜单样式优化而不是当前默认目标

人工验收标准：

- `Workspace Diff detail line` 的全角标点显示继续正确
- `SelectableSectionText` / `SelectableDiffText` 的文本选择、系统复制快捷键、行号双击复制不回退
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` smoke 通过

实际结果：

- 新增共享 Slint global token：`UiTypography.selectable_content_font_family`
- `SelectableDiffText` 与 `SelectableSectionText` 改为直接消费该 token，不再依赖 `MainWindow` / `AnalysisSectionPanel` 透传
- 行为保持与 `Phase 15.5 fix-1` 一致，只做结构收敛，不扩张菜单范围
- workspace 版本随本轮收敛到 `0.2.17`

### `Phase 15.5 fix-3` - Diff detail horizontal scrollbar stabilization

执行状态：

- 已完成（`2026-03-18`）

目标：

- 修复依赖升级后 `Workspace Diff detail` 长行横向滚动条丢失的问题
- 保证恢复横向滚动条后，尾行不再被滚动条遮挡，仍可正常选择/复制

要做的事：

- 确认回归不是 `fix-2` 字体修复引入，而是升级后 `Diff` body 滚动容器行为变化
- 把 `Diff` body 从依赖 `ListView` 双轴滚动，改为更稳定的显式 `ScrollView` 视口
- 保留 header 与 body 的横向同步，并把 scrollbar-safe inset 收敛为内容末尾 spacer

人工验收标准：

- 长行 diff 样本重新出现可操作的横向滚动条
- 横向滚动条恢复后，最后一行不被滚动条遮挡，仍可选择文本或触发行号双击复制
- `Workspace Diff detail line` 的文本选择、`Cmd/Ctrl+C`、行号/marker 双击复制、纵向滚动都不回退
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` smoke 通过

实际结果：

- 确认回归来源不是 `Phase 15.5 fix-2`；`c90f746` 之后同一套 `Diff` 代码在 `slint 1.15.1` 下继续依赖 `ListView` 承载这类“宽表 + selectable TextInput + 变高行”的自定义 diff viewer 时，横向滚动条露出不再稳定
- `Diff` body 已改为显式 `ScrollView` 视口，column header 继续镜像 body 的 `viewport-x`
- 原来的外部 scrollbar-safe inset 已收敛为内容末尾 spacer，使尾行在滚动到底部时不再压在横向滚动条下面
- workspace 版本保持 `0.2.17`，本轮不增加版本号

### `Phase 15.6` - Post-upgrade cleanup

执行状态：

- 已完成（`2026-03-18`）

目标：

- 清理升级后仍保留的旧基线技术债

要做的事：

- 重新设计 UI sync，优先移除 `50ms` 轮询依赖
- 减少 `ModelRc::new(VecModel::from(...))` 的整批重建
- 评估是否把大块内联 `slint::slint!` 拆到外部 `.slint`
- 若外置 `.slint`，则同步启用真正的 `slint-build` 编译链路

人工验收标准：

- UI 不再依赖常驻 `50ms` 轮询作为主同步路径，或轮询范围显著收窄且有明确保留理由
- 结果列表 / diff 列表不再因为无关状态变更而全量重建
- 交互与视觉不回退

实际结果：

- `fc-ui-slint` 的 compare / diff / analysis 后台完成态已通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 回推 UI，移除常驻 `50ms` polling 主同步路径
- loading-mask timeout 文案已改为按 busy phase 切换调度的一次性 timer；scope、message、auto-timeout contract 保持不变
- `Results / Navigator` 与 `Diff` 行模型已初始化为持久 `VecModel`，相关 payload 变化时只 `set_vec()` 更新，不再反复 `ModelRc::new(VecModel::from(...))`
- UI-thread callback 仍保留 cache-aware sync 与 context-menu close-on-selection/busy-start contract；未把 menu lifecycle 反向塞进 `AppState` / `Presenter`
- 已评估把大块内联 `slint::slint!` 外置到 `.slint`；当前收益不足以覆盖迁移 churn，因此 `build.rs` 与编译链路保持现状
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 启动级 smoke 通过
- workspace 版本保持 `0.2.17`，本轮不增加版本号

### `Phase 15.7` - Context-menu visual polish

执行状态：

- 已完成（`2026-03-18`）

目标：

- 在不改变当前菜单生命周期和 safe-surface 边界的前提下，改善 non-input context menu 的视觉质量

要做的事：

- 保持现有 window-local context-menu controller，只调整菜单视觉层
- 收敛菜单面板的圆角、阴影、边框、内边距、item 高度与 hover/disabled 态
- 如需增加图标或分隔，只做 style-only 级别增强，不引入平台桥接或新的 controller

人工验收标准：

- 菜单整体观感明显优于当前版本，但交互 contract 不变
- `Results / Navigator`、Workspace header、Analysis success section 的现有右键行为不回退
- 不把 `Phase 15.6` 同步清理或 `Phase 16` 导航增强混进同一轮

实际结果：

- 保持现有 window-local context-menu controller、`context_menu.rs` action build / dispatch / close contract 不变，只调整 `fc-ui-slint` 的菜单视觉层
- non-input context menu 面板现已收敛为更紧凑的圆角、内边距与 item 高度，并增加分层阴影与顶层高光，整体层次明显优于 `Phase 15.6`
- hover item 现已使用 inset hover surface + 左侧 accent strip，disabled item 改为更柔和的禁用态文本，但 action id、safe-surface coverage、触发路径均未变化
- 本轮未扩张到 `SelectableDiffText` 行级右键菜单、editable-input 菜单、平台原生菜单桥接或新的 controller
- `cargo check --workspace`、`cargo test --workspace` 通过；`cargo run -p fc-ui-slint` 启动级 smoke 已进入运行态
- workspace 版本保持 `0.2.17`，本轮不增加版本号

### `Phase 15.8` - Analysis success native text-surface context menu

执行状态：

- 已完成（`2026-03-18`）

目标：

- 为 `Workspace Analysis success` 的 `SelectableSectionText` 接入 native text-surface right-click
- 菜单对象是当前选中文本，覆盖 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes`
- 保持 section header / chrome 继续使用现有 window-local `Copy` / `Copy Summary` 菜单

要做的事：

- 在 `SelectableSectionText` 上复用 Slint 原生 text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`）
- 菜单项最小化为 `Copy`、`Select All`
- 保持正文文本选择、系统复制快捷键、成功态独立滚动、section-level copy action 不回退
- 接受 native/no-selection 默认语义：无 selection 时不强造新的 enabled contract，可保持系统一致的 disable 或 no-op 行为

人工验收标准：

- 选中文本后右键能看到 native text-surface menu，并正确复制当前选中文本
- 无 selection 时行为与 Slint / 系统一致，不引入伪造编辑语义
- `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 全覆盖
- section header 的 window-local menu、`Risk Level` 显式 `Copy` 按钮、Analysis success scroll 行为均不回退

已接受的约束：

- 不改 `crates/fc-ui-slint/src/context_menu.rs` 的 window-local controller contract
- 不把 selectable-text 右键路由到现有 non-input menu core
- 不引入 overlay 拦截、私有事件链路或自写 caret/selection/editing
- 不扩张到 `SelectableDiffText`、Analysis shell-state text、editable inputs
- 不与 `Phase 16`、`edition = "2024"` 升级或 phase15 总结混在同一轮

实际结果：

- `SelectableSectionText` 现已直接使用 Slint native text surface：`ContextMenuArea` 挂在正文 `TextInput` 外层，菜单项收敛为 `Copy`、`Select All`
- 覆盖范围仅限 Analysis success 的 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes`，未扩张到 `Risk Level`、Analysis shell-state text、`SelectableDiffText` 或 editable inputs
- 菜单对象继续是当前选中文本；本轮没有引入 undocumented selection API，也没有人为扩张 selection-aware enabled contract
- section header / chrome 继续走现有 window-local `Copy` / `Copy Summary` 菜单；`context_menu.rs` 未改动
- `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 右上角 inline `Copy` 按钮现已恢复可点击；根因是 `AnalysisSectionPanel` header 的整块右键命中层覆盖了按钮区域，本轮已改为仅覆盖 header label lane
- `cargo check --workspace`、`cargo test --workspace` 通过；`cargo run -p fc-ui-slint` 启动级 smoke 通过
- workspace 版本保持 `0.2.17`，本轮不增加版本号

### `Phase 15.8 fix-1` - Analysis section header left-alignment regression

执行状态：

- 已完成（`2026-03-18`）

目标：

- 恢复 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` section header 标题左对齐
- 不回退 `Phase 15.8` 刚落地的 native text-surface right-click 与 inline `Copy` 按钮修复

要做的事：

- 分析 `AnalysisSectionPanel` header 标题从左对齐变为居中对齐的 root cause
- 仅做最小布局修复，把标题文本恢复到显式 left-alignment geometry contract
- 保持 `header_context_lane` 的右键命中范围与 `Copy` 按钮 clickability 不变

人工验收标准：

- 除 `Risk Level` 外，`Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 的 section header 标题重新左对齐
- `Phase 15.8` 的正文文本 right-click、header window-local menu、inline `Copy` 按钮与滚动行为均不回退

实际结果：

- root cause 已确认：`Phase 15.8` 为了把 header 的右键命中层限制到 `header_context_lane`，把标题 `Text` 从 `HorizontalLayout` 直接子项挪进了普通 `Rectangle`；该 `Text` 未再声明显式 `width/height + horizontal-alignment:left`，因此退回了 `Rectangle` 下的默认居中布局语义
- `Phase 15.8 fix-1` 已在 `header_context_lane` 内为标题文本补回 `x/y/width/height` 与 `horizontal-alignment:left`，恢复左对齐
- `header_context_lane` 的 window-local 右键命中层、正文原生 `Copy/Select All`、以及右上角 inline `Copy` 按钮 clickability 保持不变
- `cargo check --workspace`、`cargo test --workspace`、`cargo run -p fc-ui-slint` 通过
- workspace 版本保持 `0.2.17`，本轮不增加版本号

### `Phase 16`（主线参考，不再在本升级计划模式中执行）

目标：

- 在升级后的基线上恢复结果视图增强

要做的事：

- 排序
- quick jump
- 更强过滤

人工验收标准：

- 在大结果集里定位目标文件的人工步骤明显下降
- 不引入 tree mode
- 不破坏 `15.x` 已收敛的 workspace shell

## 7. Upgrade Benefits Realized After `Phase 15.7`

- `15.2E` 不再长期 deferred
- 输入与非输入菜单策略分层更清晰
- `API Key` 输入回到原生 `TextInput` 编辑语义，同时保留保守的 secret-menu contract
- Search 输入菜单已回到原生 editable-input surface，且 clear affordance 的保留理由已明确
- 升级引入的 read-only selectable text glyph fallback 回归已被局部收敛，不再阻断真实 mixed Latin/CJK 文本阅读
- glyph fallback 修复现已收敛到共享 `UiTypography` token，后续维护不再需要多层 view-level prop threading
- `Workspace Diff detail` 的长行横向滚动条已从升级后的不稳定 `ListView` 路径迁移到显式 `ScrollView` 基线，尾行复制/选择不再被滚动条遮挡
- 升级后遗留的 `50ms` UI polling 主路径已被事件驱动同步替代，后台完成态能直接回推 UI 线程
- loading-mask timeout 文案不再依赖 repeated watchdog tick，runtime sync contract 更收敛
- 结果列表 / diff 列表模型已改为持久 `VecModel`，后续阶段不必在每次相关刷新时重新分配 `ModelRc`
- non-input context menu 视觉层已与当前 desktop shell 更一致，不需要为 `Phase 16` 再混入菜单 polish 噪音
- 后续 `Phase 16` 可以建立在新基线而不是旧版本临时方案上
- `Phase 15.8` 已证明 Analysis success 正文文本可以直接建立在同一升级完成基线上接入原生 right-click，而不需要重新打开 window-local menu controller 设计
- `Phase 15.8 fix-1` 已补齐一个真实回归经验：当 section label 从 layout 直接子项迁入普通容器时，必须显式保留文本几何与 left-alignment contract，否则很容易出现默认居中布局回归

## 8. Why We Do Not Recommend `edition = "2024"` In The Same Round

不建议本次升级同时切到 `edition = "2024"`，原因不是“永远不升”，而是为了把问题域拆干净：

- 本轮主风险已经是 `slint 1.8 -> 1.15`
- 如果把 edition 也一起切，编译失败、lint 变化、宏/路径变化会混进同一批 diff
- 那样会降低“行为等价回归”的定位效率

推荐策略：

1. 先完成 `Phase 15.3B` 与 `Phase 15.4`，把 Rust 和 Slint 迁移稳定下来。
2. 等 `Phase 15.4` 或 `Phase 15.5` 稳定后，再单开一轮 edition 升级。
3. 那一轮再评估是否用 `cargo fix --edition` 作为起点。

也就是说，是的，后续 edition 升级很适合在“依赖迁移完成后”单独用 `cargo fix --edition` 起步；但不建议把它和本轮依赖升级绑死。

## 9. Work That Requires Human Ownership

- 决定 `Phase 15.5` 中哪些临时本地 affordance 可以在原生能力稳定后移除：
  - `Search` 的 Clear 按钮是否在未来 `cupertino` / style surface 提供稳定 clear affordance 后移除
- `Phase 15.5` / `fix-1` / `fix-2` / `fix-3` / `15.6` / `15.7` / `15.8` / `15.8 fix-1` 在真实 macOS 桌面环境下的最终人工 smoke 与视觉验收
- 补做一次 Analysis success text-selection/native-menu/section-copy-action 人工 smoke，确认系统菜单语义与当前滚动行为在真实桌面环境下稳定
- 补做一次 Analysis success section header alignment 人工 smoke，确认除 `Risk Level` 外各 section 标题重新左对齐
- 决定 `Phase 16` 的结果导航优先级与交互切线
- 如需签名/公证/分发，处理本机签名证书与发布流程
- 决定 edition `2024` 是否在当前升级路线之后单列里程碑

## 10. Codex Prompt Templates By Phase

### Prompt for `Phase 15.3A`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
本次只执行 `Phase 15.3A`：升级前收口，不改依赖版本，不改产品行为。
目标：
- 统一版本号来源
- 对齐文档
- 输出升级 checklist 与 smoke checklist
约束：
- 不升级 Rust / Slint
- 不改 `15.2D` 代码行为
- 不推进 `15.2E` 和 Phase 16
验证：
- `cargo check --workspace`
- `cargo test --workspace`
人工验收标准：
- 文档与版本口径一致
- 当前功能行为无变化
- 后续升级输入清单完整
```

### Prompt for `Phase 15.3B`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
本次只执行 `Phase 15.3B`：只升级 Rust 到 `1.94.0`，保持 `slint = 1.8.0`。
目标：
- 锁定 toolchain 到 `1.94.0`
- 提升 workspace `rust-version`
- 修复新编译器下的兼容问题
约束：
- 不升级 Slint
- 不改变 `15.2D` 交互与视觉边界
- 不重构 UI 同步机制
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
人工验收标准：
- Compare/Diff/Analysis/Provider Settings/context menu 基线行为不回退
- macOS arm64 smoke 通过
```

### Prompt for `Phase 15.4`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
本次只执行 `Phase 15.4`：把工程迁移到 `slint 1.15.x`，恢复 `15.2D` 行为等价。
目标：
- 升级 `slint` / `slint-build`
- 修复 Slint DSL 与导入/布局兼容问题
- 保持 `15.2D` shell、menu、loading、toast 边界
约束：
- 不顺手实现 `15.2E`
- 不推进 Phase 16
- 不把 editable input menu hack 回旧方案
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
人工验收标准：
- connected tabs / workbench seam / shell hierarchy 不回退
- non-input context menu 不回退
- loading-mask / toast 行为不回退
```

### Prompt for `Phase 15.5`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
以 `Phase 15.4` 已稳定为前提，本次只执行 `Phase 15.5`：完成 `15.2E`。
把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4` 视为已完成，不要回头重做升级或 preflight。
目标：
- 为 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings`、`API Key` 接入 editable input context menu
- 优先使用 Slint 新基线提供的稳定 surface
- 仅在原生能力已经稳定替代的前提下，收敛本地手工 password toggle / clear affordance
约束：
- 禁止 overlay 拦截、私有事件链路、自写 caret/selection/editing
- 不破坏 typing/focus/selection/paste/cut/select-all
- non-input context menu core 保持 window-local
- `Risk Level` 继续保持 explicit `Copy` button-only
- 不顺手推进 `Phase 15.6`、`Phase 16`
- 执行同时同步更新 `docs/architecture.md`、`docs/thread-context.md`、`docs/upgrade-plan-rust-1.94-slint-1.15.md`
- 不再创建额外 phase checklist 文档
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
- 人工 smoke 覆盖 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings`、`API Key`
人工验收标准：
- 输入右键菜单稳定可用
- `API Key` hidden/visible 行为正确
- typing/focus/selection/paste/cut/select-all contract 不回退
- 旧有 Results/Workspace/Analysis 菜单行为不回退
- 三份主文档与当前事实同步
```

### Prompt for `Phase 15.6`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
以 `Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3` 已稳定为前提，本次只执行 `Phase 15.6`：做升级后的同步与结构清理。
把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3` 视为已完成，不要回头重做前置阶段。
目标：
- 识别并收窄 `50ms` UI polling 的主路径
- 降低结果列表 / diff 列表的整批 model 重建
- 仅在收益明确时，才把大块内联 Slint UI 外置并接入真正的 `slint-build`
约束：
- 不新增 Phase 16 功能
- 不回退 `15.x` 已稳定的 shell contract
- 不移除 `UiTypography.selectable_content_font_family` 当前的 glyph fallback 保护，除非已有真实 mixed Latin+CJK 样本验证默认 `TextInput` 路径不再出现 tofu
- 不顺手推进 `Phase 15.7` 菜单美化
- 不为“清理”引入一次大规模 UI rewrite
- 执行同时同步更新 `docs/architecture.md`、`docs/thread-context.md`、`docs/upgrade-plan-rust-1.94-slint-1.15.md`
- 不再创建额外 phase checklist 文档
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
- 人工观察 Compare、Results/Navigator、Diff、Analysis 的 busy/loading/selection 切换
人工验收标准：
- 主同步路径不再依赖高频轮询，或轮询保留理由明确且范围变小
- 列表刷新颗粒度更合理
- 大结果集或连续切换时，diff/loading 体感进一步改善
- 视觉与交互不回退
- 三份主文档与当前事实同步
```

### Prompt for `Phase 15.7`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
以 `Phase 15.6` 已稳定为前提，本次只执行 `Phase 15.7`：做 non-input context menu 的 style-only visual polish。
目标：
- 改善菜单的圆角、阴影、边框、内边距、hover/disabled 态
- 保持现有 window-local menu lifecycle、action dispatch、safe-surface coverage 不变
约束：
- 不新增新的 controller
- 不引入平台原生菜单桥接
- 不扩张到 `SelectableDiffText` 行级右键菜单
- 不把 `Phase 16` 导航增强混进同一轮
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
人工验收标准：
- 菜单观感明显提升
- Results / Workspace header / Analysis success section 现有右键交互不回退
- 三份主文档与当前事实同步
```

### Prompt for `Phase 15.8`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7` 视为已完成。
本次只执行可选收尾 `Phase 15.8`：为 `Workspace Analysis success` 的 `SelectableSectionText` 补 native text-surface right-click。
目标：
- 菜单对象是当前选中文本
- 覆盖 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes`
- 正文文本菜单最小化为 `Copy`、`Select All`
- 保持 section header / chrome 继续使用现有 window-local `Copy` / `Copy Summary`
约束：
- 正文文本必须走 Slint native text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`）
- 不改 `crates/fc-ui-slint/src/context_menu.rs`
- 不把 selectable-text 右键路由到现有 non-input menu core
- 不要求自造严格的 selection-aware enabled contract；无 selection 时可保持 Slint / 系统一致的 disable 或 no-op 行为
- 不扩张到 `Risk Level`、Analysis shell-state text、`SelectableDiffText`、editable inputs
- 不引入 overlay 拦截、私有事件链路或自写 caret/selection/editing
- 不把 `Phase 16`、`edition = "2024"` 升级或 phase15 总结混进同一轮
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
- 人工 smoke 覆盖 Analysis success 的 text selection、right-click、scroll、keyboard copy
人工验收标准：
- 选中文本右键后可复制当前选中文本
- 无 selection 时行为与 Slint / 系统一致
- header window-local menu、`Risk Level` 显式 `Copy` 按钮、成功态滚动与 section copy action 不回退
- 三份主文档与当前事实同步
```

### Prompt for `Phase 15.8 fix-1`

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8` 视为已完成。
本次只执行 `Phase 15.8 fix-1`：修复 Analysis success section header 标题从左对齐回退为居中对齐的问题。
目标：
- 恢复 `Summary`、`Core Judgment`、`Key Points`、`Review Suggestions`、`Notes` 的 section header 标题左对齐
- 保持 `Phase 15.8` 的 native text-surface right-click 与 inline `Copy` 按钮修复不回退
约束：
- 只做最小布局修复
- 不改 `crates/fc-ui-slint/src/context_menu.rs`
- 不改变 `header_context_lane` 的 window-local menu coverage
- 不把 `SelectableSectionText` 正文文本 right-click 接回 non-input menu core
- 不扩张到 `Risk Level`、Analysis shell-state text、`SelectableDiffText`、editable inputs
- 不把 `Phase 16`、`edition = "2024"` 升级或 phase15 总结混进同一轮
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
- 人工 smoke 覆盖 Analysis success section header alignment、text selection、right-click、section copy action
人工验收标准：
- 除 `Risk Level` 外各 section header 标题恢复左对齐
- `Phase 15.8` 的正文文本 native menu、header window-local menu、inline `Copy` 按钮与成功态滚动均不回退
- 三份主文档与当前事实同步
```

### Prompt for `Phase 16`（主线参考）

```text
先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md` 和 `docs/upgrade-plan-rust-1.94-slint-1.15.md`。
以升级路线全部稳定为前提，本次执行 `Phase 16`：结果视图增强。
目标：
- sorting
- quick jump
- 更强 filter ergonomics
约束：
- 不引入 tree mode
- 不破坏当前 `App Bar + Sidebar + Workspace` IA
- 不回退 `15.x` workspace shell 与 file-view contracts
验证：
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p fc-ui-slint`
人工验收标准：
- 大结果集定位目标文件的人工步骤减少
- 结果导航效率提升可被人工感知
- Diff/Analysis 工作台边界不被破坏
```
