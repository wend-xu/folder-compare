# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的短周期交接，只记录当前事实、边界和下一步，不再把已完成的 phase train 当作待执行队列。

## 本轮更新说明（2026-03-20）

- 本轮执行 `Phase 17B fix-1`，在 `Phase 17B` 已建立的 `Settings -> Provider / Behavior` 基线上，收口 Settings modal 尺寸稳定性、把 settings persistence contract 明确为 `settings.toml` 唯一基线加一次性 legacy migration，并把 hidden-files 暂不下沉到 `fc-core` 的判断写入主文档；不引入完整 settings framework，也不重开 compare/diff/analysis 核心数据结构。
- 已完成并关闭：
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
- 当前稳定基线：
  - workspace `version = "0.2.18"`
  - workspace `edition = "2024"`
  - `rust-toolchain = 1.94.0`
  - workspace `rust-version = 1.94`
  - `slint = 1.15.1`
  - `slint-build = 1.15.1`
  - `15.2E` 已在上述基线上落地
  - event-driven sync 已落地
  - persistent `VecModel` 已落地
  - `Diff` 显式 `ScrollView` 视口已落地
  - shared `UiTypography.selectable_content_font_family` 已落地
  - shared `UiTypography.editable_input_font_family` 已落地
  - non-input context-menu visual polish 已落地
  - `Analysis success` native text-surface right-click 已落地
  - section header 左对齐修复已落地
  - `Compare Status` 块内 detail tray + `Copy Summary` / `Copy Detail` 已落地
  - `Compare Status` 折叠区与展开 tray 区的右键菜单覆盖已统一
  - `Results / Navigator` 顶部集合状态条已落地
  - `Results / Navigator` row 已收口为 filename-first 主信息、reason summary 次信息、parent-path 弱信息
  - `Results / Navigator` 已支持基于现有 `path / name` contract 的克制命中高亮
  - Search / Status 改变后的 selection policy 已收口为“可保留则保留，不可保留则显式 stale，不自动跳到第一项”
  - compare 重跑后已支持基于同一路径的低风险恢复；无法恢复时进入 stale selection
  - row secondary summary 已更贴近右侧真实 viewer 能力，尤其是 non-text / binary / preview-unavailable 项
  - `Diff / Analysis` 已区分 `no selection` 与 `stale selection`，并收口基础 unavailable / ready / waiting 语义
  - `Results / Navigator` row 次信息已进一步收缩为 capability-first 短语，弱 parent-path 会更早让出宽度
  - `Compare Inputs`、`Search`、`Settings -> Provider`、`API Key` 输入框已统一走 CJK-safe editable input font token，修复全角标点 TOFU
  - window-local shared tooltip 基建已落地
  - `Results / Navigator` row 现已收口为 row-level tooltip：当 filename 或 parent-path 截断时，整行 hover 可稳定补全完整 filename 与完整 parent path
  - `Compare Inputs` 左右路径与 `Filter / Scope -> Search` 长文本输入已支持非编辑态 tooltip 补全文本，且长值会先裁切在输入框内，不再突破 Sidebar 边界
  - shared tooltip overlay 默认优先显示在目标上方；上方空间不足时再降级到下方
  - `App Bar -> Settings` 已替换 `Provider Settings`，并提供第一轮 `Provider / Behavior` 分区骨架
  - Settings modal 当前以最大内容态固定容器尺寸；`Provider / Behavior` 与 `Mock / OpenAI-compatible` 间切换不再改变外层容器尺寸
  - persisted settings 当前以 `settings.toml` 为唯一活跃基线；若只存在 legacy `provider_settings.toml`，启动时会一次性迁移到 `settings.toml`
  - `Settings -> Behavior` 已落地首个偏好项：`Hidden files` 默认显示/隐藏策略；该偏好保存后会立即作用到当前 `Results / Navigator` 可见集合，并影响后续 compare 结果展示
  - `Hidden files` 当前仅是 UI 偏好，不修改 compare request / `fc-core` contract；若它让当前项不可见，则沿用现有 stale selection 语义，Results 顶部摘要会显式提示被 Settings 隐藏的数量
- 保持不变：
  - `15.2D` 的 IA 与 shell contract 不变
  - connected tabs + attached workbench surface 不变
  - `Diff/Analysis` shell 不变
  - editable-input context-menu contract 不变
  - `Settings -> Provider -> API Key` secret contract 不变
  - window-local non-input context-menu core contract 不变
  - loading-mask / toast boundary 不变
  - UI 继续使用内联 `slint::slint!`
- 为什么下一步才是剩余 `Phase 17`：
  - `Phase 16A`、`16A fix-1`、`16B`、`16C`、`16C fix-1` 已把 Sidebar 表达、flat-list 结果扫描能力，以及 File View 状态联动收口到当前稳定基线；
  - `Phase 17A` 已在此基础上补齐克制的 tooltip completion layer，`Phase 17A fix-1` 则进一步把 row hover 稳定性、输入框裁切与 overlay 定位收口；
  - `Phase 17B` 则把设置入口升级到最小可扩展骨架，并以 UI 偏好方式收口 hidden-files 默认可见性，而没有重开完整 settings framework、tree mode 或 compare/core contract；
  - 因此下一线程应继续剩余 `Phase 17` 工作，而不是继续重开 `15.3A` 到 `15.8 fix-1`、`16A` 到 `16C fix-1`、或 edition 兼容修复。

## 快照（Snapshot）

- 日期：`2026-03-20`（Asia/Shanghai）
- 分支：当前执行 `Phase 17B fix-1`
- 工作区：`fc-ui-slint` Settings modal 尺寸稳定性、settings persistence contract 收口、hidden-files behavior preference、主文档同步改动
- 最近提交：
  - `0a8769d` Phase 16A fix-1
  - `1311f96` edition-2024 milestone
  - `7e59de3` Phase 15.8 fix-1: section title align
  - `51b28cd` Phase 15.8：Analysis success selectable-text native menu
- 当前主参考：
  - `docs/architecture.md`：长期“当前架构基线 + deferred decisions + next priority”
  - `docs/upgrade-plan-rust-1.94-slint-1.15.md`：升级与独立 edition 里程碑的归档背景

## 当前目标（Execution Focus）

1. `Phase 17B` 已完成；Sidebar 仍然保持 `Compare Inputs -> Compare Status -> Filter / Scope -> Results / Navigator`。
2. `Results / Navigator` 当前稳定 contract 是 flat list：filename-first 主信息、reason summary 次信息、parent-path 弱信息，以及仅基于现有 `path / name` 搜索 contract 的轻量命中高亮。
3. selection / file-view 当前稳定 contract：
   - Search / Status 改变后，当前项仍可见则保留；否则清掉左侧可见选中态，右侧进入 stale selection
   - compare 重跑后仅按同一路径做基础恢复；无法稳定恢复则保持 stale selection，不自动跳转
   - `Diff / Analysis` 明确区分 `no selection`、`stale selection`、`unavailable` 与 `ready/waiting`
   - `Settings -> Behavior -> Hidden files` 保存后若让当前项不可见，也沿用同一套 stale selection 收口，不自动跳转
4. settings / row / input 当前补充 contract：
   - `App Bar -> Settings` 是新的全局设置入口，当前只做第一轮 `Provider / Behavior` 分区，不引入完整 settings framework
   - `Settings -> Provider` 继续承接原有 provider 配置能力；`Settings -> Behavior` 当前只承载 `Hidden files`
   - `Hidden files` 当前只作用于 `Results / Navigator` 可见集合与其顶部摘要，不改变 `Search` 的 `path / name` contract，也不改变 `Compare Status` summary-first 统计来源
   - row 次信息优先表达是否可 `text diff` / `text preview`，并使用短语以适配当前 Sidebar 宽度
   - 普通输入框与 `ApiKeyLineEdit` 共用 `UiTypography.editable_input_font_family`，避免 `slint 1.15.1` 默认 editable widget 字体链导致的全角字符 TOFU
   - tooltip 当前是 window-local shared overlay，只承担被截断文本的补全；Results row 已收口为 row-level tooltip，只补全完整 filename + 完整 parent path；Compare/Search 继续只补全输入完整值
   - shared tooltip overlay 默认优先上方定位；空间不足时降级到下方
   - `TooltipLineEdit` 包装层会继续保留 native editable behavior，同时把可见输入宽度收紧在控件矩形内
5. 后续线程继续 `Phase 17` 剩余子项时，不要重开 `Phase 15.3A` 到 `Phase 15.8 fix-1`、`Phase 16A`、`16A fix-1`、`16B`、`16C`、`16C fix-1`、`Phase 17A`、`Phase 17A fix-1`、`Phase 17B`，也不要重开独立 workspace `edition = "2024"` 里程碑。
6. 继续保持主文档与当前代码事实一致，不创建额外 phase checklist / summary 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - `Phase 17B`
  - `Settings` 入口升级
  - `Provider / Behavior` 第一轮分区
  - `Hidden files` 首个非 Provider 偏好项与结果可见性收口
- Out of Scope：
  - tree / hierarchy / grouping navigation
  - 排序系统
  - 内容搜索
  - Diff / Analysis 主体改造
  - edition `2024` 兼容修复回合
  - 新的全局菜单 / loading / toast / controller 方案
  - 完整 settings framework
  - compare-level hidden-entry policy

## 硬契约（Do Not Break）

1. `fc-core` 必须保持确定性，并与 UI / 网络 / provider 关注点隔离。
2. `fc-ai` 是可选解释层；即使关闭 AI，compare 输出也必须完整可用。
3. `fc-ui-slint` 负责 orchestration / presentation，不承载 core 业务规则。
4. Workspace 结构保持 `Tabs -> Header -> Content`，connected workspace tabs + attached workbench surface 是 accepted baseline。
5. Compare Status 保持 summary-first，不演化为第二个重型详情面板。
6. `Compare Inputs`、`Filter / Scope -> Search`、`Settings -> Provider` 普通输入框继续走 `slint 1.15.1` 原生 editable-input context menu。
7. `Settings -> Provider -> API Key` 继续保持专用 `ApiKeyLineEdit`：hidden=`Paste` only；visible=`Select All`、`Copy`、`Paste`、`Cut`。
8. `Analysis success` 正文文本继续走 Slint native text surface（`ContextMenuArea` + `TextInput.copy()/select-all()`）；section header / chrome 继续走 window-local `Copy` / `Copy Summary` 菜单；`Risk Level` 继续保持显式 `Copy` 按钮-only。
9. 不重新引入 broad `50ms` polling，不回退 persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared `UiTypography.selectable_content_font_family`。
10. 不把 editable input 字体策略散落到多个局部组件，继续通过共享 typography token 统一控制。
11. 不把完整 settings framework、compare-level hidden policy、剩余 `Phase 17` 之外的新产品行为变更混入当前文档 closeout。

## 当前稳定事实（Stable Facts）

- 依赖与工具链：
  - `Cargo.toml` 已固定 workspace `version = "0.2.18"`、workspace `edition = "2024"`、workspace `rust-version = "1.94"`、`slint = "=1.15.1"`、`slint-build = "=1.15.1"`
  - `rust-toolchain.toml` 已固定 `channel = "1.94.0"`
  - `docs/macos_dmg.sh` 继续从 workspace manifest 派生 bundle / DMG / ZIP 版本
- UI / shell：
  - `15.2E` 已在当前基线上发货
  - `App Bar -> Settings` 现已替代 `Provider Settings`，并提供第一轮 `Provider / Behavior` 分区骨架
  - `Diff` 与 `Analysis` 共用已验收的 workbench shell，不改 tabs / header / content 层次
  - `Diff` detail 长行横向滚动维持显式 `ScrollView` 视口，尾行通过 scrollbar-safe spacer 避免被横向滚动条遮挡
  - `SelectableDiffText` 与 `SelectableSectionText` 共用 `UiTypography.selectable_content_font_family`
  - 普通输入框与 `ApiKeyLineEdit` 现在共用 `UiTypography.editable_input_font_family`，修复 mixed Latin/CJK 与全角标点输入的 TOFU
  - `Compare Status` 继续保持 summary-first，并在块内支持 `Show details / Hide details` tray
  - `Compare Status` 右键菜单支持 `Copy Summary` / `Copy Detail`
  - `Compare Status` 折叠区与展开 tray 区都支持同一套上下文菜单
  - `Filter / Scope -> Search` 当前 contract 收口为 path/name 匹配
  - Search 命中高亮当前仍是 row-local label 级高亮；若未来需要 match-span / substring highlight，应由更下层提供 match positions 或预切分结果，Slint 视图层只负责渲染
  - `Filter / Scope` 不再向用户重复显示单独的 `scope` 文案
  - `Results / Navigator` 顶部摘要现在表达当前结果集合状态（`Showing visible / total ...`）
  - `Results / Navigator` row 顶部主信息现在优先展示 filename / leaf path segment，而不是整条路径文本
  - `Results / Navigator` row 次信息现在优先表达 `diff / equal / left / right` 的 capability-first 短摘要，而不是直接暴露原始 detail 文本
  - `Results / Navigator` row 保留 parent-path 作为弱 disambiguation 信息，不引入 tree / hierarchy / grouping
  - `Results / Navigator` 搜索命中高亮仅基于当前 `path / name` contract，不引入 detail/content 搜索
  - tooltip 当前是一个共享的 window-local overlay；它只在文本被截断时补全文本，不承担新的说明职责
  - `Results / Navigator` row 的 tooltip 已收口为 row-level completion：当 filename 或 parent-path 截断时，整行 hover 可查看完整 filename 与完整 parent path
  - `Compare Inputs` 左右路径与 `Filter / Scope -> Search` 在非编辑态、长文本被截断时可通过 tooltip 查看完整值，且文本会先被裁切在输入框内
  - shared tooltip overlay 默认优先在目标上方展示，若空间不足则自动降级到下方
  - `Settings -> Behavior -> Hidden files` 当前只影响 `Results / Navigator` 可见集合；摘要会显式提示被 Settings 隐藏的数量，但 `Compare Status` 仍保持 summary-first 原始统计
  - Search / Status 改变后，若当前 source row 仍在新集合中则保持选中；若不在新集合中，则左侧清掉可见选中态，右侧保留 stale path 语义，不自动跳到第一项
  - compare 重跑后只按同一路径做低风险恢复；若同一路径仍存在且在当前 filter 下可见，则恢复选中并自动刷新 File View；否则进入 stale selection
  - row secondary summary 现在更主动提示 non-text / binary compare 与常见 preview-unavailable 文件类型，帮助用户预判右侧 viewer 能力，并通过更短文案适配当前 Sidebar 宽度
  - Analysis success 正文文本支持 native text-surface `Copy` / `Select All` right-click
  - Analysis success section header 标题继续保持显式左对齐，且不会遮挡右上角 inline `Copy`
  - `Diff` 与 `Analysis` 当前都已支持显式 stale selection 文案，不再把 filtered-out selection 与从未选中过的 no-selection 混为一谈
  - persisted settings 当前保存到 `settings.toml`；若仅存在 legacy `provider_settings.toml`，启动时会一次性迁移到新文件，之后继续以 `settings.toml` 为唯一活跃基线
- 运行时：
  - compare / diff / analysis 后台完成态继续通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 回推 UI
  - `Results / Navigator` 与 `Diff` 行模型继续使用 persistent `VecModel`
  - loading-mask timeout copy 继续使用按 busy phase 调度的一次性 timer

## 下一步（Next）

- 唯一建议的下一步是继续剩余 `Phase 17` 工作。
- 后续实现应建立在当前 `0.2.18 + edition 2024 + rust 1.94.0 + slint 1.15.1 + Phase 16A + 16A fix-1 + 16B + 16C + 16C fix-1 + Phase 17A + Phase 17A fix-1 + Phase 17B` 基线上，不重开升级、edition 迁移或 `Phase 15` closeout。
- 继续保持当前 shell / menu / loading / toast / event-driven sync contract 不变。

## 开始前优先阅读文件（Key Files）

1. `docs/thread-context.md`
2. `docs/architecture.md`
3. `docs/upgrade-plan-rust-1.94-slint-1.15.md`（仅在需要升级归档背景时再读）
4. `crates/fc-ui-slint/src/app.rs`
5. `crates/fc-ui-slint/src/context_menu.rs`
6. `crates/fc-ui-slint/src/presenter.rs`
7. `crates/fc-ui-slint/src/settings.rs`
8. `crates/fc-ui-slint/src/ui_palette.slint`
9. `Cargo.toml`
10. `rust-toolchain.toml`

## 验证（Verification）

- 本轮验证重点是 `Phase 17B` 的 Settings 入口升级、settings persistence 与 hidden-files preference 收口：
  - `cargo check -p fc-ui-slint`
  - `cargo test -p fc-ui-slint`
- 本轮未运行 `cargo run -p fc-ui-slint`：
  - 原因：本轮未在此线程内做桌面 GUI smoke；Settings 分区布局与 hidden-files 交互的真实视觉验收仍保留给人工

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> `docs/upgrade-plan-rust-1.94-slint-1.15.md` 只在需要升级与独立 edition 里程碑归档背景时再阅读。  
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8`、`Phase 15.8 fix-1`、`Phase 16A`、`Phase 16A fix-1`、`Phase 16B`、`Phase 16C`、`Phase 16C fix-1`、`Phase 17A`、`Phase 17A fix-1`、`Phase 17B`，以及独立 workspace `edition = "2024"` 里程碑，全部视为已完成。  
> 把当前稳定基线视为：workspace `version = "0.2.18"`、workspace `edition = "2024"`、`rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`，且 `15.2E`、event-driven sync、persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared `UiTypography.selectable_content_font_family`、non-input context-menu visual polish、`Analysis success` native text-surface right-click、section header 左对齐修复均已稳定。  
> 当前默认进入剩余 `Phase 17`，不要重开 phase15 summary、依赖升级 closeout、`Phase 16A` / `16A fix-1` / `16B` / `16C` / `16C fix-1` / `Phase 17A` / `Phase 17A fix-1` / `Phase 17B`、或 edition `2024` 兼容修复。  
> 保持现有产品行为、UI contract、shell / menu / loading / toast 边界不变。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `docs/upgrade-plan-rust-1.94-slint-1.15.md` 是否也需要更新。
- 每次更新都必须明确写出：
  - 什么已经完成
  - 当前稳定基线是什么
  - 什么保持不变
  - 下一步为什么是当前 active phase 的剩余工作
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
