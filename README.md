# Folder Compare (Rust Workspace)

一个面向本地目录对比的 Rust workspace 项目。

当前项目状态（2026-03-17）：

- 代码稳定基线：`Phase 15.2D`
- 依赖升级路线：已接受、尚未执行
- 当前依赖：workspace `rust-version = 1.75`，`slint = 1.8.0`

![display](./docs/assets/display_0_2_15/display.gif)

## 1. Workspace 结构

- `crates/fc-core`
  - 核心比较引擎（纯本地、确定性）
  - `compare_dirs` / `diff_text_file`
- `crates/fc-ai`
  - AI 分析能力层
  - `Analyzer` + `AiProvider` trait
  - `MockAiProvider`（稳定演示）
  - `OpenAiCompatibleProvider`（真实远程调用）
- `crates/fc-ui-slint`
  - Slint 桌面 UI
  - compare + detailed diff + analysis 闭环

## 2. 当前推进进度

### 已完成（稳定基线）

- `Phase 1-7`：workspace、core 能力、文本 diff、大目录/大文件保护
- `Phase 8`：`fc-ai` 最小可用化（Analyzer + Mock）
- `Phase 9-10.8`：UI compare MVP、detailed diff 面板与可用性收敛
- `Phase 11-12`：UI 集成 AI、OpenAI-compatible provider 与配置切换
- `Phase 13-14.2`：IA 重构、Provider Settings 全局化、配置持久化、UI 视觉收敛
- `Phase 15.0-15.2D`：
  - File View（Diff/Analysis）壳层产品化
  - Analysis 结构化结果呈现与 copy 流程
  - `toast-controller`（overlay）
  - `loading-mask`（局部遮罩）
  - `ui_palette`（语义色板）
  - window-local `context-menu core`（非输入表面）

### 当前主线（正在推进）

- `Phase 15.3A` preflight：统一版本来源、补齐升级 checklist/smoke checklist、文档对齐
- 在 `Phase 15.3A/15.3B` 完成前，不直接推进 `Phase 16`

## 3. 15.2D 后升级声明（版本限制）

由于当前稳定基线仍锁定 `slint = 1.8.0`，`15.2E`（editable input context menu）在现版本实现成本与风险过高。

因此在 `15.2D` 之后，项目将按既定路线准备升级：

- Rust 升级到 `1.94.0`（常见口述写法：`1.9.4`）
- Slint 升级到 `1.15.x`（建议 pin 到 `1.15.1`）

说明：该升级路线已经在文档中确认，但当前仓库依赖尚未实际切换。

## 4. 升级路线（Roadmap）

- `Phase 15.3A`：upgrade preflight（不改依赖）
- `Phase 15.3B`：仅升级 Rust 到 `1.94.0`（保持 Slint `1.8.0`）
- `Phase 15.4`：升级 Slint 到 `1.15.x`，恢复 `15.2D` 行为等价
- `Phase 15.5`：在新基线上重开并完成 `15.2E`
- `Phase 15.6`：升级后清理（同步机制、模型重建等）
- `Phase 16`：恢复结果导航增强（sorting / quick jump / 更强过滤）

## 5. 当前能力总览

- IA：`App Bar + Sidebar + Workspace`
- Workspace：`Diff / Analysis` 共享壳层（connected tabs + header + content）
- Compare 闭环：路径输入、Browse、校验反馈、summary-first 状态
- Results/Navigator：搜索 + 状态过滤 + 选择驱动 Diff 上下文
- Diff：`no-selection -> loading -> unavailable/error -> detailed|preview`
- Analysis：`no-selection -> not-started -> loading -> error|success`
- Provider Settings：全局 modal、Save/Cancel、持久化恢复
- context menu（当前范围）：仅 non-input safe surfaces

## 6. 运行方式

### 前置要求

- Rust `1.75+`（当前基线）
- macOS 优先（Windows / Linux 也考虑支持）

### 启动 UI

```bash
cargo run -p fc-ui-slint
```

### 基础流程

1. 输入或 Browse 选择 Left/Right 目录
2. 点击 Compare
3. 在 Results 选择文件查看 Diff
4. 如需配置 provider：App Bar -> `Provider Settings`
5. 切换到 Analysis 并点击 Analyze

## 7. OpenAI-compatible 说明

### 配置入口与持久化

- 配置入口：App Bar -> `Provider Settings`
- 持久化文件名：`provider_settings.toml`
- 配置目录优先级：
  - `FOLDER_COMPARE_CONFIG_DIR`（环境变量覆盖）
  - macOS：`~/Library/Application Support/folder-compare`
  - Windows：`%APPDATA%/folder-compare`
  - Linux：`$XDG_CONFIG_HOME/folder-compare` 或 `~/.config/folder-compare`

### 必填配置

- `Endpoint`：OpenAI-compatible 根路径（如 `https://api.openai.com/v1`）
- `API Key`
- `Model`

## 8. 测试与验证

```bash
cargo check --workspace
cargo test --workspace
```

测试原则：

- 不依赖真实外网
- 远程 provider 测试使用本地 mock server / fake response
- UI 测试重点覆盖 bridge/presenter/state 编排逻辑

## 9. 设计边界

- `fc-core` 不依赖 UI/AI
- `fc-ai` 不侵入 core 逻辑
- UI 负责编排与展示，不承载核心业务规则
- compare / diff / analysis 三层状态严格分离
- `15.2D` 是当前稳定发货基线；`15.2E` 计划在升级后推进

## 10. 后续主线（长期）

- `Phase 16`：结果视图增强（状态筛选 / 排序 / 更强过滤）
- `Phase 17`：目录树 / 层级视图
- `Phase 18`：Compare View / File View 双模式工作区
- `Phase 19`：AI 分析增强（多任务 / hunk 关联 / 缓存）
- `Phase 20`：Diff / Analysis 高级交互
- `Phase 21`：后台任务与性能体系
- `Phase 22`：产品化收尾
