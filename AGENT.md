# AGENT.md - DeepSeek Harness Desktop (Rust + GPUI) 智能体开发与协作指南

欢迎来到 `deepseek-harness-desktop` 项目！本文件为人类开发者与 AI Coding Agent（如 Antigravity / Claude Code 等）在此仓库中协同开发的核心作业指南与规范标准。

---

## 1. 项目概览与核心使命 (Project Overview)

本项目旨在基于 **Rust + GPUI (Zed GPU 加速渲染引擎)** 打造一个轻量、极速（120 FPS）、低内存占用（<80MB）、支持全平台独立分发的 **DeepSeek Harness 原生桌面工作区**。

### 核心架构要点
- **前端与交互层**：纯 Rust + GPUI 原生视图（三栏工作区、增量流式 Markdown 渲染、Tree-sitter 语法高亮、Diff 对比视图）。
- **核心协议与调度**：`dsh-core` 负责状态管理与 JSON-RPC / WebSocket 双向通信。
- **后台运行时**：内嵌便携式 Node.js 运行时，在本地后台安全托管官方 `deepseek-harness` (`dsh`) 守护进程。

---

## 2. 本地权威参考代码库 (Local Reference Codebases)

开发与重构过程中，**必须优先参考以下两个本地权威源码仓库**，以确保架构设计、UI 范式与通信协议的高度一致：

| 仓库名称 | 本地权威路径 | 关键参考内容与模块 |
| :--- | :--- | :--- |
| **Zed / GPUI** | `D:\rust\zed-fluid` | • `crates/gpui`：GPUI 元素构建、Flexbox 布局、事件监听、Context/Model 机制<br>• `crates/ui` & `crates/theme`：Zed 官方 UI 组件与暗黑主题系统<br>• `crates/markdown` & `crates/editor`：`InteractiveText`、`StyledRun` 富文本排版与 Tree-sitter 高亮 |
| **DeepSeek Harness** | `D:\typeScript\deepseek-harness` | • `packages/` 核心：Cordis 插件生命周期、Agent Loop 调度器<br>• 通信协议：Harness 的 WebSocket / JSON-RPC 事件定义（TokenChunk, ToolCall, Diff, StateChange）<br>• 插件与 MCP 生态：MCP Tools 挂载、本地沙箱与文件系统执行 |

---

## 3. Git 规范与仓库配置 (Git & Workflow Standards)

### 3.1 本地 Git 身份配置（严禁修改全局配置）
本仓库已配置独立本地 Git 身份，所有提交必须保持一致：
```ini
[user]
    name = oliver-xie666
    email = 153884673@qq.com
```

### 3.2 分支管理规范 (Branch Naming Convention)
- `main`：主分支，保证随时处于可编译、可测试的绿色状态。
- `feat/<feature-name>`：新特性/组件开发（例如 `feat/markdown-parser`, `feat/toolcall-card`）。
- `fix/<bug-name>`：问题修复（例如 `fix/ws-reconnect-backoff`）。
- `refactor/<module-name>`：模块重构（例如 `refactor/protocol-serde`）。
- `docs/<topic>`：文档或设计规范更新（例如 `docs/update-spec`）。

### 3.3 开发与验证闭环 (Development Verification Loop)
每次提交代码前，必须执行以下验证链条：
1. **代码格式与静态检查**：`cargo fmt --check` & `cargo clippy --all-targets --all-features -D warnings`
2. **单元测试与集成测试**：`cargo test --all`
3. **确认无警告**：确保无未使用的变量（`unused_variables`）或未捕获的 Panic。

### 3.4 提交信息规范 (Conventional Commits)
严格遵循 **Conventional Commits 1.0.0** 规范：
```
<type>(<scope>): <subject>

[optional body]
```

- **常用 Type**：
  - `feat`: 新增功能（如新增视图组件、协议结构、守护进程功能）
  - `fix`: 修复 Bug 或逻辑缺陷
  - `refactor`: 代码重构（不增加功能也不修改已有外部行为）
  - `perf`: 性能优化（如减少 GPUI 重新排版次数、流式增量解析优化）
  - `style`: 代码格式化、UI 样式微调
  - `test`: 增加或修改单元/集成测试
  - `docs`: 文档、注释或设计规约变更
  - `chore`: Cargo 依赖升级、构建/打包脚本调整

- **示例**：
  - `feat(markdown): implement incremental streaming AST parser with pulldown-cmark`
  - `feat(ui): add collapsible ToolCallCard with execution timer`
  - `fix(daemon): handle node subprocess unexpected termination with exponential backoff`
  - `docs(spec): update architecture spec with wix windows packaging notes`

---

## 4. Rust 编码规范与技术约定 (Coding Standards)

1. **Rust Edition**：采用 Rust 2021 / 2024 Edition。
2. **异步运行时**：统一使用 `tokio` (multi-thread flavor)。
3. **错误处理**：
   - 内部库（`dsh-protocol`, `dsh-markdown`, `dsh-daemon`）：使用 `thiserror` 定义强类型枚举错误。
   - 应用与主流程（`dsh-core`, `dsh-ui`）：使用 `anyhow::Result` 进行上下文包装（`with_context`）。
   - 严禁在生产路径中使用不安全的 `.unwrap()` 或 `.expect()`，必须使用 `?` 或显式 `match`。
4. **日志与可观测性**：
   - 统一采用 `tracing` 框架（`tracing::info!`, `tracing::warn!`, `tracing::error!`, `tracing::debug!`）。
5. **GPUI 状态约定**：
   - 业务状态与视图状态严格解耦；视图仅通过 `gpui::Model` 监听状态变化。
   - 耗时 IO/网络任务必须在 `cx.spawn()` 或 Tokio 异步线程中执行，禁止阻塞 GPUI 主 UI 线程。

---

## 5. 核心架构与设计规范索引

- **系统技术设计规范**：详见根目录下 [`DESIGN.md`](./DESIGN.md)
