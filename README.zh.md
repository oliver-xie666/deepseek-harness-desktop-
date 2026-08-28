# ⚡ DeepSeek Harness Desktop (Rust + GPUI)

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust%202021-orange.svg" alt="Rust 2021" />
  <img src="https://img.shields.io/badge/GUI-GPUI%20(Zed%20Engine)-blue.svg" alt="GPUI" />
  <img src="https://img.shields.io/badge/Engine-100%25%20Pure%20Rust%20Native-brightgreen.svg" alt="纯 Rust 原生" />
  <img src="https://img.shields.io/badge/Models-DeepSeek--V3%20%7C%20DeepSeek--R1-purple.svg" alt="支持模型" />
  <img src="https://img.shields.io/badge/Framerate-120%20FPS-brightgreen.svg" alt="120 FPS" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT 协议" />
</p>

<p align="center">
  <a href="README.md">English</a> | <b>简体中文</b>
</p>

---

**DeepSeek Harness Desktop** 是一款专为 **DeepSeek-V3** 与 **DeepSeek-R1** 深度定制的下一代高性能原生 AI 编码桌面工作区。

项目完全基于 **Rust** 语言与 **GPUI**（Zed 编辑器的 GPU 硬件加速 Direct3D 11/12、Metal、Vulkan 渲染框架）构建，内置 **100% 纯 Rust 原生大模型直连引擎**、**DeepSeek-R1 思维链实时流式折叠面板**、**原生多轮 Agent 决策中枢与 7 项核心工具集**，以及社区级 **`dsh-plugin` & `SKILL.md` 插件加载系统**。

默认采用 **Standalone 纯单机直连模式**，**无需依赖 Node.js 运行时和后台 3080 端口**，即可享受 **120 FPS 丝滑输入与长文本滚动**，基线内存占用**低于 50MB**。

---

## ✨ 核心特性

### 🚀 1. 100% 纯 Rust 原生大模型直连引擎
- **无中介直连**：基于 Rust `reqwest` 实现原生 HTTPS/SSE 流式请求，直连 DeepSeek 官方 API 或兼容端点，彻底摒弃外部 Node.js 代理中转。
- **DeepSeek-R1 双流 SSE 解析**：逐帧严格解析 Server-Sent Events，并行处理 `reasoning_content`（深度思考过程）与 `content`（正文流）（`llm/sse.rs`、`llm/translate.rs`）。
- **实时 Token 与性能计量**：精准统计首字延迟（TTFT）、生成速率（Token/秒）、提示词消耗、输出 Token、思考 Token 以及不相交缓存命中率（`llm/token_meter.rs`）。
- **工具调用回传规范**：完全契合官方多轮会话 Passback 规范，在多轮决策中准确回传 `tool_calls` 与工具执行结果（`llm/serialize.rs`）。

### 🧠 2. DeepSeek-R1 实时思考流式 UI
- **实时思考卡片**：在 `chat_view` 中提供 `🧠 思考过程 (Reasoning Process)` 可折叠卡片。
- **流畅动效与 Token 涌现**：120 FPS 无卡顿实时刷新思考步骤，直观展示 R1 模型的推理逻辑。
- **状态记忆与独立折叠**：支持跨对话轮次独立展开/收起，保持界面整洁。

### 🤖 3. 原生 Agent 闭环决策中枢与 7 项内置工具
- **多轮自主决策（`NativeAgentLoop`）**：自主分析意图、生成工具调用、捕获执行反馈并持续推进任务直至交付。
- **7 项原生核心工具集**：
  - `read_file`：行范围限制读取工作区本地文件。
  - `write_file`：原子化写入或覆盖文件内容。
  - `edit_file`：精确文本定位与替换修改。
  - `apply_patch`：原子化应用 Unified Diff 补丁并严格校验上下文。
  - `grep_search`：跨目录高速正则与文本子串检索。
  - `list_dir`：递归浏览目录与文件树结构。
  - `exec_command`：在受控子进程中执行终端命令，支持超时控制与输出捕获。
- **成果物胶囊与 Diff 审查器**：与 **Deliverables 成果物胶囊** 及行级 **DiffApplier** 深度联动，支持一键审查、应用或回滚修改。

### 🔌 4. 社区 dsh-plugin 与 Skill 插件生态
- **多级插件扫描**：自动扫描 `~/.dsh/plugins` 与当前工作区目录下的 `.dsh/plugins`。
- **Skill 技能解析**：同时兼容 `SKILL.md`（YAML Frontmatter + Markdown 标题）与 `plugin.json` 两种元数据格式。
- **Stdio 子进程与 MCP 协议**：支持通过标准 Stdio 管道动态调度外部插件和 MCP（Model Context Protocol）服务。

### 🎨 5. Zed 级 120 FPS GPUI 三栏集成工作区
- **三栏集成设计**：
  - **左侧边栏**：会话历史记录、项目文件树及 MCP / 插件运行状态。
  - **主对话区**：流式对话视图、可折叠 Tool Call 卡片与多行快捷输入栏。
  - **右侧协同面板**：行级 Diff 对比器（支持 Accept / Reject）、实时终端命令输出日志与调用链追踪。
- **现代化排版**：集成 `pulldown-cmark` 增量解析与 `tree-sitter` 多语言语法高亮（Rust、TypeScript、Python、JSON 等）。

---

## 🏗️ 架构概览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DeepSeek Harness Desktop (Rust + GPUI)               │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                            dsh-ui                                 │  │
│  │   • 三栏工作区 (会话侧边栏 / 对话主视窗 / 协同抽屉)              │  │
│  │   • DeepSeek-R1 思考过程实时折叠卡片 (120 FPS)                    │  │
│  │   • Deliverables 成果物胶囊 & Tool Call 执行卡片                  │  │
│  └─────────────────────────────────┬─────────────────────────────────┘  │
│                                    │                                    │
│  ┌─────────────────────────────────▼─────────────────────────────────┐  │
│  │                           dsh-core                                │  │
│  │   • 原生 LlmClient (Reqwest SSE + R1 双流转换器)                  │  │
│  │   • NativeAgentLoop (多轮自主规划与执行循环)                      │  │
│  │   • 7 项内置核心工具 (read, write, edit, patch, grep, list, exec) │  │
│  │   • PluginManager (SKILL.md, plugin.json, MCP Stdio 调度)         │  │
│  │   • DiffApplier (原子化 Unified Diff 解析与 Hunk 补丁应用引擎)    │  │
│  │   • TokenMeter (TTFT、缓存命中率、Token/s 实时统计)               │  │
│  └─────────────────────────────────┬─────────────────────────────────┘  │
│                                    │                                    │
│         ┌──────────────────────────┴──────────────────────────┐         │
│         ▼                                                     ▼         │
│  ┌──────────────┐                                      ┌──────────────┐ │
│  │ dsh-markdown │                                      │ dsh-protocol │ │
│  │(Tree-sitter) │                                      │  (JSON-RPC)  │ │
│  └──────────────┘                                      └──────────────┘ │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
                        HTTPS / SSE 原生直连请求
                                     │
                                     ▼
                      ┌─────────────────────────────┐
                      │    DeepSeek 官方 API 端点   │
                      │ (DeepSeek-V3 / DeepSeek-R1) │
                      └─────────────────────────────┘
```

---

## 📦 工作区子包结构

| 子模块 | 核心职责 |
|---|---|
| [`crates/dsh-core`](crates/dsh-core) | 纯 Rust 原生 LLM 引擎、SSE 解析器、R1 双流转换、Agent 循环、7 大内置工具、插件管理、Diff 引擎与本地持久化。 |
| [`crates/dsh-ui`](crates/dsh-ui) | 高性能 GPUI 桌面界面、三栏布局、R1 思考卡片、模态弹窗与设置中心。 |
| [`crates/dsh-markdown`](crates/dsh-markdown) | Markdown 增量解析（`pulldown-cmark`）与代码语法高亮引擎（`tree-sitter`）。 |
| [`crates/dsh-protocol`](crates/dsh-protocol) | 强类型通信协议、事件信封模型与 JSON-RPC 消息定义。 |
| [`crates/dsh-common`](crates/dsh-common) | 跨平台路径管理、全局常量与基础工具函数。 |
| [`crates/dsh-daemon`](crates/dsh-daemon) | 可选的历史子进程适配模块（仅在 `--remote` 模式下启用）。 |

---

## 🛠️ 快速开始

### 环境要求
- [Rust](https://www.rust-lang.org/)（2021 edition，推荐 1.80+）
- 拥有正常 GPU 加速驱动的 Windows / macOS / Linux 系统

### 1. 源码获取与编译
```bash
# 克隆仓库
git clone https://github.com/oliver-xie666/deepseek-harness-desktop-.git
cd deepseek-harness-desktop

# 运行全量单元测试套件（75 项全绿）
cargo test --workspace

# 编译 Release 优化版本
cargo build -p dsh-ui --release
```

### 2. 启动桌面客户端
```bash
# 默认以 Standalone 纯单机直连模式启动
cargo run -p dsh-ui

# (可选) 以远程 Daemon 兼容模式启动
cargo run -p dsh-ui -- --remote --port 3080

# (可选) 以 Mock 模式启动（用于 UI 界面调试与预览）
cargo run -p dsh-ui -- --mock-daemon
```

---

## ⚙️ 配置文件说明

程序配置文件位于 `~/.dsh/config.json`：

```json
{
  "api_key": "sk-your-deepseek-api-key",
  "base_url": "https://api.deepseek.com",
  "model": "deepseek-reasoner",
  "temperature": 0.6,
  "max_tokens": 8192,
  "system_prompt": null,
  "standalone": true
}
```

- **推荐模型配置**：
  - `deepseek-reasoner`：DeepSeek-R1 深度推理模型（输出实时思维链与最终解答）。
  - `deepseek-chat`：DeepSeek-V3 高性能通用与代码生成模型。
  - 支持任意兼容 OpenAI 格式的本地/私有化端点（Ollama、vLLM、OneAPI 等）。

---

## 🧪 验证与测试

项目具备完整的端到端与单元测试体系，覆盖协议序列化、SSE 流式解析、工具分发、Diff 应用与 UI 状态逻辑：

```bash
# 运行全量工作区测试
cargo test --workspace

# 检查代码格式规范
cargo fmt --check
```

---

## 🪟 Windows Standalone 发行包打包

如需制作 Windows 绿色便携 ZIP 发行包：

```powershell
# 编译 Release 二进制文件
cargo build -p dsh-ui --release

# 打包为分发 ZIP 压缩包
$DistDir = "target/dist"
New-Item -ItemType Directory -Force -Path $DistDir
Compress-Archive -Path "target/release/dsh-desktop.exe", "README.md", "README.zh.md", "LICENSE" -DestinationPath "$DistDir/DeepSeek-Harness-Desktop-Windows-x64.zip" -Force
```

---

## 📄 开源许可证

本项目遵循 [MIT License](LICENSE) 开源协议。
