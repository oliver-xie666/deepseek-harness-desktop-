# ⚡ DeepSeek Harness Desktop (Rust + GPUI)

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust%202021-orange.svg" />
  <img src="https://img.shields.io/badge/GUI-GPUI%20(Zed%20Engine)-blue.svg" />
  <img src="https://img.shields.io/badge/Backend-DeepSeek%20Harness%20(dsh)-blueviolet.svg" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" />
  <img src="https://img.shields.io/badge/Framerate-120%20FPS-brightgreen.svg" />
</p>

[English](#english) | [简体中文](#简体中文)

---

## 简体中文

**DeepSeek Harness Desktop** 是一款基于 **Rust** 和 **GPUI**（Zed 编辑器的 GPU 硬件加速渲染引擎）构建的下一代高性能本地 AI 编码桌面工作区。

它专为官方 [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness) 设计，相比传统基于 Electron 的桌面包装，带来了 **120 FPS 极速刷新**、**低至 50MB 内存占用** 和 **Zed 级别的流式代码渲染体验**。

### ✨ 核心特性

- 🚀 **120 FPS 极致流畅**：依托 GPUI 硬件加速（DirectX / Metal / Vulkan），流式 Token 输出与超长代码滚动零延迟。
- 📦 **开箱即用（Zero-Config）**：内嵌便携式 Node.js 运行时，在后台自动托管 `dsh` 守护进程，用户无需自行配置环境。
- 🎨 **Zed 级代码与 Markdown 排版**：集成 `pulldown-cmark` 增量解析器与 `Tree-sitter` 语法高亮引擎。
- 📑 **三栏集成工作区**：
  - **左侧**：会话历史、项目工作区与 MCP / Cordis 插件状态。
  - **主视窗**：流式对话、可折叠 Tool Call 卡片与多行快捷输入栏。
  - **协同面板**：行级文件 Diff 对比审查（一键 Accept / Reject）与实时 Terminal 命令日志。

### 🏗️ 架构概览

```
┌──────────────────────────────────────────────┐
│  Rust 原生客户端 (GPUI 120 FPS 渲染)          │
│  ├── dsh-ui       (三栏工作区与组件库)        │
│  ├── dsh-markdown (增量 AST 与 Tree-sitter) │
│  ├── dsh-core     (状态管理与 WebSocket 调度) │
│  └── dsh-protocol (强类型 JSON-RPC 消息协议) │
└──────────────────────┬───────────────────────┘
                       │ Localhost WebSocket / RPC
┌──────────────────────▼───────────────────────┐
│  内置受控子进程空间 (Isolated Daemon)         │
│  └── 便携式 Node.js + deepseek-harness 运行时 │
└──────────────────────────────────────────────┘
```

---

## English

**DeepSeek Harness Desktop** is a next-generation, high-performance native desktop workspace for [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness), written in **Rust** and powered by **GPUI** (Zed's GPU-accelerated UI framework).

### ✨ Features
- 🚀 **Blazing-Fast 120 FPS Rendering**: Sub-millisecond input responsiveness and smooth streaming powered by GPUI.
- 📦 **Zero-Config Distribution**: Embedded lightweight Node.js runtime with automated subprocess daemon management.
- 🎨 **Rich Markdown & Tree-sitter Highlighting**: Incremental streaming parsing and syntax highlighting.
- 📑 **Three-Pane Workspace**: Left sidebar (sessions/MCP), main chat view (tool cards), and right diff/terminal drawer.

---

## 🛠️ Quick Start

```bash
# Clone the repository
git clone https://github.com/oliver-xie666/deepseek-harness-desktop-.git
cd deepseek-harness-desktop

# Run test suite
cargo test --workspace

# Start development client
cargo run -p dsh-ui
```
