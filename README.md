# ⚡ DeepSeek Harness Desktop (Rust + GPUI)

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust%202021-orange.svg" alt="Rust 2021" />
  <img src="https://img.shields.io/badge/GUI-GPUI%20(Zed%20Engine)-blue.svg" alt="GPUI" />
  <img src="https://img.shields.io/badge/Engine-100%25%20Pure%20Rust%20Native-brightgreen.svg" alt="Pure Rust" />
  <img src="https://img.shields.io/badge/Models-DeepSeek--V3%20%7C%20DeepSeek--R1-purple.svg" alt="Models" />
  <img src="https://img.shields.io/badge/Framerate-120%20FPS-brightgreen.svg" alt="120 FPS" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License" />
</p>

<p align="center">
  <b>English</b> | <a href="README.zh.md">简体中文</a>
</p>

---

**DeepSeek Harness Desktop** is a high-performance native desktop workspace engineered specifically for **DeepSeek-V3** and **DeepSeek-R1**. 

Built entirely in **Rust** with **GPUI** (Zed's GPU-accelerated Direct3D 11/12, Metal, and Vulkan rendering framework), it combines a **100% Pure Rust Native LLM Engine**, **DeepSeek-R1 real-time reasoning streaming**, a **native multi-turn Agent loop with 7 built-in tools**, and an extensible **`dsh-plugin` & `SKILL.md` plugin system**.

By default, the application runs in **Standalone Direct Mode**, requiring **zero external Node.js dependencies and no background ports**, delivering **120 FPS input and scrolling responsiveness** with under **50MB baseline memory usage**.

---

## ✨ Key Features

### 🚀 1. 100% Pure Rust Native LLM Engine
- **Direct LLM Connection**: Direct HTTPS/SSE streaming directly to DeepSeek API (or compatible endpoints) without middleman proxies or Node.js daemons.
- **DeepSeek-R1 Dual-Stream SSE Parser**: Concurrently processes `reasoning_content` (thinking chain) and `content` (final response) with strict frame-by-frame SSE event parsing (`llm/sse.rs`, `llm/translate.rs`).
- **Real-Time Token & Latency Metering**: Tracks Time-to-First-Token (TTFT), real-time generation speed (tokens/sec), prompt tokens, output tokens, reasoning tokens, and disjoint cache hit metrics (`llm/token_meter.rs`).
- **Passback Rule Serialization**: Fully conforms to the official assistant message passback format, correctly retaining tool calls across multi-turn reasoning rounds (`llm/serialize.rs`).

### 🧠 2. DeepSeek-R1 Real-Time Reasoning UI
- **Live Thinking Cards**: Renders expandable `🧠 思考过程 (Reasoning Process)` collapsible cards in `chat_view`.
- **Fluid Animation & Token Streaming**: Live updates thinking steps with zero UI stutter at 120 FPS.
- **Stateful Folding**: Remembers open/closed states across conversational turns with toggle controls.

### 🤖 3. Native Agent Loop & Built-in Tool Suite
- **Multi-Turn Autonomous Loop (`NativeAgentLoop`)**: Formulates decisions, executes tool calls, observes outputs, and continues problem-solving until tasks reach completion.
- **7 Native Core Tools**:
  - `read_file`: Reads local workspace files with line range limits.
  - `write_file`: Atomically writes or overwrites file contents.
  - `edit_file`: Performs exact find-and-replace text modifications.
  - `apply_patch`: Applies unified diff patches atomically with context verification.
  - `grep_search`: Fast regex and substring searches across directories.
  - `list_dir`: Recursively lists directories and file structures.
  - `exec_command`: Executes terminal commands in isolated subprocesses with timeout and output capture.
- **Deliverables & Diff Reviewer**: Integrates seamlessly with the **Deliverables Capsule** and line-level **DiffApplier** for atomic file inspection and one-click Accept / Reject actions.

### 🔌 4. Extensible Community Plugin System (`dsh-plugin`)
- **Plugin Discovery**: Automatically scans `~/.dsh/plugins` and `.dsh/plugins` within active workspaces.
- **Skill Definitions**: Parses `SKILL.md` (YAML frontmatter + Markdown headers) and `plugin.json` configurations.
- **Dynamic MCP & Stdio Execution**: Interacts with community plugins and Model Context Protocol (MCP) servers via standard stdio pipes.

### 🎨 5. Zed-Level 120 FPS GPUI Workspace
- **Three-Pane Integrated Workspace**:
  - **Left Sidebar**: Session history, project workspace tree, and installed plugin/MCP statuses.
  - **Center Canvas**: Fluid chat stream, collapsible tool call cards, and multiline command input.
  - **Right Collaboration Drawer**: Line-by-line diff inspector, terminal execution logs, and trace details.
- **Syntax Highlighting**: Incremental streaming parsing powered by `pulldown-cmark` and `tree-sitter` (Rust, TypeScript, Python, JSON, etc.).

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DeepSeek Harness Desktop (Rust + GPUI)               │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                            dsh-ui                                 │  │
│  │   • Three-Pane Workspace (Sidebar / Chat Canvas / Diff Drawer)     │  │
│  │   • DeepSeek-R1 Collapsible Reasoning Cards (120 FPS)             │  │
│  │   • Deliverables Capsule & Tool Execution Cards                   │  │
│  └─────────────────────────────────┬─────────────────────────────────┘  │
│                                    │                                    │
│  ┌─────────────────────────────────▼─────────────────────────────────┐  │
│  │                           dsh-core                                │  │
│  │   • Native LlmClient (Reqwest SSE + R1 Dual Stream Translator)   │  │
│  │   • NativeAgentLoop (Multi-turn Autonomous Planning)              │  │
│  │   • 7 Built-in Tools (read, write, edit, patch, grep, list, exec) │  │
│  │   • PluginManager (SKILL.md, plugin.json, MCP Stdio Runner)       │  │
│  │   • DiffApplier (Atomic Unified Diff Engine & Hunk Matching)      │  │
│  │   • TokenMeter (TTFT, Cache Hits, Tokens/sec Accounting)          │  │
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
                    HTTPS / SSE Direct Connection
                                     │
                                     ▼
                      ┌─────────────────────────────┐
                      │    DeepSeek API Endpoint    │
                      │ (DeepSeek-V3 / DeepSeek-R1) │
                      └─────────────────────────────┘
```

---

## 📦 Workspace Crates

| Crate | Responsibility |
|---|---|
| [`crates/dsh-core`](crates/dsh-core) | Pure Rust LLM client, SSE parser, R1 stream translator, agent loop, 7 core tools, plugin manager, diff applier, and local persistence. |
| [`crates/dsh-ui`](crates/dsh-ui) | High-performance GPUI desktop application, 3-pane layout, reasoning cards, modals, and settings. |
| [`crates/dsh-markdown`](crates/dsh-markdown) | Incremental Markdown parsing (`pulldown-cmark`) and AST syntax highlighter (`tree-sitter`). |
| [`crates/dsh-protocol`](crates/dsh-protocol) | Strongly-typed message protocols, event envelopes, and JSON-RPC definitions. |
| [`crates/dsh-common`](crates/dsh-common) | Cross-platform directories, constants, and shared utilities. |
| [`crates/dsh-daemon`](crates/dsh-daemon) | Optional legacy subprocess daemon manager (used when `--remote` is enabled). |

---

## 🛠️ Quick Start

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2021 edition, 1.80+ recommended)
- Windows / macOS / Linux with GPU acceleration drivers installed

### 1. Clone & Build
```bash
# Clone the repository
git clone https://github.com/oliver-xie666/deepseek-harness-desktop-.git
cd deepseek-harness-desktop

# Run the full unit test suite (75 tests)
cargo test --workspace

# Build optimized release binary
cargo build -p dsh-ui --release
```

### 2. Run Desktop Application
```bash
# Launch in default Standalone Direct Mode
cargo run -p dsh-ui

# (Optional) Launch with Remote Daemon fallback
cargo run -p dsh-ui -- --remote --port 3080

# (Optional) Launch with Mock Daemon (for UI development/testing)
cargo run -p dsh-ui -- --mock-daemon
```

---

## ⚙️ Interactive Settings & 1:1 UI Parity

The desktop settings interface achieves full 1:1 visual and interactive parity with the official Web UI (`localhost:3080`):

- **General (常规设置)**:
  - Default Agent preset switcher (`standard`, `code`, `minimal`, `cordis`).
  - Permission & Safety mode selection (`full-access`, `ask-every-time`, `read-only`).
  - Language toggle (`zh-CN` / `en-US`) and theme customization (`light` / `dark` / `system`).
  - Enter key behavior configuration (`Enter to send / Shift+Enter for newline` vs `Ctrl+Enter to send`).
  - One-click button to open the local config directory (`~/.dsh/`).
- **Models & Providers (模型与服务商)**:
  - Multi-provider support: DeepSeek official direct stream, OpenAI, Anthropic, MiniMax, Moonshot/Kimi, Qwen, Ollama, VLLM, OpenRouter, and Custom OpenAI-compatible endpoints.
  - Interactive inputs for API Key, Base URL, Model Identifier, Temperature (0.0~2.0), and Max Tokens limit.
  - Quick model selection chips (`deepseek-reasoner`, `deepseek-chat`, `gpt-4o`, `claude-3-5-sonnet-20241022`, `qwen-plus`, `moonshot-v1-32k`, `llama3.2`).
  - Real-time save & apply with live floating status toast feedback.
- **Plugins & Extensions (插件与扩展)**:
  - **Config Tab**: Expandable configuration cards for Terminal/Shell sandbox, Agent Loop reasoning engine, Web Search, and ModLens multimodal ingestion.
  - **Inventory Tab**: 10+ installed plugin cards with search filtering and reactive green/grey toggle switches that directly update `disabled_plugins` in `config.json`.
- **Agent Presets & Custom Roles (智能体预设与定制)**:
  - Builtin presets with default status badges and instant "Set as Default" actions.
  - Interactive **Preset Copy Dialog** modal to duplicate any preset into a custom role (`custom-preset-id`, display name, description).
  - Custom preset management with instant deletion and configuration persistence.
- **Sidebar Cards (侧边栏卡片偏好)**:
  - Reactive toggle switches for default sidebar expansion, automatic jobs panel popup, workspace file tree display, and terminal logs trace display.

## ⚙️ Configuration

The application stores user settings at `~/.dsh/config.json`:

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

- **Supported Models**:
  - `deepseek-reasoner`: DeepSeek-R1 reasoning model (outputs real-time thinking process + solution).
  - `deepseek-chat`: DeepSeek-V3 general chat and coding model.
  - Compatible OpenAI-format endpoints (Ollama, vLLM, OneAPI, etc.).

---

## 🧪 Testing & Verification

The project includes an end-to-end test suite covering serialization, SSE parsing, translation, tools, diff engines, and UI state:

```bash
# Run all workspace tests
cargo test --workspace

# Check formatting
cargo fmt --check
```

---

## 🪟 Windows Standalone Packaging

To produce a portable standalone release package for Windows:

```powershell
# Build release binary
cargo build -p dsh-ui --release

# Pack into distribution ZIP archive
$DistDir = "target/dist"
New-Item -ItemType Directory -Force -Path $DistDir
Compress-Archive -Path "target/release/dsh-desktop.exe", "README.md", "README.zh.md", "LICENSE" -DestinationPath "$DistDir/DeepSeek-Harness-Desktop-Windows-x64.zip" -Force
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
