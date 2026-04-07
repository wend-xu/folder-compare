# Folder Compare Thread Context (Post-Phase 18 Closeout Handoff + macOS font-bootstrap baseline)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前真实事实、当前边界、下一线程入口。
- 当前主参考是 `docs/architecture.md`；本文件只做压缩版 handoff，不替代架构文档。

## 本轮更新说明（2026-04-07）

- 当前真实代码基线已明确收口到 `Phase 18` closeout（含 `18C fix-1`），不再把 `18C fix-*` 当默认主线。
- 默认下一入口已切换为 `Phase 19` 草案讨论；只有遇到明确 regression，才回到 `18C fix-*` 做窄修复。
- 当前必须继承的 navigator 事实已固定：
  - `Results / Navigator` 已是 `tree + flat` 双视图基线
  - 非搜索默认 view 来自 `Settings -> Behavior -> Default view`
  - 搜索非空强制 `flat`
  - tree / flat 已具备 locate / ensure-visible / selection continuity
  - 目录节点只 toggle，不进入 file-view selection
- `Compare View / File View` 双模式 workspace 仍未实现；tree 内搜索、内容搜索、目录详情、compare-core widening 仍是 deferred。
- 字体方向维持当前集中式 macOS bootstrap shim 这一临时兼容基线；不要把它扩张成长期应用层字体策略。
- `docs/architecture.md`、`docs/thread-context.md` 与 `README.md` 现已统一到同一 handoff 入口。

## 快照（Snapshot）

- 日期：`2026-04-07`（Asia/Shanghai）
- 分支：`dev`
- 当前真实代码基线：
  - `Phase 17D` 稳定 shell / window / settings / tooltip / file-view contract
  - `Phase 18` navigator baseline 已收口完成（含 `18C fix-1`）
  - macOS 字体兼容当前由集中式 bootstrap shim 承担
- 已知最近一次完整代码验证：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 字体链路后续收口阶段另已验证：
  - `cargo test -p fc-ui-slint`
  - `rg -n "UiTypography" crates/fc-ui-slint/src` 为 `0` 命中
  - macOS 15.x 已做人工验证，当前 bootstrap 基线可恢复中文与全角字符显示

## 当前真实稳定基线（Through Phase 17D + Phase 18 Closeout）

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
- `Settings -> Behavior` 当前持久化两项 presentation preference：
  - `Hidden files`
  - 默认结果视图 `Tree / Flat`
- tree / flat 切换遵循保守 selection contract，并会把当前文件 ensure-scroll 回目标视图的可见区域。
- `Locate and Open` 现覆盖 flat results 全部入口：
  - 搜索态 flat
  - 非搜索显式 flat
  - 只针对 file leaf，继续复用既有 `selected_row / load diff / load analysis` 链路
- locate 完成后，tree 中目标 leaf 会被展开祖先链并滚动到当前可见区域。
- 搜索态 locate 完成后，`Filter / Scope` 中的 search 文本会被同步清空；不会再出现 tree 已切回但 search 仍残留的状态错位。
- compare rerun 会按新树 prune / restore expanded-path overrides；不再无条件清空。
- 折叠包含当前打开文件的目录不会触发 false stale-selection；只有 membership 真变化时才 stale。

## 当前执行焦点（Execution Focus）

- 当前不再是“启动 `18B`”或“准备 `18C`”。
- 当前真实焦点是：
  - 把 `Phase 18` 视为已收口完成的 navigator 基线
  - 后续如有新线程，默认入口改为 `Phase 19` 草案讨论或新的明确目标
  - 除非遇到新回归，否则不要继续把 `18C fix-*` 当作默认主线
  - 不要回退到 “可视区域/locate 仍 deferred” 的旧叙事
- 字体方向上的当前焦点不是继续扩展应用层字体策略，而是：
  - 维持现有集中式 macOS bootstrap shim
  - 等待可验证的上游版本升级窗口
  - 避免在应用层继续分散引入新的字体中转逻辑
- 不要回退到“flat-only baseline”叙事，也不要把本轮已实现内容继续当 proposal。

## 当前不要做什么

- 不要把继续滚动 `18C fix-*` 当成默认主线；只有明确 regression 才回去做补丁。
- 不要顺手重开字体策略讨论；当前边界是维持 `macos_font_bootstrap.rs` 这一临时 shim，并等待上游升级窗口。
- 不要把 `Phase 19` 未确认的设计拆分、workspace 形态或 compare-core 变更写成已实现事实。
- 不要在没有明确目标的情况下，顺手重开 directory selection、directory detail pane、tree search、content search 或 compare-core widening。

## 仍然明确未做（Out of Scope / Deferred）

- locate 动画反馈 / 额外一次性强调效果（当前只有 ensure-visible + selection highlight）
- 目录进入右侧 file-view selection
- 目录 selection / 目录详情面板
- tree 内搜索 / 内容搜索 / match-span 高亮
- 目录 secondary text / descendant counts / summary
- Compare View / File View workspace 重构
- `fc-core` compare contract widening
- 依赖层字体栈的本地私有 fork / 长期 patch 维护

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
11. `crates/fc-ui-slint/src/state.rs`
12. `crates/fc-ui-slint/src/presenter.rs`
13. `crates/fc-ui-slint/src/app.rs`
14. `crates/fc-ui-slint/src/commands.rs`

## 验证（Verification）

- 已知最近一次完整代码验证：
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
  - tree 中目录 toggle 与文件 leaf open
  - compare rerun 后 expanded-path restore / pruning
  - status filter / hidden-files 在多层目录上的显示
  - macOS 15.x 下 `中`、`Ａ`、`（`、`中Ａ（`、左树文件行、左树目录行、diff 正文是否仍正确显示

## Phase 19 草案讨论入口

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 把 `Phase 17D` 之前的 shell / settings / tooltip / file-view contract 视为稳定基线。  
> 把 `Phase 18A + 18B + 18C` 已落地实现视为事实，而不是 proposal：`Results / Navigator` 已有 tree + flat 双视图；非搜索默认 mode 来自 `Settings -> Behavior -> Default view`；搜索非空强制 flat；tree logic 在 Rust presenter/state；目录节点只 toggle；文件 leaf 才进入右侧 file-view；tree / flat 切换会在目标视图 ensure-scroll 当前文件；flat results（搜索态或显式 flat）都有 `Locate and Open`；compare rerun 会 prune / restore expanded-path overrides。  
> 文本链路的当前事实是：`UiTypography` 已删除，Slint 文本面回到默认 generic-family 路径；macOS 兼容逻辑集中在 `macos_font_bootstrap.rs`，它是临时 shim，不是长期应用字体策略。  
> 当前默认入口是 `Phase 19` 草案讨论，不是继续滚动 `18C fix-*`；只有出现明确 regression 时，才回到 `18C fix-*` 做小范围修正。  
> 不要顺手重开 `fc-core` contract、window system、directory selection、tree search，或继续扩散应用层字体策略，除非当前线程目标明确要求。也不要把 `Phase 19` 未确认设计写成已实现。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件只记录当前事实和下一线程入口；不要把未经确认的未来能力写成已实现。
