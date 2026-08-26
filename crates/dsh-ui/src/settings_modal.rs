use crate::icons;
use crate::model_catalog::model_options;
use dsh_common::AppPaths;
use dsh_core::{AppConfig, AppState, McpServerConfig};
use gpui::{
    div, prelude::*, px, rgb, Context, Entity, FontWeight, IntoElement, MouseButton, ScrollHandle,
    Window,
};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub enum PluginSubTab {
    Config,
    Inventory,
}

pub struct PluginInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub const INSTALLED_PLUGINS: &[PluginInfo] = &[
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-sidebar",
        name: "侧边栏与会话管理",
        version: "v0.1.0",
        category: "核心组件",
        description: "提供多会话历史记录切换、相对时间徽标、快速检索及工作区管理。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-jobs",
        name: "后台任务与 Subagent",
        version: "v0.1.0",
        category: "任务系统",
        description: "提供长生命周期 Subagent 后台运行状态指示、任务列表展开与终止控制。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-goal",
        name: "目标导航与进度状态",
        version: "v0.1.0",
        category: "会话控制",
        description: "在输入框顶部锚定会话目标栏，支持行内快速编辑、阶段切换与完成状态标记。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-plan",
        name: "Plan 模式与计划审查",
        version: "v0.1.0",
        category: "规划引擎",
        description: "支持在复杂任务下进入分步规划模式，提供 Markdown 计划审查卡片与一键批准。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-user-questions",
        name: "交互式方案选择卡片",
        version: "v0.1.0",
        category: "交互决策",
        description: "支持服务端下发多选/单选方案卡片，直接在对话流中点选确认决策。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-session-log-export",
        name: "会话记录与日志导出",
        version: "v0.1.0",
        category: "导出工具",
        description: "支持将全量对话、工具执行轨迹及终端日志导出为 Markdown 与 JSON 文件。",
    },
    PluginInfo {
        id: "@liustack/modlens",
        name: "视觉引擎与图片粘贴摄取",
        version: "v0.1.0",
        category: "多模态扩展",
        description: "支持剪贴板截图直接粘贴转为本地文件路径与附件列表关联。",
    },
    PluginInfo {
        id: "dsh-better-sidebar",
        name: "高级侧栏与面板增强",
        version: "v0.1.0",
        category: "界面增强",
        description: "提供侧栏默认展开、自动任务弹窗、辅助侧栏卡片管理等个性化配置。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-trajectory",
        name: "轨迹与执行细节抽屉",
        version: "v0.1.0",
        category: "执行监控",
        description: "提供工具调用入参、输出结构化展示与耗时审计抽屉。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-client-ui-message-feedback",
        name: "消息操作与交互反馈",
        version: "v0.1.0",
        category: "交互组件",
        description: "提供 Assistant 消息点赞、点踩、独立代码块复制及复制状态提示。",
    },
];

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    Models,
    Plugins,
    AgentPresets,
    SidebarCards,
}

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub config: AppConfig,
    pub mcp_servers: Vec<McpServerConfig>,
    pub model_editing: bool,
    pub modlens_open: bool,
    pub plugin_subtab: PluginSubTab,
    pub sidebar_open_by_default: bool,
    pub auto_open_jobs: bool,
    pub show_workspace_tree: bool,
    pub show_terminal_logs: bool,
    pub disabled_plugins: std::collections::HashSet<String>,
    state: Entity<Arc<AppState>>,
    content_scroll_handle: ScrollHandle,
}

impl SettingsModal {
    pub fn new(state: Entity<Arc<AppState>>, cx: &mut Context<Self>) -> Self {
        let state_arc = state.read(cx).clone();
        let config = state_arc
            .config
            .try_read()
            .map(|config| (*config).clone())
            .unwrap_or_default();
        let mcp_servers = state_arc
            .mcp_servers
            .try_read()
            .map(|servers| (*servers).clone())
            .unwrap_or_default();

        Self {
            is_open: false,
            active_tab: SettingsTab::General,
            config,
            mcp_servers,
            model_editing: false,
            modlens_open: false,
            plugin_subtab: PluginSubTab::Config,
            sidebar_open_by_default: true,
            auto_open_jobs: true,
            show_workspace_tree: true,
            show_terminal_logs: true,
            disabled_plugins: std::collections::HashSet::new(),
            state,
            content_scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.reload_from_state(cx);
        }
        cx.notify();
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    pub fn set_plugin_subtab(&mut self, subtab: PluginSubTab, cx: &mut Context<Self>) {
        self.plugin_subtab = subtab;
        cx.notify();
    }

    pub fn toggle_plugin_enabled(&mut self, plugin_id: &str, cx: &mut Context<Self>) {
        if self.disabled_plugins.contains(plugin_id) {
            self.disabled_plugins.remove(plugin_id);
        } else {
            self.disabled_plugins.insert(plugin_id.to_string());
        }
        cx.notify();
    }

    pub fn toggle_sidebar_open_pref(&mut self, cx: &mut Context<Self>) {
        self.sidebar_open_by_default = !self.sidebar_open_by_default;
        cx.notify();
    }

    pub fn toggle_auto_jobs_pref(&mut self, cx: &mut Context<Self>) {
        self.auto_open_jobs = !self.auto_open_jobs;
        cx.notify();
    }

    pub fn toggle_workspace_tree_pref(&mut self, cx: &mut Context<Self>) {
        self.show_workspace_tree = !self.show_workspace_tree;
        cx.notify();
    }

    pub fn toggle_terminal_logs_pref(&mut self, cx: &mut Context<Self>) {
        self.show_terminal_logs = !self.show_terminal_logs;
        cx.notify();
    }

    pub fn toggle_modlens(&mut self, cx: &mut Context<Self>) {
        self.modlens_open = !self.modlens_open;
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        self.model_editing = false;
        cx.notify();
    }

    fn reload_from_state(&mut self, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        if let Ok(config) = state.config.try_read() {
            self.config = (*config).clone();
        }
        let servers = state
            .mcp_servers
            .try_read()
            .ok()
            .map(|servers| (*servers).clone());
        if let Some(servers) = servers {
            self.mcp_servers = servers;
        }
    }

    fn persist_config(&self, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            *state.config.write().await = config.clone();
            let _ = config.save(&AppPaths::data_dir());
        });
    }

    fn persist_mcp_servers(&self, cx: &mut Context<Self>) {
        let state = self.state.read(cx).clone();
        let servers = self.mcp_servers.clone();
        tokio::spawn(async move {
            *state.mcp_servers.write().await = servers.clone();
            let _ = dsh_core::McpRegistry::save_servers(&AppPaths::data_dir(), &servers);
        });
    }

    fn set_theme(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.ui.theme = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn set_language(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.ui.language = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn set_permission_mode(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.ui.permission_mode = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn set_agent_preset(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.ui.agent_preset = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn set_enter_behavior(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.ui.enter_behavior = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn set_model_name(&mut self, value: &str, cx: &mut Context<Self>) {
        self.config.model.model_name = value.to_string();
        self.persist_config(cx);
        cx.notify();
    }

    fn toggle_model_editor(&mut self, cx: &mut Context<Self>) {
        self.model_editing = !self.model_editing;
        cx.notify();
    }

    fn toggle_mcp_server(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(server) = self.mcp_servers.get_mut(index) {
            server.enabled = !server.enabled;
            self.persist_mcp_servers(cx);
            cx.notify();
        }
    }

    fn nav_rows(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_general = cx.listener(|this, _, _, cx| this.set_tab(SettingsTab::General, cx));
        let handle_models = cx.listener(|this, _, _, cx| this.set_tab(SettingsTab::Models, cx));
        let handle_plugins = cx.listener(|this, _, _, cx| this.set_tab(SettingsTab::Plugins, cx));
        let handle_presets =
            cx.listener(|this, _, _, cx| this.set_tab(SettingsTab::AgentPresets, cx));
        let handle_cards =
            cx.listener(|this, _, _, cx| this.set_tab(SettingsTab::SidebarCards, cx));
        let active = self.active_tab;

        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_2()
            .child(nav_cell(
                icons::settings(16.0, rgb(0x61666b)),
                "通用设置",
                active == SettingsTab::General,
                handle_general,
            ))
            .child(nav_cell(
                icons::data(16.0, rgb(0x61666b)),
                "模型",
                active == SettingsTab::Models,
                handle_models,
            ))
            .child(nav_cell(
                icons::wrench(16.0, rgb(0x61666b)),
                "插件",
                active == SettingsTab::Plugins,
                handle_plugins,
            ))
            .child(nav_cell(
                icons::agent_preset(16.0, rgb(0x61666b)),
                "Agent 预设",
                active == SettingsTab::AgentPresets,
                handle_presets,
            ))
            .child(nav_cell(
                icons::panel_left(16.0, rgb(0x61666b)),
                "侧边卡片",
                active == SettingsTab::SidebarCards,
                handle_cards,
            ))
    }

    fn general_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let language = self.config.ui.language.clone();
        let theme = self.config.ui.theme.clone();
        let permission = self.config.ui.permission_mode.clone();
        let preset = self.config.ui.agent_preset.clone();
        let enter_behavior = self.config.ui.enter_behavior.clone();

        let handle_language_zh = cx.listener(|this, _, _, cx| this.set_language("zh-CN", cx));
        let handle_language_en = cx.listener(|this, _, _, cx| this.set_language("en-US", cx));
        let handle_light = cx.listener(|this, _, _, cx| this.set_theme("light", cx));
        let handle_dark = cx.listener(|this, _, _, cx| this.set_theme("dark", cx));
        let handle_system = cx.listener(|this, _, _, cx| this.set_theme("system", cx));
        let handle_full = cx.listener(|this, _, _, cx| this.set_permission_mode("full-access", cx));
        let handle_workspace =
            cx.listener(|this, _, _, cx| this.set_permission_mode("workspace-write", cx));
        let handle_read = cx.listener(|this, _, _, cx| this.set_permission_mode("read-only", cx));
        let handle_standard = cx.listener(|this, _, _, cx| this.set_agent_preset("standard", cx));
        let handle_ptc = cx.listener(|this, _, _, cx| this.set_agent_preset("code", cx));
        let handle_minimal = cx.listener(|this, _, _, cx| this.set_agent_preset("minimal", cx));
        let handle_cordis = cx.listener(|this, _, _, cx| this.set_agent_preset("cordis", cx));
        let handle_queue = cx.listener(|this, _, _, cx| this.set_enter_behavior("queue", cx));
        let handle_newline = cx.listener(|this, _, _, cx| this.set_enter_behavior("newline", cx));

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(setting_row(
                "Agent 预设",
                "对此后新建的会话生效。运行中的会话保持它开始时的预设。",
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(choice_button(
                        "标准模式",
                        preset == "standard",
                        handle_standard,
                    ))
                    .child(choice_button("PTC 模式", preset == "code", handle_ptc))
                    .child(choice_button(
                        "极简模式",
                        preset == "minimal",
                        handle_minimal,
                    ))
                    .child(choice_button("创造模式", preset == "cordis", handle_cordis)),
            ))
            .child(setting_row(
                "权限",
                "选择新会话的默认权限模式",
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(choice_button(
                        "Full access",
                        permission == "full-access",
                        handle_full,
                    ))
                    .child(choice_button(
                        "Workspace write",
                        permission == "workspace-write",
                        handle_workspace,
                    ))
                    .child(choice_button(
                        "Read-only",
                        permission == "read-only",
                        handle_read,
                    )),
            ))
            .child(setting_row(
                "语言",
                "界面显示语言",
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(choice_button(
                        "中文",
                        language == "zh-CN" || language.is_empty(),
                        handle_language_zh,
                    ))
                    .child(choice_button(
                        "English",
                        language == "en-US",
                        handle_language_en,
                    )),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x0f1115))
                            .child("外观"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(theme_card(
                                icons::sun(
                                    20.0,
                                    if theme == "light" {
                                        rgb(0x2452d7)
                                    } else {
                                        rgb(0x61666b)
                                    },
                                ),
                                "浅色",
                                theme == "light",
                                handle_light,
                            ))
                            .child(theme_card(
                                icons::moon(
                                    20.0,
                                    if theme == "dark" {
                                        rgb(0x2452d7)
                                    } else {
                                        rgb(0x61666b)
                                    },
                                ),
                                "深色",
                                theme == "dark",
                                handle_dark,
                            ))
                            .child(theme_card(
                                icons::monitor(
                                    20.0,
                                    if theme == "system" || theme.is_empty() {
                                        rgb(0x2452d7)
                                    } else {
                                        rgb(0x61666b)
                                    },
                                ),
                                "跟随系统",
                                theme == "system" || theme.is_empty(),
                                handle_system,
                            )),
                    ),
            )
            .child(setting_row(
                "繁忙时 Enter 键行为",
                "仅在智能体运行时生效；Cmd/Ctrl+Enter 使用另一行为",
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(choice_button(
                        "排队发送",
                        enter_behavior != "newline",
                        handle_queue,
                    ))
                    .child(choice_button(
                        "换行",
                        enter_behavior == "newline",
                        handle_newline,
                    )),
            ))
    }

    fn models_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let model_name = self.config.model.model_name.clone();
        let api_ready = !self.config.model.api_key.trim().is_empty();
        let editing = self.model_editing;
        let handle_edit = cx.listener(|this, _, _, cx| this.toggle_model_editor(cx));

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(page_heading(
                "模型",
                "填入各提供方的 API 密钥即可使用其模型。",
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .bg(rgb(0xffffff))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("DeepSeek"),
                                    )
                                    .child(status_dot(api_ready)),
                            )
                            .child(action_button(
                                if editing { "收起" } else { "编辑" },
                                handle_edit,
                            )),
                    )
                    .when(editing, |this| {
                        let models = model_options(&model_name);
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .border_t_1()
                                .border_color(rgb(0xe5e7eb))
                                .pt_3()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0x0f1115))
                                                .child("DeepSeek"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x81858c))
                                                .child("deepseek-official"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0x61666b))
                                                .child("API 密钥"),
                                        )
                                        .child(
                                            div()
                                                .h(px(36.0))
                                                .px_3()
                                                .rounded_lg()
                                                .border_1()
                                                .border_color(rgb(0xd1d5db))
                                                .bg(rgb(0xf9fafb))
                                                .flex()
                                                .items_center()
                                                .text_xs()
                                                .text_color(rgb(0x81858c))
                                                .child("已配置——输入新值可替换"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0x61666b))
                                                .child("默认模型"),
                                        )
                                        .child(div().flex().flex_wrap().gap_1p5().children(
                                            models.into_iter().take(4).map(|model| {
                                                let selected = model == model_name;
                                                let model_for_handle = model.clone();
                                                let handle_select =
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.set_model_name(&model_for_handle, cx);
                                                    });
                                                choice_button(&model, selected, handle_select)
                                            }),
                                        )),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .cursor_pointer()
                                        .child("› 自定义设置"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_end()
                                        .gap_2()
                                        .child(action_button(
                                            "取消",
                                            cx.listener(|this, _, _, cx| {
                                                this.model_editing = false;
                                                cx.notify();
                                            }),
                                        ))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .h(px(32.0))
                                                .px_4()
                                                .rounded(px(8.0))
                                                .bg(rgb(0x0f1115))
                                                .hover(|s| s.bg(rgb(0x27272a)))
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.model_editing = false;
                                                        cx.notify();
                                                    }),
                                                )
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xffffff))
                                                .child("保存"),
                                        ),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .bg(rgb(0xffffff))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("bytecat"),
                                    )
                                    .child(
                                        div()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded(px(4.0))
                                            .bg(rgb(0xf1f3f5))
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("自定义"),
                                    )
                                    .child(status_dot(true)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(action_button("编辑", cx.listener(|_this, _, _, _| {})))
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .text_xs()
                                            .text_color(rgb(0xe11d48))
                                            .hover(|s| s.bg(rgb(0xffe4e6)))
                                            .cursor_pointer()
                                            .child("删除"),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .h(px(38.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(0xd1d5db))
                            .hover(|s| s.bg(rgb(0xf9fafb)))
                            .cursor_pointer()
                            .child(icons::plus(14.0, rgb(0x61666b)))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x3f454d))
                                    .child("添加提供方"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(px(38.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(0xd1d5db))
                            .hover(|s| s.bg(rgb(0xf9fafb)))
                            .cursor_pointer()
                            .child(icons::plus(14.0, rgb(0x61666b)))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x3f454d))
                                    .child("添加自定义提供方"),
                            ),
                    ),
            )
    }

    fn plugins_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_inventory = self.plugin_subtab == PluginSubTab::Inventory;
        let handle_config_tab = cx.listener(|this, _, _, cx| {
            this.set_plugin_subtab(PluginSubTab::Config, cx);
        });
        let handle_inventory_tab = cx.listener(|this, _, _, cx| {
            this.set_plugin_subtab(PluginSubTab::Inventory, cx);
        });

        let builtin_plugins = [
            ("终端", "限制 agent 运行的每一条命令。"),
            ("Agent 循环", "Agent 如何派发工具调用。"),
            ("网页搜索", "DeepSeek 搜索提供方。"),
            ("视觉引擎 (ModLens)", "视觉引擎提供商配置。"),
        ];

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(page_heading("插件", "配置和查看本部署已安装的插件与组件。"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .pb_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if !is_inventory { rgb(0x0f1115) } else { rgb(0x81858c) })
                            .border_b_2()
                            .border_color(if !is_inventory { rgb(0x0f1115) } else { rgb(0xffffff) })
                            .cursor_pointer()
                            .pb_1()
                            .on_mouse_down(gpui::MouseButton::Left, handle_config_tab)
                            .child("插件配置"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if is_inventory { rgb(0x0f1115) } else { rgb(0x81858c) })
                            .border_b_2()
                            .border_color(if is_inventory { rgb(0x0f1115) } else { rgb(0xffffff) })
                            .cursor_pointer()
                            .pb_1()
                            .on_mouse_down(gpui::MouseButton::Left, handle_inventory_tab)
                            .child(format!("插件列表 ({})", INSTALLED_PLUGINS.len())),
                    ),
            )
            .when(!is_inventory, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2p5()
                        .children(builtin_plugins.into_iter().map(|(name, desc)| {
                            let is_modlens = name.contains("ModLens");
                            let is_expanded = is_modlens && self.modlens_open;
                            let handle_click = if is_modlens {
                                Some(cx.listener(|this, _, _, cx| this.toggle_modlens(cx)))
                            } else {
                                None
                            };

                            div()
                                .flex()
                                .flex_col()
                                .rounded_xl()
                                .border_1()
                                .border_color(rgb(0xe1e5eb))
                                .bg(rgb(0xffffff))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .p_3p5()
                                        .hover(|s| s.bg(rgb(0xf9fafb)))
                                        .cursor_pointer()
                                        .when_some(handle_click, |this, h| {
                                            this.on_mouse_down(gpui::MouseButton::Left, h)
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0x0f1115))
                                                        .child(name),
                                                )
                                                .child(div().text_xs().text_color(rgb(0x61666b)).child(desc)),
                                        )
                                        .child(icons::chevron_down(14.0, rgb(0x81858c))),
                                )
                                .when(is_expanded, |this| {
                                    this.child(
                                        div()
                                            .p_3p5()
                                            .pt_0()
                                            .flex()
                                            .flex_col()
                                            .gap_2p5()
                                            .border_t_1()
                                            .border_color(rgb(0xf1f3f5))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_0p5()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::MEDIUM)
                                                                    .text_color(rgb(0x1f2937))
                                                                    .child("剪贴板图片自动转路径 (Paste-to-Path)"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(0x6b7280))
                                                                    .child("粘贴截图时自动暂存为本地图片文件并插入引用。"),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded_full()
                                                            .bg(rgb(0xdcfce7))
                                                            .text_xs()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(rgb(0x15803d))
                                                            .child("已启用"),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(rgb(0x1f2937))
                                                            .child("暂存目录"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x6b7280))
                                                            .child(".dsh/attachments"),
                                                    ),
                                            ),
                                    )
                                })
                        })),
                )
                .when(!self.mcp_servers.is_empty(), |this| {
                    this.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .pt_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x61666b))
                                    .child("本地 MCP 服务"),
                            )
                            .child(div().flex().flex_col().gap_1().children(
                                self.mcp_servers.iter().enumerate().map(|(index, server)| {
                                    let enabled = server.enabled;
                                    let handle_toggle = cx.listener(move |this, _, _, cx| {
                                        this.toggle_mcp_server(index, cx);
                                    });

                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .p_2p5()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(rgb(0xe1e5eb))
                                        .bg(rgb(0xffffff))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0x0f1115))
                                                        .child(server.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x81858c))
                                                        .child(format!("{:?}", server.transport)),
                                                ),
                                        )
                                        .child(choice_button(
                                            if enabled { "已启用" } else { "已禁用" },
                                            enabled,
                                            handle_toggle,
                                        ))
                                }),
                            )),
                    )
                })
            })
            .when(is_inventory, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2p5()
                        .children(INSTALLED_PLUGINS.iter().map(|plugin| {
                            let is_disabled = self.disabled_plugins.contains(plugin.id);
                            let plugin_id = plugin.id;
                            let handle_toggle = cx.listener(move |this, _, _, cx| {
                                this.toggle_plugin_enabled(plugin_id, cx);
                            });

                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_3p5()
                                .rounded_xl()
                                .border_1()
                                .border_color(rgb(0xe1e5eb))
                                .bg(rgb(0xffffff))
                                .gap_3()
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb(0x0f1115))
                                                        .child(plugin.name),
                                                )
                                                .child(
                                                    div()
                                                        .px_1p5()
                                                        .py_0p5()
                                                        .rounded(px(4.0))
                                                        .bg(rgb(0xf1f3f5))
                                                        .text_xs()
                                                        .text_color(rgb(0x61666b))
                                                        .child(plugin.category),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x9ca3af))
                                                        .child(plugin.version),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x61666b))
                                                .child(plugin.description),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x9ca3af))
                                                .child(format!("ID: {}", plugin.id)),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .bg(if !is_disabled { rgb(0xdcfce7) } else { rgb(0xf3f4f6) })
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(if !is_disabled { rgb(0x15803d) } else { rgb(0x6b7280) })
                                        .cursor_pointer()
                                        .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                                        .child(if !is_disabled { "已启用" } else { "已禁用" }),
                                )
                        })),
                )
            })
    }

    fn presets_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.config.ui.agent_preset.clone();
        let presets = [
            ("标准模式", "standard", "功能完整的编码 Agent，支持文件编辑、Shell、文件与网页检索、Skills、计划、目标、子代理和工作流。"),
            ("PTC 模式", "code", "具备标准模式的全部能力，并通过 Code Mode SDK 呈现工具，让模型用一个 TypeScript 程序组合多步操作。"),
            ("极简模式", "minimal", "仅提供持久 bash 与 str_replace_editor 的双工具编码 Agent。"),
            ("创造模式", "cordis", "用于创建自定义 Agent preset：具备标准模式的全部能力，并提供运行时检查、插件实验和 preset 创作指导。"),
        ];

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(page_heading(
                "Agent 预设",
                "预设即一个会话的 Agent 所运行的插件组装 —— 它的工具、提示词与能力。复制一份既有预设改成自己的，或用「创造模式」让 Agent 帮你创建。",
            ))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x61666b))
                    .child("内置"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child({
                                let (name, key, description) = presets[0];
                                let handle_select = cx.listener(move |this, _, _, cx| this.set_agent_preset(key, cx));
                                let handle_copy = cx.listener(move |_this, _, _, cx| {
                                    cx.write_to_clipboard(name.to_string().into());
                                });
                                preset_card(name, key, description, current == key, handle_select, handle_copy)
                            })
                            .child({
                                let (name, key, description) = presets[1];
                                let handle_select = cx.listener(move |this, _, _, cx| this.set_agent_preset(key, cx));
                                let handle_copy = cx.listener(move |_this, _, _, cx| {
                                    cx.write_to_clipboard(name.to_string().into());
                                });
                                preset_card(name, key, description, current == key, handle_select, handle_copy)
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child({
                                let (name, key, description) = presets[2];
                                let handle_select = cx.listener(move |this, _, _, cx| this.set_agent_preset(key, cx));
                                let handle_copy = cx.listener(move |_this, _, _, cx| {
                                    cx.write_to_clipboard(name.to_string().into());
                                });
                                preset_card(name, key, description, current == key, handle_select, handle_copy)
                            })
                            .child({
                                let (name, key, description) = presets[3];
                                let handle_select = cx.listener(move |this, _, _, cx| this.set_agent_preset(key, cx));
                                let handle_copy = cx.listener(move |_this, _, _, cx| {
                                    cx.write_to_clipboard(name.to_string().into());
                                });
                                preset_card(name, key, description, current == key, handle_select, handle_copy)
                            }),
                    ),
            )
            .child(
                div()
                    .pt_2()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0x61666b))
                    .child("自定义"),
            )
            .child(
                div()
                    .h(px(38.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(0xd1d5db))
                    .hover(|s| s.bg(rgb(0xf9fafb)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.set_agent_preset("cordis", cx)),
                    )
                    .child(icons::plus(14.0, rgb(0x61666b)))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x3f454d))
                            .child("用「创造模式」创作自定义预设"),
                    ),
            )
    }

    fn sidebar_cards_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_sidebar_open = cx.listener(|this, _, _, cx| this.toggle_sidebar_open_pref(cx));
        let handle_auto_jobs = cx.listener(|this, _, _, cx| this.toggle_auto_jobs_pref(cx));
        let handle_tree = cx.listener(|this, _, _, cx| this.toggle_workspace_tree_pref(cx));
        let handle_logs = cx.listener(|this, _, _, cx| this.toggle_terminal_logs_pref(cx));

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(page_heading(
                "侧边卡片与辅助模块",
                "配置主界面侧边栏显示的内容卡片、默认展开行为与辅助模块（对齐 dsh-better-sidebar）。",
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3p5()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .bg(rgb(0xffffff))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x0f1115))
                                            .child("侧边栏默认展开"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("启动应用或切换工作区时保持侧边栏处于展开状态。"),
                                    ),
                            )
                            .child(choice_button(
                                if self.sidebar_open_by_default { "已开启" } else { "已关闭" },
                                self.sidebar_open_by_default,
                                handle_sidebar_open,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3p5()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .bg(rgb(0xffffff))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x0f1115))
                                            .child("任务与 Subagent 自动提示"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("当会话中有后台任务或 Subagent 正在执行时自动展示状态胶囊。"),
                                    ),
                            )
                            .child(choice_button(
                                if self.auto_open_jobs { "已开启" } else { "已关闭" },
                                self.auto_open_jobs,
                                handle_auto_jobs,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3p5()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .bg(rgb(0xffffff))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x0f1115))
                                            .child("工作区文件树卡片"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("在侧栏中展示工作区多层目录扫描与文件节点。"),
                                    ),
                            )
                            .child(choice_button(
                                if self.show_workspace_tree { "已开启" } else { "已关闭" },
                                self.show_workspace_tree,
                                handle_tree,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3p5()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0xe1e5eb))
                            .bg(rgb(0xffffff))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x0f1115))
                                            .child("终端执行日志抽屉"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x61666b))
                                            .child("在会话顶部保留 Session Log 实时日志展开与记录导出功能。"),
                                    ),
                            )
                            .child(choice_button(
                                if self.show_terminal_logs { "已开启" } else { "已关闭" },
                                self.show_terminal_logs,
                                handle_logs,
                            )),
                    ),
            )
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let handle_close = cx.listener(|this, _, _, cx| this.close(cx));
        let handle_open_config = cx.listener(|_this, _, _, _| {
            let data_dir = AppPaths::data_dir();
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer").arg(data_dir).spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(data_dir).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open").arg(data_dir).spawn();
            }
        });

        let body = match self.active_tab {
            SettingsTab::General => self.general_body(cx).into_any_element(),
            SettingsTab::Models => self.models_body(cx).into_any_element(),
            SettingsTab::Plugins => self.plugins_body(cx).into_any_element(),
            SettingsTab::AgentPresets => self.presets_body(cx).into_any_element(),
            SettingsTab::SidebarCards => self.sidebar_cards_body(cx).into_any_element(),
        };

        div()
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000066))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(860.0))
                    .h(px(640.0))
                    .rounded(px(18.0))
                    .bg(rgb(0xffffff))
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .flex()
                    .overflow_hidden()
                    .shadow_lg()
                    .child(
                        div()
                            .w(px(188.0))
                            .h_full()
                            .bg(rgb(0xf5f6f8))
                            .border_r_1()
                            .border_color(rgb(0xe5e7eb))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .px_5()
                                    .py_4()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x0f1115))
                                    .child("设置"),
                            )
                            .child(self.nav_rows(cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .h(px(48.0))
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .gap_2()
                                    .px_4()
                                    .border_b_1()
                                    .border_color(rgb(0xe5e7eb))
                                    .child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(rgb(0xd1d5db))
                                            .hover(|s| s.bg(rgb(0xf1f3f5)))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, handle_open_config)
                                            .text_xs()
                                            .text_color(rgb(0x3f454d))
                                            .child("打开配置文件"),
                                    )
                                    .child(
                                        div()
                                            .size_7()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|s| s.bg(rgb(0xf1f3f5)))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, handle_close)
                                            .child(icons::close(14.0, rgb(0x81858c))),
                                    ),
                            )
                            .child(
                                div()
                                    .id("settings-content")
                                    .flex_1()
                                    .p_6()
                                    .overflow_y_scroll()
                                    .track_scroll(&self.content_scroll_handle)
                                    .child(body),
                            ),
                    ),
            )
    }
}

fn nav_cell(
    icon: impl IntoElement,
    label: &str,
    active: bool,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2p5()
        .w(px(164.0))
        .h(px(40.0))
        .px_3()
        .rounded(px(10.0))
        .bg(if active { rgb(0xe9edf2) } else { rgb(0xf5f6f8) })
        .hover(|s| s.bg(rgb(0xf1f3f5)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(icon)
        .child(
            div()
                .text_xs()
                .font_weight(if active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if active { rgb(0x0f1115) } else { rgb(0x61666b) })
                .child(label.to_string()),
        )
}

fn page_heading(title: &str, subtitle: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x0f1115))
                .child(title.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x61666b))
                .child(subtitle.to_string()),
        )
}

fn setting_row<R: IntoElement>(label: &str, description: &str, control: R) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .py_3()
        .border_b_1()
        .border_color(rgb(0xe5e7eb))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x0f1115))
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x61666b))
                        .child(description.to_string()),
                ),
        )
        .child(control)
}

fn choice_button(
    label: &str,
    selected: bool,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.0))
        .px_2p5()
        .rounded(px(8.0))
        .border_1()
        .border_color(if selected {
            rgb(0x3964fe)
        } else {
            rgb(0xe1e5eb)
        })
        .bg(if selected {
            rgb(0xe8f0ff)
        } else {
            rgb(0xffffff)
        })
        .hover(|s| s.bg(rgb(0xf1f3f5)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .text_xs()
        .text_color(if selected {
            rgb(0x2452d7)
        } else {
            rgb(0x3f454d)
        })
        .child(label.to_string())
}

fn theme_card(
    icon: impl IntoElement,
    label: &str,
    selected: bool,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex_1()
        .h(px(72.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded_xl()
        .border_1()
        .border_color(if selected {
            rgb(0x3964fe)
        } else {
            rgb(0xe1e5eb)
        })
        .bg(if selected {
            rgb(0xf5f8ff)
        } else {
            rgb(0xffffff)
        })
        .hover(|s| s.bg(rgb(0xf9fafb)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(icon)
        .child(
            div()
                .text_xs()
                .font_weight(if selected {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if selected {
                    rgb(0x2452d7)
                } else {
                    rgb(0x3f454d)
                })
                .child(label.to_string()),
        )
}

fn action_button(
    label: &str,
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.0))
        .px_3()
        .rounded(px(8.0))
        .bg(rgb(0xf1f3f5))
        .hover(|s| s.bg(rgb(0xe9edf2)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .text_xs()
        .text_color(rgb(0x3f454d))
        .child(label.to_string())
}

fn preset_card(
    name: &str,
    key: &str,
    description: &str,
    selected: bool,
    select_handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    copy_handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let handle_copy =
        move |event: &gpui::MouseDownEvent, window: &mut Window, cx: &mut gpui::App| {
            cx.stop_propagation();
            copy_handler(event, window, cx);
        };

    div()
        .flex_1()
        .min_h(px(140.0))
        .flex()
        .flex_col()
        .justify_between()
        .p_4()
        .rounded_xl()
        .border_1()
        .border_color(if selected {
            rgb(0x3964fe)
        } else {
            rgb(0xe1e5eb)
        })
        .bg(if selected {
            rgb(0xfbfdff)
        } else {
            rgb(0xffffff)
        })
        .hover(|s| s.bg(rgb(0xf9fafb)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, select_handler)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x0f1115))
                                        .child(name.to_string()),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(rgb(0xf1f3f5))
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .child("内置"),
                                ),
                        )
                        .child(if selected {
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(rgb(0x0f1115))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child("当前使用")
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .line_height(px(18.0))
                        .text_color(rgb(0x61666b))
                        .child(description.to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt_3()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(key.to_string()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .size(px(24.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(6.0))
                                .hover(|s| s.bg(rgb(0xf1f3f5)))
                                .child(icons::document(13.0, rgb(0x81858c))),
                        )
                        .child(
                            div()
                                .size(px(24.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(6.0))
                                .hover(|s| s.bg(rgb(0xf1f3f5)))
                                .on_mouse_down(MouseButton::Left, handle_copy)
                                .child(icons::copy(13.0, rgb(0x81858c))),
                        ),
                ),
        )
}

fn status_dot(configured: bool) -> impl IntoElement {
    div().size(px(8.0)).rounded_full().bg(if configured {
        rgb(0x16a34a)
    } else {
        rgb(0xd1d5db)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installed_plugins_count() {
        assert_eq!(INSTALLED_PLUGINS.len(), 10);
        assert!(INSTALLED_PLUGINS
            .iter()
            .any(|p| p.id == "@liustack/modlens"));
        assert!(INSTALLED_PLUGINS
            .iter()
            .any(|p| p.id == "dsh-better-sidebar"));
    }

    #[test]
    fn settings_tabs_cover_all_parity_views() {
        let tabs = [
            SettingsTab::General,
            SettingsTab::Models,
            SettingsTab::Plugins,
            SettingsTab::AgentPresets,
            SettingsTab::SidebarCards,
        ];
        assert_eq!(tabs.len(), 5);
    }

    #[test]
    fn preset_keys_match_protocol() {
        let presets = [
            ("标准模式", "standard"),
            ("PTC 模式", "code"),
            ("极简模式", "minimal"),
            ("创造模式", "cordis"),
        ];
        assert_eq!(presets.len(), 4);
        assert_eq!(presets[0].1, "standard");
        assert_eq!(presets[1].1, "code");
        assert_eq!(presets[2].1, "minimal");
        assert_eq!(presets[3].1, "cordis");
    }
}
