# macOS PingFang TOFU Root Cause And Fix (2026-03-31)

## 目的

- 记录 `folder-compare` 在 macOS 从 `13.5` 升级到 `15.6` 后，`PingFang SC` 相关中文 / 全角字符变为 TOFU 的根因。
- 收束本轮基于代码、依赖源码和本机验证得到的证据链。
- 给出可实施的修复方案，并明确推荐顺序。

## 问题摘要

- 当时 UI typography 中转层在 Slint 侧把 read-only selectable content 和 editable input 都固定为 `PingFang SC`：
  - `crates/fc-ui-slint/src/ui_palette.slint`
- 当时这两个文本面都被写成：
  - read-only selectable content: `PingFang SC`
  - editable input: `PingFang SC`
- 用户报告的现象是：
  - 操作系统从 `macOS 13.5` 升级到 `macOS 15.6` 后
  - 中文、全角字符全部显示为 TOFU
  - Latin 文本继续可见

## 分析输入

- 问题定位线程：
  - `codex://threads/019d43fc-89d1-7fc0-af36-34632b50aef3`
- 当前代码基线：
  - workspace `slint = 1.15.1`
  - transitive `fontique = 0.7.0`
- 本轮本机复核环境：
  - `macOS 15.7.5`
  - 该版本与用户报告的 `macOS 15.6` 属于同一代系统字体分发模型

## 直接结论

- 根因不在 `ui_palette.slint` 的写法本身。
- 根因在 `Slint 1.15.1` 当前使用的 `fontique 0.7.0` Apple 字体发现路径与 macOS 15.x 的系统字体分发位置不兼容。
- 结果是：
  - 应用代码要求使用 `PingFang SC`
  - CoreText / 系统层仍然知道 `PingFang SC`
  - 但 `fontique` 无法把它枚举进自己的 system font collection
  - Slint 的 parley / fontique 查询链路因此无法命中 `PingFang SC`
  - Han script fallback 也无法恢复到可用 CJK family
  - 最终只剩 Latin-safe fallback，中文和全角字符全部掉到 missing glyph，表现为 TOFU

## 证据链

### 1. 业务代码确实硬编码了 `PingFang SC`

- 文件：
  - `crates/fc-ui-slint/src/ui_palette.slint`
- 当时定义：
  - read-only selectable content: `PingFang SC`
  - editable input: `PingFang SC`

这一步说明问题不是“没有显式指定中文字体”，而是“显式指定了一个在当前 Slint 字体枚举链路里已不可解析的 family name”。

### 2. macOS 15.x 仍然有 `PingFang SC`

- 本机 `atsutil fonts -list` 仍可见：
  - `PingFang SC`
  - `PingFangSC-Regular`
  - 以及各 weight 的 `PingFangSC-*`
- 本机 `fc-list` / `fc-match` 也能解析出：
  - `PingFang.ttc: "PingFang SC" "Regular"`

这一步说明问题不是“系统升级后删除了 PingFang”。

### 3. `fontique 0.7.0` 在 macOS 上只扫描 `Library/Fonts`

- `fontique-0.7.0/src/backend/coretext.rs`
- 当前实现使用：
  - `NSSearchPathForDirectoriesInDomains(... LibraryDirectory ...)`
  - 再把结果映射成 `.../Fonts/`
  - 然后用 `scan::ScannedCollection::from_paths(paths, 8)` 扫描文件系统字体文件

这一步很关键。它不是走“CoreText 可见 family 的完整枚举”，而是走“从若干目录递归扫描字体文件”。

### 4. macOS 15.x 的 `PingFang.ttc` 已不在 `Library/Fonts` 扫描范围

- 本机 `fc-list` 显示 `PingFang.ttc` 位于：
  - `/System/Library/AssetsV2/com_apple_MobileAsset_Font7/.../AssetData/PingFang.ttc`
- 这不属于 `fontique` 当前 Apple backend 扫描的 `.../Library/Fonts/` 范围。

因此：

- 系统层面可以用
- 但 `fontique` 的 system font collection 看不到该字体文件

### 5. 本机 probe 直接证明 `fontique` 看不到 `PingFang SC`

本轮用独立 probe 对 `fontique 0.7.0` 做了最小验证，结果如下：

- `family_id("PingFang SC") => None`
- `family query "PingFang SC" => no match`
- `family_id("Hiragino Sans GB") => Some(...)`
- `family_id("Heiti SC") => Some(...)`

这说明：

- `PingFang SC` 在当前 `fontique` collection 里根本不存在
- 同时期的 `Hiragino Sans GB` 和 `Heiti SC` 仍可被正常枚举

### 6. Han fallback 也没有补回来

同一 probe 结果：

- `fallback(Hani, zh-Hans) => []`
- `fallback(Hani, zh-Hant) => []`

而 `fontique` 的 Apple fallback 实现流程是：

1. 让 CoreText 为样本文本找 fallback font
2. 读取该 fallback font 的 family name
3. 再回到 `fontique` 自己的 `name_map` 里查这个 family

如果该 family 没被前面的文件扫描收入 collection，那么这一步就会返回空。

因此，当前问题不是“首选 family 丢了，但 fallback 还在”，而是“首选 family 丢了，Han fallback 也一起断了”。

### 7. Slint 当前只会追加 generic fallback，不会自动变成 CSS font stack

- `i-slint-core-1.15.1/src/textlayout/sharedparley.rs`
- `i-slint-common-1.15.1/src/sharedfontique.rs`

当前逻辑是：

- 如果你指定了 `font-family`
- Slint 会构造：
  - `Named("PingFang SC")`
  - `Generic(SansSerif)`
  - `Generic(SystemUi)`

这不是浏览器那种可写 `"PingFang SC, Hiragino Sans GB"` 的 CSS font-family list 语义。

因此两个推论成立：

- 把 `PingFang SC, Hiragino Sans GB` 作为一个字符串塞给 Slint，并不能得到两个 family 的级联回退
- 当 `Named("PingFang SC")` 在 `fontique` 中不可解析，generic fallback 又不具备 Han 安全性时，中文就会直接掉成 TOFU

## Root Cause

可以把根因收敛成一句话：

> `folder-compare` 把 Slint 文本字体固定为 `PingFang SC`，但 `Slint 1.15.1` 所使用的 `fontique 0.7.0` 在 macOS 15.x 上无法从其当前扫描路径中发现 `PingFang.ttc`，同时 Han script fallback 也因为同一个 collection 缺口而失效，最终导致中文和全角字符全部显示为 missing glyph。 

## 不推荐的误修方向

### 1. 继续硬编码 `PingFang SC` 并假设系统会自动接管

- 不成立。
- 当前失败点不在“系统有没有这个字体”，而在“Slint/fontique 能否把它收进自己的字体集合”。

### 2. 在 Slint 字符串里写逗号分隔 font stack

- 例如：
  - `"PingFang SC, Hiragino Sans GB"`
- 不推荐。
- 当前 Slint 路径里这会更像一个 family name，而不是浏览器 CSS 那种 family list。

### 3. 仅依赖 `SansSerif` / `SystemUi`

- 不安全。
- 在当前依赖版本与当前 macOS 15.x 组合下，这条 generic fallback 对 Han script 不可靠。

## 推荐修复方案

### 方案 A：运行时 capability-first family 解析

### 做法

- 在 `fc-ui-slint` 启动阶段用与 Slint 相同的字体发现后端做 family probe。
- 候选顺序建议：
  - `PingFang SC`
  - `Hiragino Sans GB`
  - `Heiti SC`
- 选第一个 `fontique` 实际可命中的 family。
- 当前实现改为在启动阶段通过现有 macOS bootstrap 把系统字体接回 Slint 默认 generic family 路径，不再写回额外 typography global。

### 优点

- 最符合当前项目架构。
- 不依赖硬编码某个 macOS 主版本。
- 保持“capability-first”而不是“OS-version-first”。
- 即使未来 Apple 再改字体落盘位置，也只要 probe 结果正确就能继续工作。

### 风险

- 需要把 typography token 从“静态常量”转成“可在启动时覆写的 global property 使用方式”。
- 需要在 Rust 启动路径增加很小一段字体探测逻辑。

### 推荐度

- 最高。
- 这是推荐的产品级修复方案。

### 方案 B：macOS 15.x 热修为 `Hiragino Sans GB`

### 做法

- 把当前两个 `PingFang SC` token 直接改成 `Hiragino Sans GB`。

### 优点

- 改动最小。
- 在当前本机 `fontique` probe 中已确认可用。
- 可以最快恢复中文和全角字符显示。

### 风险

- 字体视觉会从 `PingFang` 切到 `Hiragino Sans GB`。
- 仍然是“硬编码一个具体 family”，不是结构性修复。
- 未来如果该 family 也因系统分发变化脱离扫描范围，会再次失效。

### 推荐度

- 中。
- 适合作为快速止血，但不建议作为长期终态。

### 方案 C：项目内 patch `fontique` / 升级 Slint 到包含修复的版本

### 做法

- 关注上游 `Slint` / `fontique` 的 Apple system-font backend 修复。
- 如果短期内没有可用上游版本，可通过 `[patch.crates-io]` 引入本地修复版 `fontique`。
- 修复方向应是：
  - 不再仅依赖 `Library/Fonts` 文件扫描
  - 改为通过 CoreText 可见字体枚举与字体来源解析建立 collection
  - 或显式纳入 macOS 15.x 的系统字体 asset 路径

### 优点

- 从根源修正依赖层问题。
- 业务代码无需保留过多平台绕行逻辑。

### 风险

- 工作量最大。
- 需要维护 patch 或承担依赖升级回归成本。
- 对当前任务来说并不是最短恢复路径。

### 推荐度

- 中高。
- 适合作为长期依赖治理方案，与方案 A 可以并行推进。

## 推荐落地顺序

1. 先落地方案 A
2. 如果需要立即止血，可同时用方案 B 作为临时 fallback
3. 后续跟进方案 C，争取把根因修回依赖层

## 建议的实现 contract

### 启动时字体解析函数

- 输入：
  - 候选 family 列表
- 输出：
  - 一个当前系统和当前 Slint/fontique 后端可实际加载的 family name

### 候选顺序

- macOS：
  - `PingFang SC`
  - `Hiragino Sans GB`
  - `Heiti SC`
- 非 macOS：
  - 保持现有策略或单独定义各平台安全 family

### 运行时写回点

- `MainWindow::new()` 之后、`run()` 之前
- 统一执行 macOS bootstrap，让 Slint 默认 generic family 走到已接入的系统字体

### 日志建议

- 启动时输出一次：
  - requested family 列表
  - resolved family
  - probe backend 结果

这样后续出现新一轮字体回归时，不需要再靠截图猜测。

## 验收标准

- `Diff`、`Analysis`、results row secondary text、普通输入框、API key 输入框中的中文可正常显示。
- 全角括号、全角冒号、全角空格、中文标点不再出现 TOFU。
- Latin、数字、符号的布局不出现明显回退。
- macOS 13.x 与 macOS 15.x 上都能得到稳定可用的 family 解析结果。

## 建议补充回归

- 增加一个小型启动 probe 测试或日志断言，确保目标 family 至少能被当前字体后端解析。
- 增加一组包含以下字符的 smoke 文本用于人工验收：
  - `中文`
  - `（）【】｛｝`
  - `，。：；！`
  - `ABC 123`

## 最终建议

- 当前最合理的产品修复是：
  - 业务层立即采用方案 A
  - 依赖治理层继续推进方案 C
- 如果本轮目标是最快恢复用户可用性，则先短期落地方案 B，再尽快补上方案 A。

## 附：本轮关键事实摘要

- `PingFang SC` 不是从 macOS 15.x 消失了。
- 真正消失的是它在当前 `fontique` collection 里的可见性。
- `Hiragino Sans GB` 和 `Heiti SC` 当前仍可被 `fontique` 正常发现。
- 当前 TOFU 是“业务层 family 指定”和“依赖层字体发现失效”叠加后的结果，不是单点配置错误。
