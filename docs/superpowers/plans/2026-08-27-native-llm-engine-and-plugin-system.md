# 纯 Rust 原生大模型引擎与社区 dsh-plugin 系统实施计划 (对齐官方 packages 源码)

**文档版本：** v1.1.0 (基于官方 `deepseek-harness/packages` 源码 1:1 原生重构)  
**制定时间：** 2026-08-27  
**基线版本：** `main`（最新提交 `1cc58f9`）  
**参考源码基准：** 本地官方库 `D:\typeScript\deepseek-harness\packages`

---

## 1. 架构目标与重构原则

1. **1:1 纯 Rust 原生重写官方 LLM 与 Agent 引擎**：
   - 参考 `packages/llm/llm-deepseek` 与 `packages/core`，在 `crates/dsh-core` 中构建 100% 纯 Rust 原生实现。
   - 彻底摆脱对外部 Node.js 守护进程和 3080 端口的依赖，单二进制 `dsh-desktop.exe` 直接直连 DeepSeek API。
2. **官方协议与特性 100% 对齐**：
   - **DeepSeek-R1**：原生 SSE 解析 `delta.reasoning_content`（思维链实时流）与 `delta.content`（正文流）。
   - **DeepSeek-V3**：原生支持 Function Calling 工具调用组装与多轮 Agent Loop 循环。
   - **上下文窗口与错误处理**：对齐 `packages/llm` 的 `CONTEXT_WINDOW_EXCEEDED`、速率限制重试与 Token 计量。
3. **社区 `dsh-plugin` 插件生态无缝兼容**：
   - 对齐 `packages/extensions` 与 `packages/skill`，原生扫描并解析 `SKILL.md`、`plugin.json` 与 MCP Servers，支持 Stdio 子进程工具调度。

---

## 2. 官方 packages 模块与 Rust 1:1 对齐映射表

| 官方 TypeScript 包 (`packages/`) | 核心功能与源码参考 | 对应的 Rust 原生重构模块 (`crates/dsh-core/`) |
| :--- | :--- | :--- |
| **`packages/llm/llm-deepseek`** | `adapter.ts`, `sse.ts`, `serialize.ts`, `translate.ts` | **`src/llm/`**<br>• `client.rs` (reqwest SSE 流式传输)<br>• `sse.rs` (流式分块解析器)<br>• `types.rs` (Chat 请求/响应结构体) |
| **`packages/core`** | `agent.ts`, `session.ts`, `context.ts`, `compaction` | **`src/agent/`**<br>• `agent_loop.rs` (多轮决策与 Prompt 组装)<br>• `context.rs` (历史上下文与裁剪) |
| **`packages/fs` & `shell`** | 文件读写、Diff 生成、终端命令执行、权限检查 | **`src/tools/`**<br>• `fs_tools.rs` (`read_file`, `write_file`, `edit_file`)<br>• `search_tools.rs` (`grep_search`, `list_dir`)<br>• `exec_tools.rs` (`exec_command`) |
| **`packages/skill` & `extensions`** | 技能发现、`SKILL.md` 解析、插件工具注入、MCP 桥接 | **`src/plugin/`**<br>• `loader.rs` (扫描 `~/.dsh/plugins` / 工作区)<br>• `runner.rs` (Stdio 子进程工具派发)<br>• `skill_parser.rs` (解析 `SKILL.md` 与 Prompt 注入) |
| **`packages/llm/token-meter`** | Token 估算、上下文消耗统计与统计栏计算 | **`src/llm/token_meter.rs`**<br>• 实时统计 Token 流速 (token/s)、总计数与耗时 |

---

## 3. 分阶段实施落地路线 (Phased Implementation)

### Phase 1: 纯 Rust 原生 LLM 通信引擎 (`crates/dsh-core/src/llm`)
- **对齐源码**：`packages/llm/llm-deepseek/src/adapter.ts` 与 `sse.ts`。
- **具体实现**：
  1. 引入 `reqwest`（`features = ["json", "stream", "rustls-tls"]`）与 `eventsource-stream`。
  2. 实现 `LlmClient`，向 `https://api.deepseek.com/chat/completions` 发起 `stream: true` 请求。
  3. 严格解析 SSE 格式：实时流式分发 `delta.reasoning_content`（驱动思考面板）与 `delta.content`（驱动正文打字），正确捕获 `[DONE]` 与 `delta.tool_calls`。

### Phase 2: 原生 Agent 循环与内置核心工具集 (`crates/dsh-core/src/agent` & `tools`)
- **对齐源码**：`packages/core/src/agent.ts`、`packages/fs`、`packages/shell`。
- **具体实现**：
  1. 纯 Rust 实现核心工具：`read_file`、`write_file`、`edit_file`、`apply_patch`、`grep_search`、`list_dir`、`exec_command`。
  2. 结合现有 `DiffApplier` 与 `extract_produced_files`，自动生成 Deliverables 产出文件胶囊。
  3. 实现 `NativeAgentLoop`：用户输入 ──► 组装 Prompt/Tools ──► 触发 LLM ──► 捕获 ToolCall ──► 调度本地执行 ──► 结果回填 ──► 下一轮决策直到完成。

### Phase 3: 社区 `dsh-plugin` 插件系统 (`crates/dsh-core/src/plugin`)
- **对齐源码**：`packages/skill/src` 与 `packages/extensions/src`。
- **具体实现**：
  1. 自动扫描插件路径（`~/.dsh/plugins`、`%APPDATA%\deepseek\plugins`、工作区 `.dsh/plugins`）。
  2. 解析 `SKILL.md` / `plugin.json`，提取技能描述并注入 System Prompt，自动在 UI 生成 `/command` 快捷指令。
  3. 对非 Rust 的 CLI/Node/Python 外部插件，通过 `tokio::process::Command` 建立 Stdio JSON 管道调度。
  4. 与现有的 `crates/dsh-core/src/mcp.rs` 深度打通，支持标准 MCP Tools 挂载。

### Phase 4: UI 联动、默认脱离 3080 端口与打包验证
1. **默认启动模式**：直接使用纯 Rust 内置 `NativeAgentLoop`，无需启动本地 3080 服务（保留 `--remote` 作为远程调试选项）。
2. **UI 增强**：在 `chat_view.rs` 呈现 DeepSeek-R1 思考折叠面板；在 `settings_modal.rs` 提供直观的 API Key 输入与保存。
3. **全量测试与打包**：
   - 保持全 Workspace 52 项单元测试 100% 通过，并追加原生 LLM 与插件加载单测。
   - 重新执行 Windows Release 打包生成独立免安装的 `DeepSeek-Harness-Desktop-Windows-x64.zip`。

---

## 4. 验收与交付标准
1. **0 外部依赖单机运行**：在无 Node.js、无 3080 服务的纯净 Windows 环境下，直接启动 `dsh-desktop.exe`，输入 Key 即可直接发起流式对话与代码生成。
2. **DeepSeek-R1/V3 完美支持**：支持思维链流式折叠卡片与代码高亮打字流。
3. **社区插件自由加载**：放入任意社区 `dsh-plugin`，自动识别 Tools 与 Skills。
4. **全绿测试与安装包交付**：`cargo test --workspace` 全过，Windows ZIP 打包成功。
