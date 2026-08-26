# UI 持续优化交接

**更新时间：** 2026-08-26

**基线：** `main` / `origin/main` 的最新提交（`7e1f469` / `feat/deliverables-skill-card-command-menu`）
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

- **生成成果物与产出文件胶囊（Deliverables / ProducedFiles）**：
  - 对齐 `@deepseek-ai/dsh-client-ui-deliverables`，自动识别工具调用（`write_file`、`apply_patch`、`edit_file` 等）生成的产出文件。
  - 在 Assistant 消息底部渲染带文档矢量图标的 `生成文件:` 胶囊列表，支持点击直接在系统默认编辑器/文件管理器中打开。
- **技能调用专用卡片与视觉标识（Skill Cards）**：
  - 对齐 `@deepseek-ai/dsh-client-ui-skill`，对 `skill` / `activate_skill` 工具调用赋予专属紫色高亮徽标（`⚡ 技能: <skill_name>`）与独立执行状态胶囊。
- **全功能斜杠快捷指令菜单（Slash Commands Menu）**：
  - 对齐 `@deepseek-ai/dsh-client-ui-commands`，输入框 `+` 菜单全面扩展为带命令名、功能说明与高亮样式的快捷指令弹窗（支持 `/help`、`/model`、`/plan`、`/export`、`/diff`、`/clear`）。
- **对话与轨迹双重视图与深度检索**：
  - 顶部“对话 / 轨迹”自由切换，轨迹视图真实展示工具调用入参、输出与执行耗时。
  - 轨迹检索输入框深度支持按工具类别（TOOL/LLM/AGENT）、工具名称、执行参数（args）与工具输出（output）全文检索与回合过滤。
  - 提供单回合折叠/展开与全量一键折叠/展开（`▸` / `▾`）及耗时模式切换。
  - 详情抽屉支持参数和输出的一键复制（带 SVG 矢量图标）。
- **消息交互、点赞点踩与复制即时反馈**：
  - Assistant 消息底部操作条提供复制、点赞（Thumbs Up，激活高亮蓝色）、点踩（Thumbs Down，激活高亮红色）、重试/Fork 按钮。
  - 消息内容复制与 Markdown 代码块独立复制按钮提供即时视觉反馈（绿色 Check 矢量图标与“已复制”提示）。
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

### 模型、设置模态框、插件清单与预设管理

- **提供方层级化模型目录**：模型选项按提供方（DeepSeek 官方、DeepSeek 视觉增强、自定义/bytecat 等）分组呈现，保留自定义模型与扩展模型能力。
- **设置模态框完备性**：常规设置、模型设置、插件配置、Agent 预设与侧边卡片等 5 大导航页全面接入 GPUI `ScrollHandle` 垂直滚动容器，右上角支持打开配置文件目录与关闭操作。
- **插件清单与插件配置切换（Plugin Inventory）**：
  - “插件”导航支持“插件配置”与“插件列表 (10)”分段切换。
  - 完整呈现 10 项已安装核心插件清单、官方包名 ID、分类徽标、版本、描述与启用/禁用开关。
- **侧边栏卡片偏好管理（Sidebar Cards Prefs）**：
  - 提供侧栏默认展开、任务与 Subagent 自动提示、工作区文件树卡片、终端执行日志抽屉 4 项独立偏好开关。
- **外观主题多态选择**：通用设置提供“浅色”、“深色”与“跟随系统”三态卡片，集成独立矢量图标与激活边框高亮。
- **模型提供方管理**：模型设置呈现 DeepSeek 官方与 bytecat 自定义提供方卡片，支持展开内嵌表单配置 API 密钥、选择默认模型及保存/取消。
- **Agent 预设 2x2 网格对齐**：Agent 预设采用 2x2 双列网格卡片布局（标准模式、PTC 模式、极简模式、创造模式），完整展示“内置”、“当前使用”徽标、key 标识及文档/复制按钮；复制事件采用 `stop_propagation()` 彻底隔离点击穿透。
- **MCP 服务管理**：支持内置插件（包含视觉引擎 ModLens 展开配置）手风琴折叠以及本地 MCP 服务状态开关。

### Diff 审查与应用引擎

- **多场景 Unified Diff 引擎**：除常规文件修改外，已支持新建文件（`--- /dev/null`）、删除文件（`+++ /dev/null`）及 `\ No newline at end of file` 无换行末尾格式处理。
- **原子替换与冲突保护**：采用唯一临时文件与上下文严格校验，上下文不匹配时拒绝写入；成功应用或拒绝后自动清除旧的错误提示（`diff_notice`）。
- **单元测试覆盖**：覆盖原子文件写入、现有文件 diff 修改、新文件生成、文件删除、无尾随换行、产出文件提取及上下文不匹配拦截等 17 项 core 单元测试。

### 视觉一致性与矢量资产

- **全量矢量化图标**：侧栏搜索、视图选项、添加工作区、会话菜单、详情抽屉扳手/关闭/复制、刷新、点赞、点踩、重试、太阳、月亮、显示器、文档、Target 目标、回形针附件、下载导出、播放、暂停、Check 勾选等全面接入 SVG 矢量图标，杜绝 Emoji 或文本符号替代。
- **官方 Harness 视觉回归**：已对照本地 3080 端口官方 Harness 进行全功能视觉与交互审查（对话、轨迹、侧栏、设置模态框、模型选择、Plan 卡片、问题卡片、GoalBar、任务列表、附件输入、日志导出、插件清单、消息反馈、成果物、技能卡片、斜杠菜单等）。

### 工程交付

- **Windows 独立打包**：`scripts/package_windows.ps1` 可构建并打包包含可执行程序及完整 `assets/` 矢量资源目录的 `DeepSeek-Harness-Desktop-Windows-x64.zip`（~6.27 MB）。
- **代码质量与测试**：全 workspace 单元测试 52 项全部通过（`cargo test --workspace`），`cargo fmt` 格式化通过，`cargo check -p dsh-ui` 零警告零报错。

## 关键实现入口

| 领域 | 入口 |
| --- | --- |
| 会话、持久化、Plan 与问题状态、Goal 目标、任务状态、服务端事件、diff 操作、导出格式化、产出文件识别 | `crates/dsh-core/src/lib.rs` |
| unified diff 解析、新建/删除与原子写入 | `crates/dsh-core/src/diff_applier.rs` |
| 主对话、生成成果物胶囊、技能卡片、斜杠快捷菜单、轨迹深度检索过滤、Plan 卡片、问题卡片、GoalBar、任务状态列表、附件栏、消息点赞/点踩与即时复制反馈、代码块复制反馈、底栏统计、会话日志导出 | `crates/dsh-ui/src/chat_view.rs` |
| 模型目录与提供方分组 | `crates/dsh-ui/src/model_catalog.rs` |
| 设置模态框、主题选择、预设 2x2 网格、插件清单与配置切换、侧边栏卡片偏好、ModLens 与 MCP | `crates/dsh-ui/src/settings_modal.rs` |
| 侧栏、工作区树、搜索排序、相对时间与会话菜单 | `crates/dsh-ui/src/sidebar.rs`、`crates/dsh-ui/src/dropdown.rs` |
| 矢量图标体系 | `crates/dsh-ui/src/icons.rs`、`crates/dsh-ui/assets/` |
| 标题和工作区同步 | `crates/dsh-ui/src/title_bar.rs`、`crates/dsh-ui/src/workspace.rs` |
| Windows 打包脚本 | `scripts/package_windows.ps1` |

## Git 与验证约定

- 从已同步的 `main` 创建功能分支，沿用命名：`feat/<scope>`、`fix/<scope>` 或 `docs/<scope>`。严禁使用 `codex/` 前缀。
- 功能完成后依次执行：`cargo fmt --check`、`cargo test --workspace`、`cargo check -p dsh-ui`，再检查 `git diff main...HEAD --check`。
- 合并回 `main` 后推送 `origin/main`。UI 或运行时变更时重新运行 `scripts/package_windows.ps1`，确认 ZIP 存在且非空。
- 远端仓库：`https://github.com/oliver-xie666/deepseek-harness-desktop-.git`，主分支为 `main`。
