# UI 持续优化交接

**更新时间：** 2026-08-25

**基线：** `main` / `origin/main` 的 `1f15a65`（`merge: prevent preset copy selection`）
**工作树：** 交接开始时干净；`.reasonix/` 与 `.superpowers/` 均为本机生成的忽略目录。

本文件记录桌面端应用与官方 Harness（本地 3080 端口）逐项视觉与交互对照、已完成的功能项、修复差距以及后续维护规范。

## 已完成

### 状态、会话与工作区

- **会话持久化与状态管理**：`AppState` 为唯一事实来源；会话可新建、选择、重命名、复制、删除，通过稳定 ID 定位；删除当前会话平滑切换后继会话。
- **工作区切换与原生目录选择器**：侧栏“添加工作区...”接入 `rfd::FileDialog` 原生目录选择器，取消或空选安全回退；切换工作区同步更新窗口标题、会话上下文与文件树。
- **可展开折叠的 Explorer 文件树**：支持多层目录递归扫描、展开/折叠状态管理，并提供刷新按钮；文件项可交给系统默认程序打开。
- **侧栏检索与视图排序**：支持会话名称过滤搜索以及“按最近使用”与“按名称”排序切换。

### 对话、轨迹与工具状态

- **对话与轨迹双重视图**：顶部“对话 / 轨迹”自由切换，轨迹视图真实展示工具调用入参、输出与执行耗时；详情抽屉支持参数和输出的一键复制。
- **工具调用状态细化**：按协议真实 `ToolStatus` 细分“运行中（Running）”、“成功（Success）”与“失败（Error）”状态并渲染对应状态徽标与色彩。
- **全量可滚动 Session 日志**：支持展开查看全量终端与执行事件日志，支持日志内容复制与导出。
- **输入区域与命令菜单**：支持文本换行与快捷键提交；支持快速切换权限模式（Full access / Read-only / Ask）、Agent 预设与命令菜单（`/help`、`/model`、`/clear`）。

### 模型与提供方管理

- **提供方层级化模型目录**：模型选项按提供方（DeepSeek 官方、DeepSeek 视觉增强、自定义/bytecat 等）分组呈现，保留自定义模型与扩展模型能力。
- **设置模态框完备性**：常规设置、模型设置、插件配置和 Agent 预设页均接入 GPUI `ScrollHandle` 垂直滚动容器，避免内容裁切；模型密钥配置状态实时指示。
- **预设复制事件隔离**：Agent 预设卡片复制按钮使用 `stop_propagation()`，解决点击复制误触发选中卡片的问题。

### Diff 审查与应用引擎

- **多场景 Unified Diff 引擎**：除常规文件修改外，已支持新建文件（`--- /dev/null`）、删除文件（`+++ /dev/null`）及 `\ No newline at end of file` 无换行末尾格式处理。
- **原子替换与冲突保护**：采用唯一临时文件与上下文严格校验，上下文不匹配时拒绝写入；成功应用或拒绝后自动清除旧的错误提示（`diff_notice`）。
- **单元测试覆盖**：覆盖原子文件写入、现有文件 diff 修改、新文件生成、文件删除、无尾随换行及上下文不匹配拦截等 14 项 core 单元测试。

### 视觉一致性与矢量资产

- **全量矢量化图标**：侧栏搜索、视图选项、添加工作区、会话菜单（更多操作）、详情抽屉扳手/关闭、刷新等全面替换为与官方一致的 SVG 矢量图标，杜绝 Emoji 或文本符号替代。
- **官方 Harness 视觉回归**：已启动官方 Harness（3080 端口），完成设置、侧栏、对话、轨迹、模型下拉与导出弹窗等关键屏的截图与比对验证。

### 工程交付

- **Windows 独立打包**：`scripts/package_windows.ps1` 可构建并打包包含可执行程序及完整 `assets/` 矢量资源目录的 `DeepSeek-Harness-Desktop-Windows-x64.zip`。
- **代码质量与测试**：全 workspace 单元测试 33 项全部通过（`cargo test --workspace`），`cargo fmt` 格式化通过，`cargo check -p dsh-ui` 零警告零报错。

## 后续建议与演进

1. **虚拟列表优化**：对于超长历史会话（数百轮以上），可进一步引入基于 GPUI `uniform_list` 或视图虚拟化渲染。
2. **三方宏兼容性追踪**：持续关注上游 `proc-macro-error2` 等过渡警告的依赖版本更新。
3. **CI/CD 自动化流水线**：接入自动化 GitHub Actions 进行打包与 Windows Release 产物发布。

## 关键实现入口

| 领域 | 入口 |
| --- | --- |
| 会话、持久化、服务端事件、diff 操作 | `crates/dsh-core/src/lib.rs` |
| unified diff 解析、新建/删除与原子写入 | `crates/dsh-core/src/diff_applier.rs` |
| 主对话、轨迹、输入、diff 卡片、会话日志 | `crates/dsh-ui/src/chat_view.rs` |
| 模型目录与提供方分组 | `crates/dsh-ui/src/model_catalog.rs` |
| 设置与模型配置 | `crates/dsh-ui/src/settings_modal.rs` |
| 侧栏、工作区树、搜索排序与会话菜单 | `crates/dsh-ui/src/sidebar.rs`、`crates/dsh-ui/src/dropdown.rs` |
| 矢量图标体系 | `crates/dsh-ui/src/icons.rs`、`crates/dsh-ui/assets/` |
| 标题和工作区同步 | `crates/dsh-ui/src/title_bar.rs`、`crates/dsh-ui/src/workspace.rs` |
| Windows 打包脚本 | `scripts/package_windows.ps1` |

## Git 与验证约定

- 从已同步的 `main` 创建功能分支，沿用命名：`feat/<scope>`、`fix/<scope>` 或 `docs/<scope>`。
- 功能完成后依次执行：`cargo fmt --check`、`cargo test --workspace`、`cargo check -p dsh-ui`，再检查 `git diff main...HEAD --check`。
- 合并回 `main` 后推送 `origin/main`。UI 或运行时变更时重新运行 `scripts/package_windows.ps1`，确认 ZIP 存在且非空。
- 远端仓库：`https://github.com/oliver-xie666/deepseek-harness-desktop-.git`，主分支为 `main`。
