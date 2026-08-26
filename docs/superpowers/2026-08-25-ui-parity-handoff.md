# UI 持续优化交接

**更新时间：** 2026-08-26

**基线：** `main` / `origin/main` 的最新提交（`b19e4b8` / `feat/jobs-goal-modlens-export`）
**工作树：** 干净；`.reasonix/` 与 `.superpowers/` 均为本机生成的忽略目录。

本文件记录桌面端应用与官方 Harness（本地 3080 端口）逐项视觉与交互对照、已完成的功能项、修复差距以及后续维护规范。

## 已完成

### 状态、会话与工作区

- **会话持久化与状态管理**：`AppState` 为唯一事实来源；会话可新建、选择、重命名、复制、删除，通过稳定 ID 定位；删除当前会话平滑切换后继会话。
- **存储路径隔离与测试套件**：`AppState` 支持 `new_with_storage` 自定义数据目录与 `DSH_DATA_DIR` 环境变量覆盖，单元测试全面接入隔离临时目录，杜绝文件权限污染。
- **会话相对时间徽标**：侧栏会话项基于 `session.updated_at` 实时渲染相对时间标记（`刚刚`、`X分钟前`、`X小时前`、`昨天`、`X天前` 或 `%m-%d`），长标题支持平滑省略。
- **工作区切换与原生目录选择器**：侧栏“添加工作区...”接入 GPUI 原生目录选择器，取消或空选安全回退；切换工作区同步更新窗口标题、会话上下文与文件树。
- **可展开折叠的 Explorer 文件树**：支持多层目录递归扫描、展开/折叠状态管理，并提供一键刷新与 Explorer 面板折叠展开（`▸` / `▾`）；文件项可交给系统默认程序打开。
- **侧栏检索与视图排序**：支持会话名称过滤搜索以及“按最近使用”与“按名称”排序切换。

### 对话、轨迹、消息操作与交互组件

- **对话与轨迹双重视图**：顶部“对话 / 轨迹”自由切换，轨迹视图真实展示工具调用入参、输出与执行耗时；详情抽屉支持参数和输出的一键复制（带 SVG 矢量图标）。
- **消息交互与代码块操作**：Assistant 消息底部提供操作条（复制、点赞、点踩、重试/Fork 按钮），支持消息内容与代码块独立一键复制到剪贴板，带有悬浮效果。
- **输入框底部运行时统计栏（Stats Line）**：完全对齐官方 Harness 统计栏，动态格式化输出 `X 轮 · Y 步 | LLM ... · 工具调用 ... | 首 token 平均 ... · ... token/s | 缓存命中 ...% | 输入 ... · 输出 ...`，0 步骤时平滑回退。
- **Plan 模式徽标与审核卡片（Plan Review）**：
  - 输入框内支持渲染琥珀色 Plan 模式徽标（`Plan ✕`），点击退出 Plan 模式。
  - 会话流中支持呈现 Plan 计划审核卡片（包含琥珀色状态指示、Markdown 计划内容展示与“批准计划”操作）。
- **交互式方案选择卡片（Question Cards）**：
  - 支持服务端下发交互式多选/单选方案卡片（`QuestionPrompt`），渲染选项胶囊按钮并在前端交互选中，点击“确认选择”回传服务端。
- **目标导航条（GoalBar）**：
  - 输入框上方锚定 GoalBar 目标条（Target 靶心矢量图标、阶段标签 `目标 (进行中)` / `目标 (已暂停)` / `目标 (阻塞)` / `目标 (已完成)`）。
  - 支持目标行内快速编辑、保存、取消、暂停/继续切换以及一键清除。
- **后台任务指示器与任务列表（JobListAction & JobListMenu）**：
  - 输入框底部工具栏渲染动态任务状态徽标（运行中绿/停止中琥珀/失败红/完成灰，以及任务计数）。
  - 点击展开任务详情菜单，展示任务类型标签、Monospace 标识、状态、耗时以及运行中任务的终止控制。
- **图片与文件附件输入（ModLens Attachment Intake）**：
  - 输入框底部集成回形针附件选择按钮，支持多文件选择，在输入框上方渲染可移除的附件胶囊条。
  - 发送消息时自动关联附件列表并同步保存与展示。
- **Session 完整记录导出（Session Log Export）**：
  - 会话顶部工具栏与 Session Log 展开抽屉提供“导出 Markdown”与“导出 JSON”操作。
  - 自动生成结构化会话 Markdown / JSON 文件并复制到剪贴板，在日志流中记录导出路径。
- **工具调用状态细化**：按协议真实 `ToolStatus` 细分“运行中（Running）”、“成功（Success）”与“失败（Error）”状态并渲染对应状态徽标与色彩。
- **全量可滚动 Session 日志**：支持展开查看全量终端与执行事件日志，支持日志内容复制与导出。
- **输入区域与命令菜单**：支持文本换行与快捷键提交；支持快速切换权限模式（Full access / Workspace write / Read-only / Ask）、Agent 预设与命令菜单。

### 模型、设置模态框与预设管理

- **提供方层级化模型目录**：模型选项按提供方（DeepSeek 官方、DeepSeek 视觉增强、自定义/bytecat 等）分组呈现，保留自定义模型与扩展模型能力。
- **设置模态框完备性**：常规设置、模型设置、插件配置、Agent 预设与侧边卡片等 5 大导航页全面接入 GPUI `ScrollHandle` 垂直滚动容器，右上角支持打开配置文件目录与关闭操作。
- **外观主题多态选择**：通用设置提供“浅色”、“深色”与“跟随系统”三态卡片，集成独立矢量图标与激活边框高亮。
- **模型提供方管理**：模型设置呈现 DeepSeek 官方与 bytecat 自定义提供方卡片，支持展开内嵌表单配置 API 密钥、选择默认模型及保存/取消。
- **Agent 预设 2x2 网格对齐**：Agent 预设采用 2x2 双列网格卡片布局（标准模式、PTC 模式、极简模式、创造模式），完整展示“内置”、“当前使用”徽标、key 标识及文档/复制按钮；复制事件采用 `stop_propagation()` 彻底隔离点击穿透。
- **插件与 MCP 管理**：插件设置支持内置插件（包含视觉引擎 ModLens 展开配置）手风琴折叠以及本地 MCP 服务状态开关。

### Diff 审查与应用引擎

- **多场景 Unified Diff 引擎**：除常规文件修改外，已支持新建文件（`--- /dev/null`）、删除文件（`+++ /dev/null`）及 `\ No newline at end of file` 无换行末尾格式处理。
- **原子替换与冲突保护**：采用唯一临时文件与上下文严格校验，上下文不匹配时拒绝写入；成功应用或拒绝后自动清除旧的错误提示（`diff_notice`）。
- **单元测试覆盖**：覆盖原子文件写入、现有文件 diff 修改、新文件生成、文件删除、无尾随换行及上下文不匹配拦截等 16 项 core 单元测试。

### 视觉一致性与矢量资产

- **全量矢量化图标**：侧栏搜索、视图选项、添加工作区、会话菜单、详情抽屉扳手/关闭/复制、刷新、点赞、点踩、重试、太阳、月亮、显示器、文档、Target 目标、回形针附件、下载导出、播放、暂停等全面接入 SVG 矢量图标，杜绝 Emoji 或文本符号替代。
- **官方 Harness 视觉回归**：已对照本地 3080 端口官方 Harness 进行全功能视觉与交互审查（对话、轨迹、侧栏、设置模态框、模型选择、Plan 卡片、问题卡片、GoalBar、任务列表、附件输入与日志导出等）。

### 工程交付

- **Windows 独立打包**：`scripts/package_windows.ps1` 可构建并打包包含可执行程序及完整 `assets/` 矢量资源目录的 `DeepSeek-Harness-Desktop-Windows-x64.zip`（~6.22 MB）。
- **代码质量与测试**：全 workspace 单元测试 48 项全部通过（`cargo test --workspace`），`cargo fmt` 格式化通过，`cargo check -p dsh-ui` 零警告零报错。

## 关键实现入口

| 领域 | 入口 |
| --- | --- |
| 会话、持久化、Plan 与问题状态、Goal 目标、任务状态、服务端事件、diff 操作、导出格式化 | `crates/dsh-core/src/lib.rs` |
| unified diff 解析、新建/删除与原子写入 | `crates/dsh-core/src/diff_applier.rs` |
| 主对话、轨迹、Plan 卡片、问题卡片、GoalBar、任务状态列表、附件栏、消息操作、代码块复制、底栏统计、会话日志导出 | `crates/dsh-ui/src/chat_view.rs` |
| 模型目录与提供方分组 | `crates/dsh-ui/src/model_catalog.rs` |
| 设置模态框、主题选择、预设 2x2 网格、插件管理与 ModLens | `crates/dsh-ui/src/settings_modal.rs` |
| 侧栏、工作区树、搜索排序、相对时间与会话菜单 | `crates/dsh-ui/src/sidebar.rs`、`crates/dsh-ui/src/dropdown.rs` |
| 矢量图标体系 | `crates/dsh-ui/src/icons.rs`、`crates/dsh-ui/assets/` |
| 标题和工作区同步 | `crates/dsh-ui/src/title_bar.rs`、`crates/dsh-ui/src/workspace.rs` |
| Windows 打包脚本 | `scripts/package_windows.ps1` |

## Git 与验证约定

- 从已同步的 `main` 创建功能分支，沿用命名：`feat/<scope>`、`fix/<scope>` 或 `docs/<scope>`。
- 功能完成后依次执行：`cargo fmt --check`、`cargo test --workspace`、`cargo check -p dsh-ui`，再检查 `git diff main...HEAD --check`。
- 合并回 `main` 后推送 `origin/main`。UI 或运行时变更时重新运行 `scripts/package_windows.ps1`，确认 ZIP 存在且非空。
- 远端仓库：`https://github.com/oliver-xie666/deepseek-harness-desktop-.git`，主分支为 `main`。
