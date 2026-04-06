# Folder Compare Thread Context (Post-Phase 18C + macOS font-bootstrap baseline)

## 目的

- 本文件用于“开新线程”的短周期交接。
- 只记录当前真实事实、当前边界、下一线程入口。
- 当前主参考是 `docs/architecture.md`；本文件只做压缩版 handoff，不替代架构文档。

## 本轮更新说明（2026-04-05）

- `Phase 18A` dual-view baseline 与 `Phase 18B` mode-linkage baseline 继续成立，并已推进到 `Phase 18C` visible-region / polish baseline。
- `Settings -> Behavior` 现已持久化默认结果视图（`Tree` / `Flat`），并写入 `settings.toml`。
- tree / flat 切换现会在目标 mode 仍有效时保持当前文件 selection；切换完成后，目标 row 会在当前视口内自动滚动回可见区域。
- flat results 现已统一支持 `Locate and Open`：
  - 搜索态 flat 与显式 flat mode 都可用
  - 动作链路为：必要时清搜索 -> 切 tree -> 展开祖先 -> 定位并滚动到 file leaf -> 打开右侧 file-view
- compare rerun 不再清空 expanded-path overrides；当前会保留仍然有效的 expanded/collapsed 路径并 prune 无效项。
- tree row 保持 lightweight navigator 语言，同时把 restrained status tone 轻量延伸到文件/目录名。
- `UiTypography` 与 runtime font-family 中转层已删除；窗口文本面回到 Slint 默认 generic-family 路径。
- macOS 文本兼容逻辑现集中在 `crates/fc-ui-slint/src/macos_font_bootstrap.rs`：
  - 启动时用 CoreText 找到并注册 `PingFang SC`
  - 作为当前 `Slint 1.15.1 + fontique 0.7.0` 栈的临时兼容 shim
- 已确认的依赖层事实：
  - `macOS 13.5` 时先暴露的是字体回退/选字问题，当时通过显式 `PingFang SC` 暂时规避
  - 升级到 `macOS 15.7` 后，`fontique 0.7.0` 的字体发现问题也被暴露出来，单纯指定字体不再可靠
  - 当前 `Slint + Parley + fontique (+ renderer)` 栈仍存在 mixed-text fallback/selection 问题，但这部分暂不归因到单一 crate
  - `fontique 0.8.0` 已修复相关发现问题，但当前项目何时能随 Slint 升级拿到该修复仍不确定
- `docs/architecture.md`、`docs/thread-context.md`、`README.md` 已同步到当前 `18C` 基线，避免 handoff 仍停留在 “auto scroll deferred / search-only locate” 叙事。

## 快照（Snapshot）

- 日期：`2026-04-05`（Asia/Shanghai）
- 分支：`dev`
- 当前真实代码基线：
  - `Phase 17D` 稳定 shell / window / settings / tooltip / file-view contract
  - `Phase 18C` visible-region / locate / polish baseline 已实现
  - macOS 字体兼容当前由集中式 bootstrap shim 承担
- 已知最近一次完整代码验证：
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- 字体链路后续收口阶段另已验证：
  - `cargo test -p fc-ui-slint`
  - `rg -n "UiTypography" crates/fc-ui-slint/src` 为 `0` 命中
  - macOS 15.x 已做人工验证，当前 bootstrap 基线可恢复中文与全角字符显示

## 当前真实稳定基线（Through Phase 17D + Phase 18C）

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
- compare rerun 会按新树 prune / restore expanded-path overrides；不再无条件清空。
- 折叠包含当前打开文件的目录不会触发 false stale-selection；只有 membership 真变化时才 stale。

## 当前执行焦点（Execution Focus）

- 当前不再是“启动 `18B`”或“准备 `18C`”。
- 当前真实焦点是：
  - 把 `Phase 18C` 视为已实现基线
  - 后续如有新线程，优先做 smoke、边界修正、或非常小的 `18C fix-*`
  - 不要回退到 “可视区域/locate 仍 deferred” 的旧叙事
- 字体方向上的当前焦点不是继续扩展应用层字体策略，而是：
  - 维持现有集中式 macOS bootstrap shim
  - 等待可验证的上游版本升级窗口
  - 避免在应用层继续分散引入新的字体中转逻辑
- 不要回退到“flat-only baseline”叙事，也不要把本轮已实现内容继续当 proposal。

## 仍然明确未做（Out of Scope / Deferred）

- locate 动画反馈 / 额外一次性强调效果（当前只有 ensure-visible + selection highlight）
- 目录进入右侧 file-view selection
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

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> 把 `Phase 17D` 之前的 shell / settings / tooltip / file-view contract 视为稳定基线。  
> 把 `Phase 18A + 18B + 18C` 已落地实现视为事实，而不是 proposal：`Results / Navigator` 已有 tree + flat 双视图；非搜索默认 mode 来自 `Settings -> Behavior -> Default view`；搜索非空强制 flat；tree logic 在 Rust presenter/state；目录节点只 toggle；文件 leaf 才进入右侧 file-view；tree / flat 切换会在目标视图 ensure-scroll 当前文件；flat results（搜索态或显式 flat）都有 `Locate and Open`；compare rerun 会 prune / restore expanded-path overrides。  
> 文本链路的当前事实是：`UiTypography` 已删除，Slint 文本面回到默认 generic-family 路径；macOS 兼容逻辑集中在 `macos_font_bootstrap.rs`，它是临时 shim，不是长期应用字体策略。  
> 不要顺手重开 `fc-core` contract、window system、directory selection、tree search，或继续扩散应用层字体策略，除非当前线程目标明确要求。  
> 如果后续继续这个方向，优先做 `18C` smoke 与小范围 contract 修正，不要把当前已落地的 visible-region / locate baseline 再写回 proposal，也不要混成新一轮架构改写。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `README.md` 是否也需要更新。
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
- 本文件只记录当前事实和下一线程入口；不要把未经确认的未来能力写成已实现。
