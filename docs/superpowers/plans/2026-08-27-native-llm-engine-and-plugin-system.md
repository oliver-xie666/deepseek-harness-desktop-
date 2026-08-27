# 纯 Rust 原生大模型直连引擎与社区 dsh-plugin 插件系统实施计划

**文档版本：** v1.0.0  
**制定时间：** 2026-08-27  
**基线版本：** `main`（最新提交 `67fc5de`）

---

## 1. 目标与设计哲学 (Goals & Philosophy)

### 核心目标
1. **0 外部依赖，单二进制 Standalone**：彻底移除对外部 Node.js 守护进程与本地 3080 端口的默认依赖。单一 `dsh-desktop.exe` 启动即用。
2. **纯 Rust 原生直连大模型 (Native LLM Engine)**：
   - 内置基于 `reqwest` 的流式 HTTP/SSE 客户端，直连 DeepSeek 官方 API（`api.deepseek.com`）、OpenAI 兼容端点或本地 Ollama。
   - 专属支持 **DeepSeek-R1 思维链流式解析**（`reasoning_content`）与 **DeepSeek-V3 代码正文流式输出**（`content`）。
   - 原生内置多轮 Agent 循环调度器（Agent Loop），直接驱动 GPUI 120 FPS 局部重绘。
3. **全面拥抱社区 `dsh-plugin` 插件生态 (Community Plugin Ecosystem)**：
   - 自动扫描并加载社区插件目录（如 `~/.dsh/plugins`、工作区 `.dsh/plugins`）。
   - 原生支持 **技能插件 (Skills)**：自动解析 `SKILL.md` / 提示词规则，注入 System Prompt 并动态注册斜杠指令（`/command`）。
   - 原生支持 **可执行工具插件 (Tools)**：通过 Rust 子进程标准输入输出（Stdio JSON）安全调度 Node/Python/CLI 脚本工具。
   - 原生支持 **MCP 标准插件 (Model Context Protocol)**：无缝挂载社区标准 MCP Servers。
4. **内置原生核心工具集 (Native Core Tools)**：
   - 纯 Rust 实现 `read_file`、`write_file`、`edit_file`、`apply_patch`、`grep_search`、`list_dir`、`exec_command`。
   - 与现有的权限控制（`Full access` / `Workspace write` / `Read-only`）和成果物胶囊（`Deliverables`）无缝联动。

---

## 2. 系统架构分解 (Architecture Breakdown)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                        GPUI 原生 UI 层 (crates/dsh-ui)                                   │
│   • 输入框 (Prompt / @file: / /command) ──► ChatView 会话流与 120 FPS 流式打字           │
│   • DeepSeek-R1 思考折叠卡片 ──► Subagent / Skill 专属徽标 ──► Deliverables 成果物胶囊    │
└────────────────────────────────────────────▲────────────────────────────────────────────┘
                                             │ (HarnessServerEvent / AppState Context)
┌────────────────────────────────────────────▼────────────────────────────────────────────┐
│                    核心状态与调度中枢 (crates/dsh-core)                                   │
│  ┌──────────────────────────────┐  ┌──────────────────────────────────────────────────┐  │
│  │ LlmEngine (原生大模型通信)    │  │ NativeAgentLoop (多轮决策中枢)                   │  │
│  │ • SSE 流式请求解析           │  │ • 组装 System Prompt + 历史上下文 + Tools 声明   │  │
│  │ • reasoning_content 思考流   │  │ • 驱动 LLM 生成 ──► 捕获 ToolCall ──► 调度执行   │  │
│  │ • content 回答正文流         │  │ • 结果回填 ──► 发起下一轮决策直到完成            │  │
│  └──────────────▲───────────────┘  └────────────────────────┬─────────────────────────┘  │
│                 │                                           │                             │
│  ┌──────────────┴───────────────┐  ┌────────────────────────▼─────────────────────────┐  │
│  │ PluginManager (社区插件系统) │  │ NativeToolRunner (内置核心工具库)                │  │
│  │ • 插件扫描 (~/.dsh/plugins)  │  │ • 文件读写 / Diff / 全文检索 / 终端执行          │  │
│  │ • SKILL.md / 规则注入        │  │ • 自动生成 Deliverables 产出物                   │  │
│  │ • Stdio / MCP 子进程工具执行 │  │ • 严格遵循权限模式 (Full/Write/Read)             │  │
│  └──────────────────────────────┘  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 详细实施步骤 (Implementation Roadmap)

### Phase 1: 纯 Rust 原生 LLM 通信引擎 (`dsh-core::llm`)
1. **依赖升级**：在 `Cargo.toml` 引入 `reqwest`（`features = ["json", "stream", "rustls-tls"]`）与 `eventsource-stream` / `tokio_util`。
2. **协议与请求定义**：
   - `crates/dsh-core/src/llm/types.rs`：定义 `ChatCompletionRequest`、`ChatMessage`、`ToolDefinition`、`ChatCompletionChunk`。
   - 字段级别支持 `reasoning_content`（DeepSeek-R1）与 `tool_calls`。
3. **SSE 流式客户端**：
   - `crates/dsh-core/src/llm/client.rs`：实现 `LlmClient`，负责异步 HTTP POST、Bearer Token 注入、SSE 行解析与网络异常重试。

### Phase 2: 原生内置核心工具与 Agent 多轮循环 (`dsh-core::agent`)
1. **原生工具集 (`dsh-core::tools`)**：
   - `fs_tools.rs`：`read_file`、`write_file`、`edit_file`、`list_dir`。
   - `search_tools.rs`：`grep_search`、`find_files`。
   - `exec_tools.rs`：`exec_command`（支持后台任务与终止控制）。
   - `diff_tools.rs`：结合现有的 `DiffApplier` 原生生成/应用 Diff。
2. **多轮决策循环 (`NativeAgentLoop`)**：
   - 当用户发送消息时，组装当前工作区文件树、环境信息、已启用工具列表。
   - 流式接收 LLM 输出，发出 `TokenChunk`。
   - 遇到 `ToolCall` 时发出 `ToolCallStart`，调用本地工具/插件，发出 `ToolCallEnd` 并回传 Output，自动递归触发下一步决策直到 `Completed`。

### Phase 3: 社区 `dsh-plugin` 插件系统 (`dsh-core::plugin`)
1. **插件目录规范与发现**：
   - 自动扫描多级插件路径：
     - 全局插件：`~/.dsh/plugins/*` 或 `%APPDATA%\deepseek\plugins\*`
     - 工作区插件：`./.dsh/plugins/*` 或 `./.codex/skills/*`
2. **插件元数据与技能解析**：
   - 解析 `plugin.json` / `package.json` / `SKILL.md`。
   - 提取 `name`、`description`、`skills`、`tools`、`commands`。
3. **插件工具子进程执行器**：
   - 对非 Rust 原生的可执行插件（Node.js / Python / 二进制脚本），通过 `tokio::process::Command` 建立 Stdio 管道安全调度，输入 JSON 参数并捕获执行结果。
4. **MCP (Model Context Protocol) 插件直连**：
   - 与 `crates/dsh-core/src/mcp.rs` 联动，自动探测 MCP 服务并挂载 tools。

### Phase 4: UI 联动与全链路验证 (`dsh-ui`)
1. **R1 思考链 UI 渲染**：
   - 在 `chat_view.rs` 中为 Assistant 消息增加专属的“思维链 (Thinking)”流式折叠面板。
2. **设置界面 Key/Endpoint 即时输入**：
   - 在 `settings_modal.rs` 的“模型”页面增加 API Key 与 Base URL 的文本输入与保存。
3. **默认启动模式切换**：
   - 默认直接启动内置 `NativeAgentLoop`，不再检测或强制依赖 3080 端口；保留 `--remote-daemon` 可选参数以兼容远程场景。
4. **全量测试与打包**：
   - 编写完整的单测与 Mock SSE 测试。
   - 打包生成最新的 Windows Standalone Release ZIP。

---

## 4. 验证与验收标准 (Acceptance Criteria)

1. **Standalone 独立运行**：机器在未安装 Node.js、未启动 3080 的纯净环境下，双击 `dsh-desktop.exe` 输入 DeepSeek API Key 即可直接发起对话。
2. **流式打字与思考链**：
   - 选择 `deepseek-reasoner` (R1) 时，实时流式呈现思考过程与最终答案。
   - 选择 `deepseek-chat` (V3) 时，极速流式打字并支持工具调用。
3. **社区插件无缝加载**：将任意社区 `dsh-plugin`（如包含 `SKILL.md` 或 CLI Tool）放入插件目录，启动即可在快捷菜单看到对应技能，并在对话中被大模型正确调度。
4. **测试与打包 100% 通过**：`cargo test --workspace` 保持全绿，Windows ZIP 正常打包生成。
