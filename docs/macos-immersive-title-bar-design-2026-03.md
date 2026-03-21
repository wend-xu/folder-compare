# macOS Immersive Title Bar Design (2026-03-21)

## Scope

- 本文是一份实现导向的代码改造设计稿，目标是在当前 `Folder Compare` 基线上引入 `macOS` 专用的沉浸式标题栏。
- 本设计明确要求：
  - `macOS` 启用沉浸式标题栏；
  - `Windows` / `Linux` 保持当前行为，不强制切换 backend，不改变现有窗口外观与交互；
  - 不改 compare / diff / analysis 业务流程；
  - 不引入 `no-frame` 全无边框窗口方案。
- 本文不直接修改产品代码；它定义一版 implementation-ready 的落地方案。

## Current Baseline

- 当前主窗口直接定义在 `crates/fc-ui-slint/src/app.rs` 的内联 `slint::slint!` 中，根组件是 `MainWindow inherits Window`。
- 当前顶部 `App Bar` 只是内容区第一张 `SectionCard`，不是原生标题栏内容。
- 当前启动路径没有自定义 Slint backend 选择，也没有定制 `winit::WindowAttributes`。
- 当前依赖基线：
  - workspace `version = "0.2.18"`
  - workspace `edition = "2024"`
  - `rust-version = "1.94"`
  - `slint = "1.15.1"`
  - `slint-build = "1.15.1"`
- 当前主验证平台是 `macOS arm64`，但项目仍保留 Windows / Linux 运行能力。

## Goals

### Product / UX goals

- 在 `macOS` 上让窗口内容延伸进 title bar 区域，形成“系统窗框仍在，但标题栏沉浸到应用内容中”的效果。
- 保留原生 macOS traffic lights、窗口阴影、缩放、全屏与系统级窗口行为。
- 让现有 `App Bar` 升级为真正的顶部 title bar content strip，而不是单独悬浮的一张卡片。
- 保持当前顶部动作最小集不变：
  - `Folder Compare` 标题
  - `Settings` 入口

### Engineering goals

- `Windows` / `Linux` 不强制切到 `winit` backend，保持当前默认 backend 选择逻辑。
- 不重写 Slint backend，不自建 `WindowAdapter`。
- 不引入 `objc2` / `raw-window-handle` 直接操作 `NSWindow` 的第一阶段实现。
- 尽量把改动限制在 `fc-ui-slint`，避免波及 `fc-core` / `fc-ai`。

## Non-Goals

- 不实现跨平台统一的沉浸式标题栏。
- 不实现 `Windows 11` 标题栏着色、Mica、DWM backdrop 一类并行能力。
- 不实现 Linux CSD / Wayland / X11 的统一标题栏体验。
- 不切到 `no-frame: true`。
- 不做自定义窗口 resize hit-test。
- 不在本阶段把 `Workspace Tabs`、`Filter`、`Compare Inputs` 等更多控制条搬进 title bar。
- 不调整 `Presenter`、`AppState`、`Settings` 持久化与 compare domain contract。

## Chosen Approach

### Summary

- 在 `macOS` 下，启动期显式选择 `winit` backend，并在 window 创建前通过 Slint 的 `with_winit_window_attributes_hook()` 修改 macOS 原生窗口属性。
- 具体使用 `winit::platform::macos::WindowAttributesExtMacOS` 打开：
  - `with_titlebar_transparent(true)`
  - `with_fullsize_content_view(true)`
  - `with_title_hidden(true)`
  - `with_movable_by_window_background(true)`
- Slint 视图层新增一个 `immersive_titlebar_enabled` 开关；仅在该开关为 `true` 时渲染新的 immersive top strip。
- `Windows` / `Linux` 不调用 backend selector，不改 window attributes，继续走当前默认路径。

### Why this approach

- 它保留系统 decorations，因此不需要自己补齐拖拽、缩放、系统按钮与全屏行为。
- 当前 UI 已经有自绘 `App Bar`，只要把它从“卡片”改成“title bar content”即可，视觉迁移成本低。
- 方案只要求在启动前插入一层 backend/window attribute 定制，不会侵入现有事件驱动同步、Presenter、Context Menu、Loading Mask、Toast 这些边界。
- 这是当前基线下最稳的 macOS-only 改造路径。

## Explicitly Rejected Approach

### `no-frame: true`

拒绝原因：

- 这会把窗口拖拽、缩放、traffic lights、系统双击标题栏行为、阴影、全屏切换的一部分责任转移到应用侧。
- 当前项目是工具型桌面应用，不值得为了 title bar 效果引入一整套 borderless window 维护成本。
- 在保持 `Windows` / `Linux` 零行为变化的要求下，`no-frame` 还会额外放大跨平台差异。

## Platform Compatibility Contract

### macOS

- 允许显式切换到 `winit` backend。
- 允许调整原生 title bar 属性。
- 允许顶部内容区视觉发生变化。

### Windows

- 不调用 `BackendSelector::backend_name("winit")`。
- 不调用任何 Windows titlebar / DWM 定制 API。
- 保持当前默认 backend 选择逻辑与当前 `App Bar` 视觉结构不变。

### Linux

- 不调用 `BackendSelector::backend_name("winit")`。
- 不改 Wayland / X11 decorations 策略。
- 保持当前默认 backend 选择逻辑与当前 `App Bar` 视觉结构不变。

### Result

- `Windows` / `Linux` 的零行为变化是通过“完全不进入新的窗口初始化路径”来保证的，而不是通过“进入同一条路径后再尝试回退”来保证的。

## Proposed File-Level Changes

## 1. `crates/fc-ui-slint/Cargo.toml`

### Required change

- 把 `slint.workspace = true` 改为显式 feature 形式：

```toml
[dependencies]
slint = { workspace = true, features = ["unstable-winit-030"] }
```

### Notes

- 第一阶段不要求启用 `raw-window-handle-06`。
- 第一阶段不要求新增 `objc2` 相关依赖。
- `unstable-winit-030` 只用于 macOS 的 window attributes hook 与 `winit` accessor 接口。

## 2. `crates/fc-ui-slint/src/main.rs`

### Required change

- 新增一个 window chrome 模块：

```rust
mod window_chrome;
```

### Reason

- 把平台窗口初始化逻辑从 `app.rs` 主体中拆出去，避免继续膨胀单文件复杂度。

## 3. `crates/fc-ui-slint/src/window_chrome.rs`

### New file

新增一个跨平台门面模块，内部用 `cfg(target_os = "macos")` 实现 macOS 分支，用 no-op 实现非 macOS 分支。

### Proposed responsibilities

- `install_platform_windowing() -> anyhow::Result<()>`
  - `macOS`：只执行一次 backend selector + window attributes hook
  - 非 `macOS`：直接 `Ok(())`
- `immersive_titlebar_enabled() -> bool`
  - `macOS` 返回 `true`
  - 非 `macOS` 返回 `false`
- `titlebar_visual_height() -> f32`
- `titlebar_leading_inset() -> f32`

### Proposed implementation shape

```rust
pub fn install_platform_windowing() -> anyhow::Result<()> { ... }
pub fn immersive_titlebar_enabled() -> bool { ... }
pub fn titlebar_visual_height() -> f32 { ... }
pub fn titlebar_leading_inset() -> f32 { ... }
```

### macOS branch

- 使用 `std::sync::OnceLock` 或等价一次性初始化手段，避免重复 `set_platform`。
- 在第一次调用时执行：

```rust
slint::BackendSelector::new()
    .backend_name("winit".into())
    .with_winit_window_attributes_hook(|attrs| {
        use slint::winit_030::winit::platform::macos::WindowAttributesExtMacOS;

        attrs
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true)
            .with_title_hidden(true)
            .with_movable_by_window_background(true)
    })
    .select()?;
```

### Why a separate module

- `app.rs` 当前已经承载大量 Slint 组件、回调绑定与同步逻辑，不适合再把平台窗口定制塞进去。
- 这个模块将来可扩展为：
  - `macOS` 标题栏相关运行时补丁
  - `Windows` 标题栏实验
  - 但默认仍可保持其他平台 no-op

## 4. `crates/fc-ui-slint/src/app.rs`

### Bootstrap change

- 在 `MainWindow::new()` 之前调用：

```rust
window_chrome::install_platform_windowing()?;
```

- 创建窗口后立即把平台属性同步到 Slint 根组件：

```rust
app.set_immersive_titlebar_enabled(window_chrome::immersive_titlebar_enabled());
app.set_titlebar_visual_height(window_chrome::titlebar_visual_height().into());
app.set_titlebar_leading_inset(window_chrome::titlebar_leading_inset().into());
```

### New root properties

在 `MainWindow` 上新增：

```slint
in property <bool> immersive_titlebar_enabled: false;
in property <length> titlebar_visual_height: 36px;
in property <length> titlebar_leading_inset: 0px;
```

### Layout strategy

这里不建议把现有 `App Bar` 直接参数化替换为一个多状态组件；更稳的做法是保留 legacy 分支，新增 immersive 分支。

#### Reason

- `Windows` / `Linux` 要求零行为变化。
- 如果把现有 `SectionCard` 改写成条件化样式组件，很容易在 `false` 分支上引入细微 spacing/border drift。
- 直接保留原块并在 immersive 模式下使用一条独立分支，最容易保证 non-mac 完全不动。

### Proposed top layout contract

#### Non-mac path

- 保持现有：
  - 顶部 `VerticalLayout { padding: 10px; spacing: 8px; }`
  - 第一项还是当前 36px 高的 `SectionCard`
  - 标题与 `Settings` 的结构不变

#### macOS immersive path

- 外层根布局改为：
  - `padding-top: 0px`
  - `padding-left/right/bottom: 10px`
  - `spacing: 8px`
- 最上方渲染一条新的 `TitleBarSurface`
  - 高度建议：`52px`
  - 无卡片式圆角边框
  - 使用顶部一体化平面 + 底部分隔线
  - 左侧预留 traffic lights 安全区
  - 标题文本从安全区右边开始布局
  - `Settings` 保持右对齐

### Proposed visual constants

- `titlebar_visual_height = 52px`
- `titlebar_leading_inset = 86px`
- `titlebar_inner_horizontal_padding = 12px`
- `titlebar_bottom_separator = 1px`

### Why fixed leading inset is acceptable in phase 1

- 第一阶段不直接访问 `NSWindow` / `NSTitlebarAccessoryViewController` 也不 reposition traffic lights。
- 原生 traffic lights 仍由系统管理，因此应用侧只需要稳定留出一个“不要放文字和按钮”的左侧安全区。
- 固定 `86px` 在当前工具型桌面窗口上足够保守，且实现成本远低于直接读写 AppKit 布局。

## 5. New Slint component inside `app.rs`

### `TitleBarSurface`

建议在 `MainWindow` 前新增一个小组件：

```slint
component TitleBarSurface inherits Rectangle {
    in property <bool> immersive: false;
    in property <length> leading_inset: 0px;
    in property <string> title_text: "Folder Compare";
    callback settings_tapped();
}
```

### Rendering contract

#### `immersive == false`

- 继续复用现有 `SectionCard` 风格：
  - `height: 36px`
  - `border-color: #e5e9ef`
  - `background: #f7f9fc`
  - 标题左对齐
  - `Settings` 右对齐

#### `immersive == true`

- 顶部 strip 与窗口顶边贴合；
- 背景应明显比当前卡片更“并入窗口”，例如：
  - `background: #f7f9fc`
  - 但去掉独立 card 外框与圆角
  - 只保留底部一条 `#e2e8f0` 分隔线
- 标题文本起点：
  - `leading_inset + 12px`
- `Settings` 继续右对齐，不进入 traffic lights 区域

### Pointer / drag behavior

- 第一阶段不额外实现 `drag_window()` 主动拖拽逻辑。
- 依赖原生：
  - `with_movable_by_window_background(true)`

### Why no explicit drag callback in phase 1

- 原生标题栏背景拖拽更可能保留系统级双击标题栏行为。
- 主动调用 `drag_window()` 更适合作为 smoke test 后的补丁，而不是默认方案。

## 6. No presenter/state changes

本设计明确不改：

- `Presenter`
- `AppState`
- `UiBridge`
- `Settings` 读写
- `ContextMenuController`
- `LoadingMaskController`
- `ToastController`

唯一新增的是 view 层的只读平台属性与启动时的窗口初始化逻辑。

## Implementation Sequence

### Step 1. Add platform windowing module

- 新增 `src/window_chrome.rs`
- 接入一次性初始化
- macOS 下接入 `winit` titlebar attributes
- 非 macOS 保持 no-op

### Step 2. Enable required Slint feature

- `fc-ui-slint/Cargo.toml` 打开 `unstable-winit-030`

### Step 3. Wire bootstrap

- `app::run()` 中，在 `MainWindow::new()` 前调用 `install_platform_windowing()`

### Step 4. Add root properties

- 在 `MainWindow` 上增加 `immersive_titlebar_enabled`、`titlebar_visual_height`、`titlebar_leading_inset`

### Step 5. Split legacy / immersive app bar path

- 现有顶部 `SectionCard` 保留为 legacy branch
- 新增 `TitleBarSurface` 作为 immersive branch

### Step 6. Manual smoke on macOS

- 确认 traffic lights 不被覆盖
- 确认标题栏空白区可拖动
- 确认 `Settings` 可点击
- 确认下方 sidebar/workspace 的现有布局未被挤压错位

## Expected Runtime Behavior

## macOS

- 窗口仍是系统装饰窗口；
- 原生 title 文本隐藏；
- 内容延伸进入 title bar 区域；
- 应用自绘顶部 strip 成为视觉上的 title bar 内容层；
- 左上角 traffic lights 仍由系统绘制；
- 顶部空白区可拖动窗口；
- `Settings` 保持可点击；
- `Compare Inputs / Sidebar / Workspace` 的主体 IA 不变。

## Windows / Linux

- 窗口初始化路径维持现状；
- 顶部 `App Bar` 仍是原来的 `SectionCard`；
- 不新增 backend 选择逻辑；
- 不改平台标题栏能力；
- 用户可见行为应与当前基线一致。

## Risks

### 1. macOS backend drift

- 风险：如果某些开发环境当前实际跑的是 Qt backend，强制切到 `winit` 可能导致 macOS 下的某些 widget 视觉细节发生变化。
- 结论：这是 macOS 范围内的可接受变化，但必须通过人工 smoke 确认输入框、context menu、tooltip、滚动与字体表现没有回退。

### 2. Traffic lights overlap

- 风险：左侧安全区不足时，标题文本可能压到 traffic lights。
- 缓解：
  - 第一版用保守 `86px` inset；
  - 若某个 macOS 版本仍有重叠，再单独加大 inset，而不是立即切到 AppKit reposition 方案。

### 3. Blank-area drag experience

- 风险：`with_movable_by_window_background(true)` 在当前 Slint 内容结构下，实际可拖动区域可能比预期窄。
- 缓解：
  - 第一版先保留原生路径；
  - 如果 smoke test 证明不足，再加第二阶段补丁：
    - 仅对 immersive strip 的空白区显式触发 `winit_window.drag_window()`
  - 该补丁应作为 phase 1.1，而不是第一阶段默认行为。

### 4. Top anchor geometry drift

- 风险：tooltips / context menus / overlay anchors 靠近顶部时可能因为 `padding-top` 变化出现细微偏移。
- 缓解：
  - 顶部 strip 落地后手工验证靠近顶部区域的 tooltip / context menu；
  - 如果出现轻微漂移，只修正锚点换算，不改变 controller 边界。

## Deferred Items

- 不在本稿中实现：
  - 直接访问 `NSView` / `NSWindow`
  - 自定义 traffic lights 位置
  - `NSTitlebarAccessoryViewController`
  - Windows 11 Mica / caption color
  - Linux client-side decorations

这些能力若后续需要，应单独起新文档，不混入本次最小落地方案。

## Validation Plan

### Automated

- `cargo check -p fc-ui-slint`
- `cargo test -p fc-ui-slint`

### Manual on macOS

1. 启动主窗口后，系统标题文字不可见，但 traffic lights 可见。
2. 顶部 strip 与窗口上边缘连成一体，不再是单独一张“卡片”。
3. 顶部左侧安全区内没有标题或按钮覆盖。
4. 点击 `Settings` 正常打开设置中心。
5. 顶部空白区可拖动窗口。
6. 窗口缩放、全屏、最小化仍是系统原生行为。
7. 顶部附近 tooltip / context menu / loading mask / toast 没有明显错位。
8. `Compare`、`Diff`、`Analysis` 的主体布局未发生错位。

### Manual on Windows / Linux

- 若具备运行环境，只做 smoke：
  - 启动窗口后，顶部 `App Bar` 外观与当前基线一致；
  - `Settings`、`Compare`、`Results`、`Diff` 正常；
  - 不出现新的 backend 选择或窗口外观变化。

## Minimal Patch Sketch

以下片段仅用于说明落地方向，不是最终代码：

```rust
// src/window_chrome.rs
#[cfg(target_os = "macos")]
pub fn install_platform_windowing() -> anyhow::Result<()> {
    use std::sync::OnceLock;
    use slint::winit_030::winit::platform::macos::WindowAttributesExtMacOS;

    static INIT: OnceLock<anyhow::Result<()>> = OnceLock::new();
    INIT.get_or_init(|| {
        slint::BackendSelector::new()
            .backend_name("winit".into())
            .with_winit_window_attributes_hook(|attrs| {
                attrs
                    .with_titlebar_transparent(true)
                    .with_fullsize_content_view(true)
                    .with_title_hidden(true)
                    .with_movable_by_window_background(true)
            })
            .select()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    });

    match INIT.get().expect("window chrome init missing") {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow::anyhow!(err.to_string())),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn install_platform_windowing() -> anyhow::Result<()> {
    Ok(())
}
```

```slint
// app.rs (concept only)
if !root.immersive_titlebar_enabled : SectionCard {
    // keep the current App Bar block byte-for-byte as close as possible
}

if root.immersive_titlebar_enabled : TitleBarSurface {
    immersive: true;
    leading_inset: root.titlebar_leading_inset;
    title_text: "Folder Compare";
}
```

## Recommended Acceptance Bar

- 若实现后 `macOS` 获得原生沉浸式标题栏效果；
- 且 `Windows` / `Linux` 没有被强制切 backend；
- 且 non-mac 外观不变；
- 且 `Compare / Diff / Analysis / Settings` 的功能不回退；

则本设计可视为成功落地。
