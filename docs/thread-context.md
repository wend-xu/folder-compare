# Folder Compare Thread Context (Live)

## 目的

本文件用于“开新线程”的短周期交接，只记录当前事实、边界和下一步，不再把已完成的 phase train 当作待执行队列。

## 本轮更新说明（2026-03-21）

- 本轮执行 `Phase 17D`，在 `Phase 16A + 16A fix-1 + 16B + 16C + 16C fix-1 + 17A + 17A fix-1 + 17B + 17B fix-1 + 17C + 17C-A` 的稳定基线上，为 `macOS` 引入第一阶段沉浸式标题栏：新增 `fc-ui-slint::window_chrome` 平台门面、仅在 `macOS` 启用显式 `winit` title bar attributes，并把顶部 `App Bar` 分成 immersive / legacy 双路径；不引入 `no-frame`、`objc2`、raw AppKit 操作或非 mac 平台窗口系统改造。
- 已完成并关闭：
  - `Phase 17D`
  - `Phase 17C-A`
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
  - `Compare Inputs` 的 `Compare` 现已收口为独占主动作按钮行，不再依赖右侧 `Ready to compare / Running compare / Select left and right folders` 文字提示
  - `Compare` 按钮现在只在禁用态或 compare 运行中提供轻量 tooltip；可 compare 时不再重复说明
  - shared tooltip overlay 默认优先显示在目标上方；上方空间不足时再降级到下方
  - `App Bar -> Settings` 已替换 `Provider Settings`，并提供第一轮 `Provider / Behavior` 分区骨架
  - Settings modal 当前以最大内容态固定容器尺寸；`Provider / Behavior` 与 `Mock / OpenAI-compatible` 间切换不再改变外层容器尺寸
  - persisted settings 当前以 `settings.toml` 为唯一活跃基线；若只存在 legacy `provider_settings.toml`，启动时会一次性迁移到 `settings.toml`
  - `Settings -> Behavior` 已落地首个偏好项：`Hidden files` 默认显示/隐藏策略；该偏好保存后会立即作用到当前 `Results / Navigator` 可见集合，并影响后续 compare 结果展示
  - `Hidden files` 当前仅是 UI 偏好，不修改 compare request / `fc-core` contract；若它让当前项不可见，则沿用现有 stale selection 语义，Results 顶部摘要会显式提示被 Settings 隐藏的数量
  - workspace 外层双卡片感已收口：外层 workspace wrapper 现为透明视觉壳，workbench host 现已占满整个右侧透明 wrapper，workspace loading mask 也已挂到同一 workbench surface
  - `Diff` header/body separator 现已共用一套列几何；横向滚动时 header/body divider 不再漂移
  - `Diff` 与 `Analysis` 在 non-ready 状态下现已保留一致的 top-stack：title row / metadata-badge row / helper strip / shell-content body
  - `Diff` 现已在所有状态保留 helper strip；non-ready 状态复用克制的上下文文案，而不是移除 strip
  - embedded `DiffStateShell` 现已区分 standalone 与 embedded 呈现；embedded mode 使用 layout-driven compact badge lane、`neutral` 保持 `neutral`、并把左侧 accent 收缩到不高于 workbench border 的低噪声边缘
  - embedded shell 的 title/body 起始线现已更贴近 ready content 的 workbench 内边距节奏，不再像独立大卡片下沉在内容区里
  - `Compare Inputs` 的 `Compare` 现已占满整个卡片内容宽度，不再受 `Left / Right` label gutter 约束
  - `fc-ui-slint` 现已新增 `window_chrome` 平台门面；窗口 backend/titlebar 定制逻辑不再继续膨胀进 `app.rs`
  - `macOS` 启动路径现已在 `MainWindow::new()` 前显式安装 `winit` backend selector，并通过 Slint 的 winit hook 打开 transparent title bar / full-size content view / hidden title / movable-by-window-background
  - `macOS` 顶部 `App Bar` 现已切为 immersive strip；`Windows / Linux` 继续保留原有 legacy `SectionCard` App Bar，不进入新的窗口初始化路径
  - 沉浸式顶部 strip 当前使用固定 `86px` 左侧安全区给 traffic lights 留位；第一阶段不做 traffic lights reposition
- 保持不变：
  - `15.2D` 的 IA 与 shell contract 不变
  - connected tabs + attached workbench surface 不变
  - `Diff/Analysis` 的 `Tabs -> Header -> Content` 大 contract 不变
  - editable-input context-menu contract 不变
  - `Settings -> Provider -> API Key` secret contract 不变
  - window-local non-input context-menu core contract 不变
  - loading / toast 仍保持 UI-local feedback contract，不引入新的全局 controller
  - UI 继续使用内联 `slint::slint!`
- 为什么当前基线仍然是剩余 `Phase 17` 的继续点：
  - `Phase 16A`、`16A fix-1`、`16B`、`16C`、`16C fix-1` 已把 Sidebar 表达、flat-list 结果扫描能力，以及 File View 状态联动收口到当前稳定基线；
  - `Phase 17A` 已在此基础上补齐克制的 tooltip completion layer，`Phase 17A fix-1` 则进一步把 row hover 稳定性、输入框裁切与 overlay 定位收口；
  - `Phase 17B` 则把设置入口升级到最小可扩展骨架，并以 UI 偏好方式收口 hidden-files 默认可见性，而没有重开完整 settings framework、tree mode 或 compare/core contract；
  - `Phase 17C` 又进一步收口了 Compare 主动作交互与 workbench 的结构性 UI bug `B/C/D`，并把这些变化同步写回主文档；
  - `Phase 17C-A` 则继续用同一条 UI-side 低风险路径，单独收口 embedded state shell 的剩余视觉问题，而没有把范围扩张到 core、tree mode、search、或 settings framework；
  - `Phase 17D` 则继续把范围限制在 `fc-ui-slint`，只为 `macOS` 落地第一阶段沉浸式标题栏，同时用平台隔离保证 `Windows / Linux` 零行为变化；
  - 因此后续线程若继续推进，应建立在当前基线上，而不是继续重开 `15.3A` 到 `15.8 fix-1`、`16A` 到 `16C fix-1`、或 edition 兼容修复。

## 快照（Snapshot）

- 日期：`2026-03-21`（Asia/Shanghai）
- 分支：当前已完成 `Phase 17D`
- 工作区：`fc-ui-slint` macOS immersive title bar phase 1、platform window chrome 门面、主文档同步改动
- 最近提交：
  - `0a8769d` Phase 16A fix-1
  - `1311f96` edition-2024 milestone
  - `7e59de3` Phase 15.8 fix-1: section title align
  - `51b28cd` Phase 15.8：Analysis success selectable-text native menu
- 当前主参考：
  - `docs/architecture.md`：长期“当前架构基线 + deferred decisions + next priority”
  - `docs/upgrade-plan-rust-1.94-slint-1.15.md`：升级与独立 edition 里程碑的归档背景

## 当前目标（Execution Focus）

1. `Phase 17D` 已完成；Sidebar 仍然保持 `Compare Inputs -> Compare Status -> Filter / Scope -> Results / Navigator`。
2. `Results / Navigator` 当前稳定 contract 是 flat list：filename-first 主信息、reason summary 次信息、parent-path 弱信息，以及仅基于现有 `path / name` 搜索 contract 的轻量命中高亮。
3. selection / file-view 当前稳定 contract：
   - Search / Status 改变后，当前项仍可见则保留；否则清掉左侧可见选中态，右侧进入 stale selection
   - compare 重跑后仅按同一路径做基础恢复；无法稳定恢复则保持 stale selection，不自动跳转
   - `Diff / Analysis` 明确区分 `no selection`、`stale selection`、`unavailable` 与 `ready/waiting`
   - `Settings -> Behavior -> Hidden files` 保存后若让当前项不可见，也沿用同一套 stale selection 收口，不自动跳转
4. compare / tooltip / workbench 当前补充 contract：
   - `Compare` 现已是占满整个 `Compare Inputs` 卡片内容宽度的 full-width primary action lane；上方 `Left / Right + input + Browse` 节奏保持不变
   - `Compare Inputs` 不再保留按钮右侧文字提示；按钮禁用原因与 compare 运行中说明由轻量 tooltip 承担
   - shared tooltip overlay 当前承担两类轻量职责：被截断文本 completion，以及 restrained Compare-action state hint；它仍不是 explanation-heavy hover system
   - workspace 当前只保留一个主要可见 surface；loading mask 边界与用户感知的 workbench surface 一致
   - `Diff` 与 `Analysis` 在 non-ready 状态下已保留一致 top-stack；`Diff` helper strip 不再按是否 ready 条件消失
   - diff table header/body 当前已共用列几何；若后续调整列宽或 divider，应继续只维护一套 contract
5. settings / row / input 当前补充 contract：
   - `App Bar -> Settings` 是新的全局设置入口，当前只做第一轮 `Provider / Behavior` 分区，不引入完整 settings framework
   - `Settings -> Provider` 继续承接原有 provider 配置能力；`Settings -> Behavior` 当前只承载 `Hidden files`
   - `Hidden files` 当前只作用于 `Results / Navigator` 可见集合与其顶部摘要，不改变 `Search` 的 `path / name` contract，也不改变 `Compare Status` summary-first 统计来源
   - row 次信息优先表达是否可 `text diff` / `text preview`，并使用短语以适配当前 Sidebar 宽度
   - 普通输入框与 `ApiKeyLineEdit` 共用 `UiTypography.editable_input_font_family`，避免 `slint 1.15.1` 默认 editable widget 字体链导致的全角字符 TOFU
   - tooltip 当前是 window-local shared overlay；Results row 继续收口为 row-level tooltip，只补全完整 filename + 完整 parent path；Compare/Search 输入继续只补全输入完整值
   - shared tooltip overlay 默认优先上方定位；空间不足时降级到下方
   - `TooltipLineEdit` 包装层会继续保留 native editable behavior，同时把可见输入宽度收紧在控件矩形内
6. platform window chrome 当前补充 contract：
   - 仅 `macOS` 会进入 `window_chrome::install_platform_windowing()` 并显式切到 `winit` + macOS title bar attributes
   - `Windows / Linux` 不会进入该路径，也不会被 forced backend selection 影响
   - 沉浸式标题栏当前只把标题与 `Settings` 入口并入顶部 strip，不搬运更多控制条
   - 第一阶段继续依赖 `with_movable_by_window_background(true)`；若未来 blank-area drag 仍不足，再单独补 `drag_window()`，不要在当前基线里提前升级
7. 后续线程继续 `Phase 17` 剩余子项时，不要重开 `Phase 15.3A` 到 `Phase 15.8 fix-1`、`Phase 16A`、`16A fix-1`、`16B`、`16C`、`16C fix-1`、`Phase 17A`、`Phase 17A fix-1`、`Phase 17B`、`Phase 17B fix-1`、`Phase 17C`、`Phase 17C-A`、`Phase 17D`，也不要重开独立 workspace `edition = "2024"` 里程碑。
8. 下一线程若继续 UI bug 收口，应把本轮已经完成的 embedded shell low-noise contract 与 macOS immersive title bar contract 视为新基线，不要回退到 standalone-state-card 的重 accent 呈现，也不要把范围重新扩张到 `B/C/D`、Compare Inputs、或窗口系统大重构。
9. 继续保持主文档与当前代码事实一致，不创建额外 phase checklist / summary 文档。

## 本阶段范围（In Scope / Out of Scope）

- In Scope：
  - 剩余 `Phase 17`
  - `DiffStateShell` embedded visual 收口后的基线维护
  - workbench shell / Compare Inputs 的已验收视觉基线维护
- Out of Scope：
  - tree / hierarchy / grouping navigation
  - 排序系统
  - 内容搜索
  - Compare Inputs 重开
  - 已关闭的 workbench bug `B/C/D` 重做
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
12. 不回退 `Phase 17C` 已建立的 `Compare` full-width 主动作、restrained Compare-action tooltip、workspace single-surface shell、shared diff column geometry、以及 `Diff/Analysis` 一致 top-stack。
13. 不把 `Phase 17D` 升级成 `no-frame`、raw AppKit、traffic lights reposition 或跨平台标题栏统一方案；`Windows / Linux` 继续保持当前默认窗口路径。

## 当前稳定事实（Stable Facts）

- 依赖与工具链：
  - `Cargo.toml` 已固定 workspace `version = "0.2.18"`、workspace `edition = "2024"`、workspace `rust-version = "1.94"`、`slint = "=1.15.1"`、`slint-build = "=1.15.1"`
  - `rust-toolchain.toml` 已固定 `channel = "1.94.0"`
  - `docs/macos_dmg.sh` 继续从 workspace manifest 派生 bundle / DMG / ZIP 版本
- UI / shell：
  - `15.2E` 已在当前基线上发货
  - `App Bar -> Settings` 现已替代 `Provider Settings`，并提供第一轮 `Provider / Behavior` 分区骨架
  - `macOS` 现已拥有第一阶段沉浸式标题栏：应用顶部 strip 会并入原生标题栏区域，系统标题文本隐藏，traffic lights 继续由系统管理
  - `macOS` 窗口初始化现已通过 `window_chrome` 模块集中管理；仅该平台显式启用 `winit` backend selector 与 macOS window attributes hook
  - `Windows / Linux` 继续保留 legacy `SectionCard` App Bar，不进入新的窗口初始化路径
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
  - tooltip 当前是一个共享的 window-local overlay；它主要承担截断文本 completion，并额外承接 restrained Compare-action state hint，不承担 explanation-heavy hover 说明
  - `Results / Navigator` row 的 tooltip 已收口为 row-level completion：当 filename 或 parent-path 截断时，整行 hover 可查看完整 filename 与完整 parent path
  - `Compare Inputs` 左右路径与 `Filter / Scope -> Search` 在非编辑态、长文本被截断时可通过 tooltip 查看完整值，且文本会先被裁切在输入框内
  - `Compare` 主动作现已占满整个 `Compare Inputs` 卡片内容宽度；它不再与右侧说明文字共享空间，也不再受 label gutter 约束
  - `Compare` 按钮当前只在禁用态或 compare 运行中显示轻量 tooltip；可 compare 时不再重复说明状态
  - shared tooltip overlay 默认优先在目标上方展示，若空间不足则自动降级到下方
  - tooltip overlay 当前除了截断文本 completion，也承接 restrained Compare-action state hint；它仍不承担 explanation-heavy hover 说明
  - `Settings -> Behavior -> Hidden files` 当前只影响 `Results / Navigator` 可见集合；摘要会显式提示被 Settings 隐藏的数量，但 `Compare Status` 仍保持 summary-first 原始统计
  - Search / Status 改变后，若当前 source row 仍在新集合中则保持选中；若不在新集合中，则左侧清掉可见选中态，右侧保留 stale path 语义，不自动跳到第一项
  - compare 重跑后只按同一路径做低风险恢复；若同一路径仍存在且在当前 filter 下可见，则恢复选中并自动刷新 File View；否则进入 stale selection
  - row secondary summary 现在更主动提示 non-text / binary compare 与常见 preview-unavailable 文件类型，帮助用户预判右侧 viewer 能力，并通过更短文案适配当前 Sidebar 宽度
  - Analysis success 正文文本支持 native text-surface `Copy` / `Select All` right-click
  - Analysis success section header 标题继续保持显式左对齐，且不会遮挡右上角 inline `Copy`
  - `Diff` 与 `Analysis` 当前都已支持显式 stale selection 文案，不再把 filtered-out selection 与从未选中过的 no-selection 混为一谈
  - workspace 外层 wrapper 当前已退为透明视觉壳，workbench host 当前已占满整个透明 wrapper，workspace loading mask 与用户感知的 workbench surface 保持同一边界
  - workspace tabs 下方用于连接 panel 的顶部 border bridge 当前会避开左右圆角区，不再在 left/right top corner 处产生可见断层
  - `Diff` table header/body 当前已共用列几何；若后续继续调优视觉，不能再让 header/body 各自维护 separator 位置
  - `Diff` 与 `Analysis` 当前在 non-ready 状态下已保留一致的 title / metadata / helper strip / shell-body 节奏；`Diff` helper strip 不再按 ready 条件消失
  - embedded `DiffStateShell` 当前已是独立的 low-noise workbench presentation：badge 走 layout lane、`No Selection` 保持 neutral、`Stale` 保持 restrained warn、左侧 accent 不再显得比 workbench border 更厚重
  - persisted settings 当前保存到 `settings.toml`；若仅存在 legacy `provider_settings.toml`，启动时会一次性迁移到新文件，之后继续以 `settings.toml` 为唯一活跃基线
- 运行时：
  - compare / diff / analysis 后台完成态继续通过 presenter notifier + `slint::Weak::upgrade_in_event_loop` 回推 UI
  - `Results / Navigator` 与 `Diff` 行模型继续使用 persistent `VecModel`
  - loading-mask timeout copy 继续使用按 busy phase 调度的一次性 timer

## 下一步（Next）

- 唯一建议的下一步是继续任何剩余的 `Phase 17` 工作，但以当前 embedded shell / workbench / Compare Inputs 基线为起点。
- 后续实现应建立在当前 `0.2.18 + edition 2024 + rust 1.94.0 + slint 1.15.1 + Phase 16A + 16A fix-1 + 16B + 16C + 16C fix-1 + Phase 17A + Phase 17A fix-1 + Phase 17B + Phase 17B fix-1 + Phase 17C + Phase 17C-A + Phase 17D` 基线上，不重开升级、edition 迁移或 `Phase 15` closeout。
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

- 本轮验证重点是 `Phase 17D` 的 macOS immersive title bar phase 1、platform window chrome 门面，以及主文档同步：
  - `cargo check -p fc-ui-slint`
  - `cargo test -p fc-ui-slint`
- 本轮未运行 `cargo run -p fc-ui-slint`：
  - 原因：本轮未在此线程内做桌面 GUI smoke；macOS traffic lights / drag area / immersive top strip 的真实视觉验收仍保留给人工

## 新线程提示词模板（Handoff Prompt）

建议新线程首条消息直接使用：

> 先阅读 `docs/thread-context.md`，再阅读 `docs/architecture.md`。  
> `docs/upgrade-plan-rust-1.94-slint-1.15.md` 只在需要升级与独立 edition 里程碑归档背景时再阅读。  
> 把 `Phase 15.3A`、`Phase 15.3B`、`Phase 15.4`、`Phase 15.5`、`Phase 15.5 fix-1`、`Phase 15.5 fix-2`、`Phase 15.5 fix-3`、`Phase 15.6`、`Phase 15.7`、`Phase 15.8`、`Phase 15.8 fix-1`、`Phase 16A`、`Phase 16A fix-1`、`Phase 16B`、`Phase 16C`、`Phase 16C fix-1`、`Phase 17A`、`Phase 17A fix-1`、`Phase 17B`、`Phase 17B fix-1`、`Phase 17C`、`Phase 17C-A`、`Phase 17D`，以及独立 workspace `edition = "2024"` 里程碑，全部视为已完成。  
> 把当前稳定基线视为：workspace `version = "0.2.18"`、workspace `edition = "2024"`、`rust-toolchain = 1.94.0`、workspace `rust-version = 1.94`、`slint = 1.15.1`、`slint-build = 1.15.1`，且 `15.2E`、event-driven sync、persistent `VecModel`、`Diff` 显式 `ScrollView` 视口、shared `UiTypography.selectable_content_font_family`、non-input context-menu visual polish、`Analysis success` native text-surface right-click、section header 左对齐修复均已稳定。  
> 当前默认进入剩余 `Phase 17`，优先单独处理 `docs/ui-bug-root-cause-and-fix-plan-2026-03.md` 中仍未关闭的 `A. Diff shell state card still feels oversized, accent-heavy, and visually misaligned`；不要重开 phase15 summary、依赖升级 closeout、`Phase 16A` / `16A fix-1` / `16B` / `16C` / `16C fix-1` / `Phase 17A` / `Phase 17A fix-1` / `Phase 17B` / `Phase 17B fix-1` / `Phase 17C`、或 edition `2024` 兼容修复。  
> 保持现有产品行为、UI contract、shell / menu / loading / toast 边界不变。

## 更新契约（Mandatory）

- 编辑本文件时，必须同步检查 `docs/architecture.md` 与 `docs/upgrade-plan-rust-1.94-slint-1.15.md` 是否也需要更新。
- 每次更新都必须明确写出：
  - 什么已经完成
  - 当前稳定基线是什么
  - 什么保持不变
  - 下一步为什么是当前 active phase 的剩余工作
- 术语必须与 `docs/architecture.md` 对齐，不得为同一 contract 造新别名。
