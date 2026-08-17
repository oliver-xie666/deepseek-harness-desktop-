# DeepSeek Harness 原生桌面端 (Rust + GPUI) 技术设计规范

- **文档版本**：v1.0.0
- **创建日期**：2026-08-17
- **状态**：已批准 (Approved)
- **目标平台**：macOS, Windows, Linux
- **核心技术栈**：Rust, GPUI (Zed GPU 渲染框架), Tokio, Tree-sitter, pulldown-cmark, Node.js (内嵌轻量便携运行时), DeepSeek Harness (`dsh`)

---

## 1. 项目背景与目标

### 1.1 项目背景
`deepseek-ai/deepseek-harness` 是 DeepSeek 官方推出的基于 Cordis 插件架构的本地 Agent 运行时及开发环境。目前社区桌面包装主要采用 Electron（如 `anywhere-labs/deepseek-harness-desktop`），存在内存开销大（300MB~800MB+）、响应延迟高、与原生系统深度整合不足等问题。

### 1.2 核心诉求与目标
1. **极致性能与流畅体验**：借助 GPUI 原生 GPU 加速渲染，实现 120 FPS 的平滑滚动与流式 Token 输出，常驻内存控制在 50MB~100MB 以内。
2. **Zed 级代码与富文本体验**：复用 Zed 经过实战检验的 Markdown AST 解析、增量流式渲染与 Tree-sitter 语法高亮体系。
3. **独立分发与开箱即用**：内嵌便携式 Node.js 运行时与 `dsh` 依赖，用户双击安装即用，无需预装 Node 或配置环境变量。
4. **工作区集成与开发者交互**：采用 Zed/Cursor 风格的三栏布局，集成会话管理、Tool Call 折叠卡片、Diff 审查与实时终端。

---

## 2. 系统总体架构

系统采用 **分层解耦架构（Decoupled Core + GPUI UI + Subprocess Daemon Manager）**：

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Rust 原生进程空间 (dsh-desktop)                          │
│                                                                                 │
│  ┌─────────────────────────────────┐    ┌────────────────────────────────────┐  │
│  │           dsh-ui (GPUI)         │◄───┤         dsh-core (State/IPC)       │  │
│  │  - 主工作区布局 (Sidebar/Chat)   │    │  - AppState / SessionManager       │  │
│  │  - 增量 Markdown 流式渲染        │    │  - WebSocket / JSON-RPC Client     │  │
│  │  - Tree-sitter 高亮代码块       │    │  - Process Daemon Manager          │  │
│  └────────────────┬────────────────┘    └─────────────────┬──────────────────┘  │
│                   │ GPUI Context                          │ Tokio Async Task    │
│                   │ (Model / Subscription)                │                     │
└───────────────────┼───────────────────────────────────────┼─────────────────────┘
                    │                                       │ Localhost WebSocket / RPC
                    │                                       ▼
┌───────────────────┼─────────────────────────────────────────────────────────────┐
│                   │     内置受控子进程空间 (Isolated Managed Subprocess)           │
│                   │                                                             │
│                   │     ┌──────────────────────────────────────────────┐        │
│                   │     │ 便携式 Node.js Runtime + deepseek-harness     │        │
│                   └────►│  - Cordis 插件体系 / MCP 工具集 / Sandboxes    │        │
│                         │  - Agent Loop 运行调度器                       │        │
│                         └──────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Cargo Workspace 与模块划分

```
deepseek-harness-desktop/
├── Cargo.toml               # Workspace 根配置
├── DESIGN.md                # 架构设计规范文档
├── AGENT.md                 # Agent 协作与开发规范
├── crates/
│   ├── dsh-common/          # 通用类型、全局配置、路径解析、统一错误类型
│   ├── dsh-protocol/        # 强类型 JSON-RPC / WebSocket 协议结构与 Serde 序列化
│   ├── dsh-daemon/          # 便携式 Node.js 运行时解压、进程生命周期守护与健康巡检
│   ├── dsh-core/            # 异步事件循环、会话数据持久化、状态管理
│   ├── dsh-markdown/        # 基于 pulldown-cmark + Tree-sitter 的增量高亮与富文本引擎
│   └── dsh-ui/              # GPUI 原生视图、三栏布局、组件库与主题管理
├── resources/
│   ├── runtime/             # 打包内嵌的轻量便携式 Node.js + @deepseek-ai/dsh Bundle
│   ├── grammars/            # Tree-sitter 常用语言语法文件 (.wasm / native)
│   └── icons/               # 跨平台高清应用图标 (.icns, .ico, .png)
└── scripts/
    ├── bundle_runtime.sh    # 预打包 Node.js 运行时与 dsh 生产包
    └── build_package.sh     # 调用 cargo-packager 产出安装包 (.dmg, .msi, .deb)
```

---

## 4. 详细模块设计

### 4.1 `dsh-protocol` 协议层
负责与 `dsh` 服务的 JSON-RPC / WebSocket 双向消息通信：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HarnessClientMessage {
    InitSession { workspace_path: String, mode: String },
    SendPrompt { session_id: String, text: String, attachments: Vec<String> },
    CancelExecution { session_id: String },
    AcceptDiff { diff_id: String },
    RejectDiff { diff_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HarnessServerEvent {
    SessionCreated { session_id: String },
    TokenChunk { session_id: String, text: String },
    ToolCallStart { id: String, tool_name: String, input: serde_json::Value },
    ToolCallEnd { id: String, output: serde_json::Value, status: ToolStatus },
    FileDiffReady { id: String, file_path: String, diff_content: String },
    AgentStateChange { session_id: String, state: AgentState },
    TerminalLog { session_id: String, line: String },
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Success,
    Failed,
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Thinking,
    ExecutingTool,
    WaitingForApproval,
    Completed,
}
```

### 4.2 `dsh-daemon` 后台进程守护
1. **运行时提取**：
   - 首次启动或校验哈希不一致时，从内嵌资源释放便携式 Node.js 运行时到用户数据目录（如 Windows `%APPDATA%/dsh-desktop/runtime` 或 macOS `~/Library/Application Support/dsh-desktop/runtime`）。
2. **进程管理**：
   - 随机探测未占用的 Localhost 端口。
   - 使用 `tokio::process::Command` 启动 `dsh` 服务并绑定该端口。
   - 监听子进程的 `stderr`，收集启动日志与崩溃告警。
3. **健康检查与优雅退出**：
   - 定时发送 Ping 探测心跳。
   - 客户端退出时发送 `SIGTERM` / `TerminateProcess` 确保无孤儿子进程残留。

### 4.3 `dsh-markdown` 富文本与高亮引擎
1. **增量流式解析（Incremental Streaming Parsing）**：
   - 将流式 Markdown 按段落双换行 `\n\n` 划分成已完成 Block 列表和当前正在生成的 Active Block。
   - 已完成 Block 固化为 GPUI 渲染元素并加入缓存；尾部 Block 进行即时 `pulldown-cmark` 解析。
2. **Tree-sitter 语法高亮**：
   - 注册常用语法解析器（Rust, TS, JS, Python, Go, JSON, YAML, SQL, Shell, HTML/CSS）。
   - 将语法 Token 映射至 GPUI `StyledRun`，呈现 Zed 风格语法着色。
   - 代码块外层包裹带有“语言标签”、“一键复制”、“Apply 到工作区”的工具栏。
3. **交互式标签（Interactive Spans）**：
   - 文件路径（如 `src/main.rs:12`）渲染为高亮胶囊标签，点击直接触发右侧 Diff 查看或打开本地编辑器。
   - 支持 GitHub 风格的 Alert 容器（`[!NOTE]`, `[!WARNING]`, `[!TIP]`）。

### 4.4 `dsh-ui` GPUI 界面布局与交互
1. **TitleBar（顶部控制栏）**：
   - 当前工作区路径下拉切换。
   - Agent 模型与模式选择器（Standard / Code / Minimal / Creator）。
   - Daemon 状态指示灯（绿色正常 / 黄色连接中 / 红色错误）。
2. **Sidebar（左侧面板）**：
   - 项目目录树状导航。
   - 历史会话列表（支持搜索、重命名、归档、删除）。
   - 已加载的 MCP 工具与 Cordis 插件状态面板。
3. **ChatView（主对话区）**：
   - 使用 GPUI `UniformList` 虚拟化长列表，保障海量消息下的流畅滚动。
   - `ToolCallCard` 组件：可展开/折叠显示工具执行入参、终端输出及耗时。
   - 底部自适应高度输入框，支持 `@` 引用文件、`/` 呼出快捷指令。
4. **WorkspacePanel（右侧/底部辅助面板）**：
   - **Diff 查看器**：彩色呈现文件变更，支持一键采纳（Accept）与拒绝（Reject）。
   - **实时终端日志**：流式展示 Shell 命令的 Standard Output 与 Error。

---

## 5. 异常防护与容灾机制

1. **守护进程异常自愈**：
   - 若 Node.js 子进程发生非预期崩溃，Rust 守护器以指数退避策略自动拉起新进程并重建 WebSocket 连接，UI 呈现非阻塞重连提示。
2. **文件安全写入校验**：
   - Agent 生成 Diff 改动在写入本地文件前，计算目标文件 Hash。若本地文件在生成期间被外部修改，弹出冲突提醒，防止用户代码丢失。
3. **网络与生成中断重试**：
   - 记录最新消息的 `message_id` 与执行状态，网络闪断恢复后支持一键重试。

---

## 6. 打包与分发流水线

1. **开发阶段（Dev Mode）**：
   - `cargo run` 优先检查本地是否已有运行中的 `dsh`（如 `localhost:3000`），已有则直接连接，极大加速 UI 迭代。
2. **生产构建（Release Mode）**：
   - 执行 `scripts/bundle_runtime.sh` 编译并固化 Node.js 运行时与 `dsh` 生产包。
   - 调用 `cargo-packager` 打包为跨平台原生安装产物：
     - **macOS**：`.dmg` / `.app`
     - **Windows**：`.msi` / `.exe`
     - **Linux**：`.AppImage` / `.deb`
