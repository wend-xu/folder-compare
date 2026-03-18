# Folder Compare Upgrade Plan (`rust 1.94.0` / `slint 1.15.x`)

## 1. Purpose

本文件记录依赖升级方案与执行结果。截止 `2026-03-18`，`Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1` 已完成；本文件继续作为 `Phase 15.6`、`Phase 16` 的约束与提示词入口。

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

### Current baseline (after `Phase 15.5 fix-1`)

- `Cargo.toml`
  - workspace `version = "0.2.16"`
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
  - `SelectableDiffText` / `SelectableSectionText` 现已显式绑定 window-local selectable-content `font-family`，优先落到 `PingFang SC`，用于修复 `slint 1.15.1` `TextInput` 在 mixed Latin+CJK 文本里把全角标点渲染成 tofu 的回归
  - UI 同步仍保留 `50ms` 轮询，作为 `Phase 15.6` 清理目标
  - `15.2D` 行为已在新依赖下恢复等价

### Remaining target

- 在后续清理轮次收敛 `50ms` 轮询与 model churn
- 再在稳定升级基线上推进 `Phase 16`

## 4. Why This Upgrade Is Worth Doing

- `15.2E` 的阻断来自 `slint = 1.8.0` 缺少稳定 editable-input context-menu surface。
- 新版 Slint 已提供更适合输入控件的能力，可以显著降低 `Compare Inputs`、`Filter / Scope -> Search`、`Provider Settings` 的实现成本。
- 当前 UI 里为旧版本保留的局部手工能力可以收敛：
  - `API Key` 的手工 Show/Hide 按钮
  - `Search` 的手工 Clear 按钮
  - 对输入菜单继续“只做非输入表面”的分裂策略
- 升级后再推进 `Phase 16`，能避免在旧基线下继续堆临时实现。

## 5. Files And Surfaces Changed / Remaining

### Actually changed in `Phase 15.3A` - `Phase 15.5 fix-1`

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
- 兼容性清理
  - `crates/fc-core/src/services/classifier.rs`

### Notably unchanged during / after the migration train

- `crates/fc-ui-slint/src/context_menu.rs`
- `crates/fc-ui-slint/src/presenter.rs`
- `crates/fc-ui-slint/build.rs`

### Likely hotspots for `Phase 15.6`

- `crates/fc-ui-slint/src/app.rs`
  - `50ms` polling + snapshot sync 清理接入点
  - `Search` clear affordance 后续是否还能进一步收敛
- `crates/fc-ui-slint/src/context_menu.rs`
  - non-input safe surfaces 与 editable-input 分层保持不回退
- `crates/fc-ui-slint/src/presenter.rs` + `src/app.rs`
  - `50ms` polling + snapshot sync 清理与局部收敛
- `crates/fc-ui-slint/build.rs`
  - 仅在决定外置 `.slint` 时，才接入真正的 Slint 编译入口

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

### `Phase 15.6` - Post-upgrade cleanup

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

### `Phase 16`

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

## 7. Upgrade Benefits Realized After `Phase 15.5` / `fix-1`

- `15.2E` 不再长期 deferred
- 输入与非输入菜单策略分层更清晰
- `API Key` 输入回到原生 `TextInput` 编辑语义，同时保留保守的 secret-menu contract
- Search 输入菜单已回到原生 editable-input surface，且 clear affordance 的保留理由已明确
- 升级引入的 read-only selectable text glyph fallback 回归已被局部收敛，不再阻断真实 mixed Latin/CJK 文本阅读
- 后续 `Phase 16` 可以建立在新基线而不是旧版本临时方案上

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
- `Phase 15.5` / `fix-1` 在真实 macOS 桌面环境下的最终人工 smoke 与视觉验收
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
以 `Phase 15.5` 与 `Phase 15.5 fix-1` 已稳定为前提，本次只执行 `Phase 15.6`：做升级后的同步与结构清理。
把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1` 视为已完成，不要回头重做前置阶段。
目标：
- 识别并收窄 `50ms` UI polling 的主路径
- 降低结果列表 / diff 列表的整批 model 重建
- 仅在收益明确时，才把大块内联 Slint UI 外置并接入真正的 `slint-build`
约束：
- 不新增 Phase 16 功能
- 不回退 `15.x` 已稳定的 shell contract
- 不移除 `SelectableDiffText` / `SelectableSectionText` 当前的 glyph fallback 保护，除非已有真实 mixed Latin+CJK 样本验证默认 `TextInput` 路径不再出现 tofu
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

### Prompt for `Phase 16`

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
