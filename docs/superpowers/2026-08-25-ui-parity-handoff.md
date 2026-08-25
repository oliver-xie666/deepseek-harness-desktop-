# UI 持续优化交接

**更新时间：** 2026-08-25

**基线：** `main` / `origin/main` 的 `7c6e7fa`（`merge: diff review feedback`）
**工作树：** 交接开始时干净；`.reasonix/` 与 `.superpowers/` 均为本机生成的忽略目录。

本文件是后续会话的工作入口。它记录已经真实合入的功能、仍需完成的差距，以及继续推进时应遵循的分支和验证方式。

## 已完成

### 状态、会话与工作区

- `AppState` 成为会话事实来源；会话可新建、选择、改名、复制、删除，并使用 ID 而非标题定位。
- 会话含创建/更新时间，服务端 token、工具、diff、Agent 状态和终端日志事件会持久化。
- 删除会话同步删除持久化文件；删除当前会话会选择可用的后继会话。
- 侧栏从 `AppState::session_snapshot` 同步会话，搜索和排序不再依赖硬编码示例数据。
- 工作区路径可以切换，窗口标题、聊天视图和侧栏文件树会同步；Explorer 文件项可以交给系统默认程序打开。

### 对话与输入

- 顶部“对话 / 轨迹”可切换，轨迹展示真实工具调用详情；详情抽屉可复制参数和输出。
- 会话日志按钮可展开最近的终端日志。
- 输入框支持文本选择、提交和换行行为；模式选择、权限选择、预设和模型选择会写入配置。
- `+` 按钮提供 `/help`、`/model`、`/clear` 命令菜单。
- 对话区从已持久化消息渲染用户、助手和系统消息，而非仅显示静态提示文本。

### 设置、模型和窗口

- 设置中已移除不属于该插件的“侧边卡片”内容，包含常规、模型、插件和 Agent 预设页。
- 聊天与设置共用模型选项列表，配置中已有的自定义模型不会被菜单丢弃。
- CLI 的 mock daemon 改为显式 `--mock`，默认不会启动模拟服务。
- 采用原生标题栏、拖拽区域和 macOS 风格控制按钮；Windows 已按用户要求取消窗口圆角。

### Diff 审查

- 收到 `FileDiffReady` 后，聊天区会显示待审查文件、预览和“应用 / 拒绝”操作。
- “应用”使用 unified diff 修改现有工作区文件；上下文不一致时拒绝写入并显示错误。
- “拒绝”会更新并持久化 diff 状态。
- 已覆盖 unified diff 正常应用与上下文不一致的核心测试。

### 工程交付

- Windows 打包脚本可生成 `target/dist/DeepSeek-Harness-Desktop-Windows-x64.zip`。
- 最近一次代码功能分支均已合入 `main` 并推送至 `origin/main`；最后一次打包产物生成于 2026-08-25 09:43（本地时间）。

## 未完成与已知差距

按优先级继续处理；这些项目不得在交接时被表述为“已经与官方一致”。

### P0：视觉和基本可用性

1. 设置页与详情抽屉的主体使用 `overflow_hidden()`，长内容会裁切，尚未接入 GPUI 正确的滚动 API。
2. 尚未建立与官方 Harness 的逐项视觉/交互回归。继续前应启动官方 UI（此前为本地 3080），逐屏比对并记录截图，避免凭记忆调整样式。
3. “添加工作区...”当前不是目录选择器，实际会回退到当前目录；工作区列表也只提供当前目录和上级目录。
4. 侧栏文件树限制扫描深度为两层，且没有展开/折叠状态；它目前是“可打开的预览”，不是完整 Explorer。

### P1：功能完整性

1. 模型目录是静态默认值（`gpt-5.6-luna`、`deepseek-chat`、`deepseek-reasoner`），尚未按当前 provider 或官方 Harness 的可用模型动态获取；默认模型和 DeepSeek provider 的组合需要产品层面确认。
2. Diff 应用仅覆盖现有文本文件的 unified diff。新建文件、删除文件、二进制变更、无行尾换行和基于文件 hash 的冲突保护尚未实现。
3. Diff 成功后没有显式清除旧的失败提示；应在下一次成功的应用/拒绝操作中清空 `diff_notice`。
4. Session log 仅展示最近六行，未提供完整可滚动日志视图或导出。
5. Tool 卡片与轨迹视图还应按协议真实 `ToolStatus` 展示运行中、成功和失败状态，而非统一使用成功样式。
6. 长对话仍按普通元素列表渲染，尚未实现设计文档中承诺的虚拟列表；大历史会话需要性能验证。

### P2：一致性和交付质量

1. 部分侧栏/标题栏仍使用文本符号或 emoji，而非统一图标资源；需以官方 UI 参考为准统一。
2. Agent 预设的复制按钮需要验证事件传播，避免点击复制同时触发选中卡片。
3. 目前没有 GPUI UI 自动化测试或官方 UI 的视觉快照测试。核心 Rust 单测存在，但用户关键交互仍主要依赖手工验证。
4. `cargo check` 会提示第三方 `proc-macro-error2 v2.0.1` 的未来兼容性警告；它不是当前改动导致的失败，但应在依赖升级时处理。

## 下一轮建议顺序

1. 先恢复视觉对照：启动本项目和官方 Harness，完成设置页、对话/轨迹页、输入栏、侧栏的截图和点击路径基线。
2. 修复 GPUI 滚动容器，再让设置页和详情抽屉支持完整内容滚动；完成一次桌面人工验证。
3. 实现真实的工作区目录选择和可展开 Explorer，确保选择后的 `AppState.workspace_path`、标题和树同时刷新。
4. 以官方/provider 实际数据重做模型选择，随后补齐 tool 状态、完整 session log 和 diff 提示清理。
5. 扩展 diff 引擎前先补测试，明确新建/删除文件和冲突策略。

## 关键实现入口

| 领域 | 入口 |
| --- | --- |
| 会话、持久化、服务端事件、diff 操作 | `crates/dsh-core/src/lib.rs` |
| unified diff 解析和原子写入 | `crates/dsh-core/src/diff_applier.rs` |
| 主对话、轨迹、输入、diff 卡片、会话日志 | `crates/dsh-ui/src/chat_view.rs` |
| 设置与模型配置 | `crates/dsh-ui/src/settings_modal.rs`、`crates/dsh-ui/src/model_catalog.rs` |
| 侧栏、工作区树和会话菜单 | `crates/dsh-ui/src/sidebar.rs`、`crates/dsh-ui/src/dropdown.rs` |
| 标题和工作区同步 | `crates/dsh-ui/src/title_bar.rs`、`crates/dsh-ui/src/workspace.rs` |
| Windows 打包 | `scripts/package_windows.ps1` |

## Git 与验证约定

- 从已同步的 `main` 创建一个功能分支，沿用仓库既有命名：`feat/<scope>`、`fix/<scope>` 或 `docs/<scope>`；不要创建 `codex/` 前缀分支。
- 一个分支只提交一个可验证的功能。提交采用现有格式，例如 `feat(ui): ...`、`fix(core): ...`、`docs: ...`。
- 功能完成后依次执行：格式化、相关测试、`cargo check -p dsh-ui`（UI 改动）、`cargo test --workspace`（跨 crate 改动），再检查 `git diff main...HEAD --check`。
- 合并回 `main` 后推送 `origin/main`。涉及 UI/运行时变更时重新运行 `scripts/package_windows.ps1`，确认 ZIP 存在且非空，再交给人工验证。
- 当前远端：`https://github.com/oliver-xie666/deepseek-harness-desktop-.git`；主分支是 `main`，不是 `master`。
