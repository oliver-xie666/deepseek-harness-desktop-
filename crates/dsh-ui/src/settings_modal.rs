use crate::text_input::TextInput;
use dsh_common::AppPaths;
use dsh_core::{AppConfig, AppState, CustomPresetConfig, McpServerConfig, ProviderType};
use gpui::{
    div, prelude::*, px, rgb, Context, Entity, FontWeight, IntoElement, MouseButton,
    MouseDownEvent, ScrollHandle, Window,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PluginSubTab {
    Config,
    Inventory,
}

#[derive(Clone, Debug)]
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
        name: "会话侧边栏",
        version: "v0.1.0",
        category: "UI",
        description: "会话列表管理、工作区切换与持久化树状结构展示。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-agent-loop",
        name: "Agent 决策循环引擎",
        version: "v0.1.0",
        category: "核心",
        description: "多轮工具调用调度、推理流解析与上下文生命周期管控。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-bash",
        name: "终端执行与 Shell 命令",
        version: "v0.1.0",
        category: "工具",
        description: "安全沙箱子进程运行、实时日志推流与退出码监控。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-fs",
        name: "工作区文件系统读写",
        version: "v0.1.0",
        category: "工具",
        description: "本地文件读取、新建、递归目录扫描与文件树感知。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-edit",
        name: "精准字符串与差异比对编辑器",
        version: "v0.1.0",
        category: "工具",
        description: "精准唯一上下文定位替换与局部代码重构应用。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-patch",
        name: "统一差异补丁应用器",
        version: "v0.1.0",
        category: "工具",
        description: "Unified Diff 补丁解析、原子事务合并与冲突审查。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-search",
        name: "代码仓库快速搜索与目录检索",
        version: "v0.1.0",
        category: "工具",
        description: "基于 Ripgrep 原生高性能代码文本搜索与文件模式匹配。",
    },
    PluginInfo {
        id: "@deepseek-ai/dsh-tool-web-search",
        name: "联网搜索与实时资料检索",
        version: "v0.1.0",
        category: "工具",
        description: "在线搜索引擎检索、结构化网页摘要抽取与上下文注入。",
    },
    PluginInfo {
        id: "@liustack/modlens",
        name: "视觉引擎与图片多模态摄取",
        version: "v0.1.0",
        category: "多模态",
        description: "剪贴板截图直接粘贴、本地图片文件解析与视觉 Token 编码。",
    },
    PluginInfo {
        id: "dsh-better-sidebar",
        name: "高级侧栏与面板增强",
        version: "v0.1.0",
        category: "增强",
        description: "侧栏默认展开控制、自动任务面板唤起与卡片视图定制。",
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General,
    Models,
    Plugins,
    AgentPresets,
    SidebarCards,
}

#[derive(Clone, Debug)]
pub struct BuiltInPreset {
    pub key: &'static str,
    pub name: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub file_name: &'static str,
}

pub const BUILT_IN_PRESETS: &[BuiltInPreset] = &[
    BuiltInPreset {
        key: "standard",
        name: "标准模式",
        tag: "全能助手",
        description:
            "通用代码研发模式，启用全部读写、搜索、编辑与终端执行工具，适合日常绝大多数开发任务。",
        file_name: "agent.standard.yml",
    },
    BuiltInPreset {
        key: "code",
        name: "PTC 模式",
        tag: "精准编码",
        description: "专注于单文件与多文件精准重构，优化 Unified Diff 应用速度与代码语法校验。",
        file_name: "agent.code.yml",
    },
    BuiltInPreset {
        key: "minimal",
        name: "极简模式",
        tag: "只读安全",
        description:
            "禁用所有写盘与终端执行工具，仅保留只读与文件搜索能力，适合安全审计与代码审查。",
        file_name: "agent.minimal.yml",
    },
    BuiltInPreset {
        key: "cordis",
        name: "创造模式",
        tag: "深度规划",
        description: "启用 Cordis 扩展架构与自主子任务规划分解，支持长生命周期复杂系统演进。",
        file_name: "agent.cordis.yml",
    },
];

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub plugin_subtab: PluginSubTab,
    pub config: AppConfig,
    pub mcp_servers: Vec<McpServerConfig>,
    pub selected_provider: ProviderType,
    pub api_key_input: Entity<TextInput>,
    pub base_url_input: Entity<TextInput>,
    pub model_name_input: Entity<TextInput>,
    pub temperature_input: Entity<TextInput>,
    pub max_tokens_input: Entity<TextInput>,
    pub plugin_search_input: Entity<TextInput>,
    pub preset_id_input: Entity<TextInput>,
    pub preset_name_input: Entity<TextInput>,
    pub preset_desc_input: Entity<TextInput>,
    pub preset_copy_dialog_open: bool,
    pub preset_copying_from: Option<String>,
    pub delete_confirm_provider: Option<String>,
    pub status_toast: Option<(String, Instant)>,
    pub sidebar_open_by_default: bool,
    pub auto_open_jobs: bool,
    pub show_workspace_tree: bool,
    pub show_terminal_logs: bool,
    pub disabled_plugins: std::collections::HashSet<String>,
    pub shell_expanded: bool,
    pub agent_loop_expanded: bool,
    pub web_search_expanded: bool,
    pub modlens_expanded: bool,
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

        let api_key_input = cx.new(|cx| TextInput::new("sk-...", cx));
        let base_url_input = cx.new(|cx| TextInput::new("https://api.deepseek.com", cx));
        let model_name_input = cx.new(|cx| TextInput::new("deepseek-reasoner", cx));
        let temperature_input = cx.new(|cx| TextInput::new("0.6", cx));
        let max_tokens_input = cx.new(|cx| TextInput::new("8192", cx));
        let plugin_search_input = cx.new(|cx| TextInput::new("搜索已安装插件名称或 ID...", cx));
        let preset_id_input = cx.new(|cx| TextInput::new("custom-agent-preset", cx));
        let preset_name_input = cx.new(|cx| TextInput::new("我的定制预设", cx));
        let preset_desc_input = cx.new(|cx| TextInput::new("预设描述信息...", cx));

        let disabled_set: std::collections::HashSet<String> =
            config.ui.disabled_plugins.iter().cloned().collect();

        let auto_jobs = config.ui.auto_open_jobs;
        let ws_tree = config.ui.show_workspace_tree;
        let term_logs = config.ui.show_terminal_logs;
        let sidebar_open = config.ui.open_files_in_sidebar;
        let provider = config.model.provider;

        let modal = Self {
            is_open: false,
            active_tab: SettingsTab::General,
            plugin_subtab: PluginSubTab::Config,
            config,
            mcp_servers,
            selected_provider: provider,
            api_key_input,
            base_url_input,
            model_name_input,
            temperature_input,
            max_tokens_input,
            plugin_search_input,
            preset_id_input,
            preset_name_input,
            preset_desc_input,
            preset_copy_dialog_open: false,
            preset_copying_from: None,
            delete_confirm_provider: None,
            status_toast: None,
            sidebar_open_by_default: sidebar_open,
            auto_open_jobs: auto_jobs,
            show_workspace_tree: ws_tree,
            show_terminal_logs: term_logs,
            disabled_plugins: disabled_set,
            shell_expanded: true,
            agent_loop_expanded: false,
            web_search_expanded: false,
            modlens_expanded: false,
            state,
            content_scroll_handle: ScrollHandle::new(),
        };

        modal.sync_inputs_from_config(cx);
        modal
    }

    fn sync_inputs_from_config(&self, cx: &mut Context<Self>) {
        let key = self.config.model.api_key.clone();
        let base = self.config.model.base_url.clone();
        let model = self.config.model.model_name.clone();
        let temp = self.config.model.temperature.to_string();
        let max_tok = self.config.model.max_tokens.to_string();

        self.api_key_input.update(cx, |input, cx| {
            input.set_text(&key, cx);
        });
        self.base_url_input.update(cx, |input, cx| {
            input.set_text(&base, cx);
        });
        self.model_name_input.update(cx, |input, cx| {
            input.set_text(&model, cx);
        });
        self.temperature_input.update(cx, |input, cx| {
            input.set_text(&temp, cx);
        });
        self.max_tokens_input.update(cx, |input, cx| {
            input.set_text(&max_tok, cx);
        });
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.reload_from_state(cx);
        }
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        self.preset_copy_dialog_open = false;
        self.delete_confirm_provider = None;
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

    pub fn show_toast(&mut self, message: &str, cx: &mut Context<Self>) {
        self.status_toast = Some((message.to_string(), Instant::now()));
        cx.notify();
    }

    pub fn reload_from_state(&mut self, cx: &mut Context<Self>) {
        let state_arc = self.state.read(cx).clone();
        if let Ok(config) = state_arc.config.try_read() {
            self.config = (*config).clone();
            self.disabled_plugins = self.config.ui.disabled_plugins.iter().cloned().collect();
            self.auto_open_jobs = self.config.ui.auto_open_jobs;
            self.show_workspace_tree = self.config.ui.show_workspace_tree;
            self.show_terminal_logs = self.config.ui.show_terminal_logs;
            self.sidebar_open_by_default = self.config.ui.open_files_in_sidebar;
            self.selected_provider = self.config.model.provider;
            self.sync_inputs_from_config(cx);
        }
        if let Ok(servers) = state_arc.mcp_servers.try_read() {
            self.mcp_servers = (*servers).clone();
        }
        cx.notify();
    }

    pub fn persist_config(&self, cx: &mut Context<Self>) {
        let state_arc = self.state.read(cx).clone();
        let config = self.config.clone();
        let state_entity = self.state.clone();

        state_entity.update(cx, |_state, _cx| {
            if let Ok(mut lock) = state_arc.config.try_write() {
                *lock = config.clone();
            }
        });

        cx.background_executor()
            .spawn(async move {
                let _ = config.save(&AppPaths::config_dir());
            })
            .detach();
    }

    pub fn select_provider(&mut self, provider: ProviderType, cx: &mut Context<Self>) {
        self.selected_provider = provider;
        self.config.model.provider = provider;

        let (default_base, default_model) = match provider {
            ProviderType::DeepSeek => ("https://api.deepseek.com", "deepseek-reasoner"),
            ProviderType::OpenAI => ("https://api.openai.com/v1", "gpt-4o"),
            ProviderType::Anthropic => {
                ("https://api.anthropic.com/v1", "claude-3-5-sonnet-20241022")
            }
            ProviderType::MiniMax => ("https://api.minimax.chat/v1", "abab6.5s-chat"),
            ProviderType::Moonshot => ("https://api.moonshot.cn/v1", "moonshot-v1-32k"),
            ProviderType::Qwen => (
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "qwen-plus",
            ),
            ProviderType::Ollama => ("http://localhost:11434/v1", "llama3.2"),
            ProviderType::VLLM => ("http://localhost:8000/v1", "vllm-model"),
            ProviderType::OpenRouter => ("https://openrouter.ai/api/v1", "deepseek/deepseek-r1"),
            ProviderType::Custom => ("https://api.openai.com/v1", "custom-model"),
        };

        if self.config.model.base_url.is_empty()
            || self.config.model.base_url == "https://api.deepseek.com"
        {
            self.base_url_input.update(cx, |input, cx| {
                input.set_text(default_base, cx);
            });
            self.config.model.base_url = default_base.to_string();
        }

        self.model_name_input.update(cx, |input, cx| {
            input.set_text(default_model, cx);
        });
        self.config.model.model_name = default_model.to_string();

        cx.notify();
    }

    pub fn set_quick_model(&mut self, model_name: &str, cx: &mut Context<Self>) {
        self.model_name_input.update(cx, |input, cx| {
            input.set_text(model_name, cx);
        });
        self.config.model.model_name = model_name.to_string();
        self.show_toast(&format!("已选择模型：{}", model_name), cx);
    }

    pub fn save_model_config(&mut self, cx: &mut Context<Self>) {
        let api_key = self.api_key_input.read(cx).text().trim().to_string();
        let base_url = self.base_url_input.read(cx).text().trim().to_string();
        let model_name = self.model_name_input.read(cx).text().trim().to_string();
        let temp_str = self.temperature_input.read(cx).text().trim();
        let max_tok_str = self.max_tokens_input.read(cx).text().trim();

        let temperature = temp_str.parse::<f32>().unwrap_or(0.6);
        let max_tokens = max_tok_str.parse::<u32>().unwrap_or(8192);

        self.config.model.provider = self.selected_provider;
        self.config.model.api_key = api_key;
        self.config.model.base_url = if base_url.is_empty() {
            "https://api.deepseek.com".to_string()
        } else {
            base_url
        };
        self.config.model.model_name = if model_name.is_empty() {
            "deepseek-reasoner".to_string()
        } else {
            model_name
        };
        self.config.model.temperature = temperature;
        self.config.model.max_tokens = max_tokens;

        self.persist_config(cx);
        self.show_toast("✓ 模型与接口配置已保存并立即生效！", cx);
    }

    pub fn toggle_plugin_enabled(&mut self, plugin_id: &str, cx: &mut Context<Self>) {
        let id_str = plugin_id.to_string();
        let is_currently_disabled = self.disabled_plugins.contains(&id_str);
        if is_currently_disabled {
            self.disabled_plugins.remove(&id_str);
            self.show_toast(&format!("已启用插件 {}", plugin_id), cx);
        } else {
            self.disabled_plugins.insert(id_str);
            self.show_toast(&format!("已禁用插件 {}", plugin_id), cx);
        }
        self.config.ui.disabled_plugins = self.disabled_plugins.iter().cloned().collect();
        self.persist_config(cx);
        cx.notify();
    }

    pub fn set_agent_preset(&mut self, preset_key: &str, cx: &mut Context<Self>) {
        self.config.ui.agent_preset = preset_key.to_string();
        self.persist_config(cx);
        self.show_toast(&format!("默认 Agent 预设已切换为：{}", preset_key), cx);
    }

    pub fn set_permission_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        self.config.ui.permission_mode = mode.to_string();
        self.persist_config(cx);
        self.show_toast(&format!("权限模式已切换为：{}", mode), cx);
    }

    pub fn set_language(&mut self, lang: &str, cx: &mut Context<Self>) {
        self.config.ui.language = lang.to_string();
        self.persist_config(cx);
        self.show_toast(&format!("界面语言已切换为：{}", lang), cx);
    }

    pub fn set_theme(&mut self, theme: &str, cx: &mut Context<Self>) {
        self.config.ui.theme = theme.to_string();
        self.persist_config(cx);
        self.show_toast(&format!("外观主题已切换为：{}", theme), cx);
    }

    pub fn set_enter_behavior(&mut self, behavior: &str, cx: &mut Context<Self>) {
        self.config.ui.enter_behavior = behavior.to_string();
        self.persist_config(cx);
        self.show_toast(&format!("回车按键行为已切换为：{}", behavior), cx);
    }

    pub fn toggle_sidebar_open_pref(&mut self, cx: &mut Context<Self>) {
        self.sidebar_open_by_default = !self.sidebar_open_by_default;
        self.config.ui.open_files_in_sidebar = self.sidebar_open_by_default;
        self.persist_config(cx);
        self.show_toast(
            if self.sidebar_open_by_default {
                "已开启：启动时默认展开侧边栏"
            } else {
                "已关闭：启动时默认折叠侧边栏"
            },
            cx,
        );
    }

    pub fn toggle_auto_jobs_pref(&mut self, cx: &mut Context<Self>) {
        self.auto_open_jobs = !self.auto_open_jobs;
        self.config.ui.auto_open_jobs = self.auto_open_jobs;
        self.persist_config(cx);
        self.show_toast(
            if self.auto_open_jobs {
                "已开启：自动展开后台子任务面板"
            } else {
                "已关闭：后台子任务面板自动弹出"
            },
            cx,
        );
    }

    pub fn toggle_workspace_tree_pref(&mut self, cx: &mut Context<Self>) {
        self.show_workspace_tree = !self.show_workspace_tree;
        self.config.ui.show_workspace_tree = self.show_workspace_tree;
        self.persist_config(cx);
        self.show_toast(
            if self.show_workspace_tree {
                "已开启：侧边栏展示工作区文件树"
            } else {
                "已关闭：侧边栏展示工作区文件树"
            },
            cx,
        );
    }

    pub fn toggle_terminal_logs_pref(&mut self, cx: &mut Context<Self>) {
        self.show_terminal_logs = !self.show_terminal_logs;
        self.config.ui.show_terminal_logs = self.show_terminal_logs;
        self.persist_config(cx);
        self.show_toast(
            if self.show_terminal_logs {
                "已开启：侧边栏展示终端日志与轨迹"
            } else {
                "已关闭：侧边栏展示终端日志与轨迹"
            },
            cx,
        );
    }

    pub fn open_copy_preset_dialog(&mut self, from_preset: &str, cx: &mut Context<Self>) {
        self.preset_copying_from = Some(from_preset.to_string());
        self.preset_copy_dialog_open = true;

        let new_id = format!("custom-{}-copy", from_preset);
        let new_name = format!("{} 副本", from_preset);
        let new_desc = format!("基于 {} 预设定制的扩展配置", from_preset);

        self.preset_id_input.update(cx, |input, cx| {
            input.set_text(&new_id, cx);
        });
        self.preset_name_input.update(cx, |input, cx| {
            input.set_text(&new_name, cx);
        });
        self.preset_desc_input.update(cx, |input, cx| {
            input.set_text(&new_desc, cx);
        });

        cx.notify();
    }

    pub fn confirm_copy_preset(&mut self, cx: &mut Context<Self>) {
        let id = self.preset_id_input.read(cx).text().trim().to_string();
        let name = self.preset_name_input.read(cx).text().trim().to_string();
        let desc = self.preset_desc_input.read(cx).text().trim().to_string();

        if id.is_empty() || name.is_empty() {
            self.show_toast("预设 ID 与名称不能为空！", cx);
            return;
        }

        let custom = CustomPresetConfig {
            id: id.clone(),
            name: name.clone(),
            description: desc,
        };

        self.config.ui.custom_presets.push(custom);
        self.persist_config(cx);
        self.preset_copy_dialog_open = false;
        self.show_toast(&format!("✓ 成功复制并创建新预设：{}", name), cx);
    }

    pub fn delete_custom_preset(&mut self, id: &str, cx: &mut Context<Self>) {
        self.config.ui.custom_presets.retain(|p| p.id != id);
        self.persist_config(cx);
        self.show_toast("已删除自定义预设", cx);
    }

    pub fn open_config_folder(&self) {
        let path = AppPaths::config_dir();
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(path).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(path).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        }
    }

    fn render_general_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_preset = self.config.ui.agent_preset.as_str();
        let current_perm = self.config.ui.permission_mode.as_str();
        let current_lang = self.config.ui.language.as_str();
        let current_theme = self.config.ui.theme.as_str();
        let current_enter = self.config.ui.enter_behavior.as_str();

        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("默认 Agent 预设 (Default Agent Preset)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("新建会话时默认采用的决策模型与工具策略预设。"),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_3()
                            .child(self.render_option_card(
                                "标准模式 (Standard)",
                                "全工具启用，适合绝大多数端到端编码与探索。",
                                current_preset == "standard",
                                cx.listener(|this, _, _, cx| this.set_agent_preset("standard", cx)),
                            ))
                            .child(self.render_option_card(
                                "PTC 模式 (Code)",
                                "精准代码编辑与差异补丁应用，极致编码效率。",
                                current_preset == "code",
                                cx.listener(|this, _, _, cx| this.set_agent_preset("code", cx)),
                            ))
                            .child(self.render_option_card(
                                "极简模式 (Minimal)",
                                "禁用一切写盘与终端执行，适合安全审查与只读探索。",
                                current_preset == "minimal",
                                cx.listener(|this, _, _, cx| this.set_agent_preset("minimal", cx)),
                            ))
                            .child(self.render_option_card(
                                "创造模式 (Cordis)",
                                "多轮深度规划与自主子任务编排，复杂系统演进。",
                                current_preset == "cordis",
                                cx.listener(|this, _, _, cx| this.set_agent_preset("cordis", cx)),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("权限与安全模式 (Permission Mode)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("控制 Agent 在执行系统命令或修改工作区文件时的拦截与确认策略。"),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(3)
                            .gap_3()
                            .child(self.render_option_card(
                                "全自动免确认",
                                "自动执行所有工具，极致流畅",
                                current_perm == "full-access",
                                cx.listener(|this, _, _, cx| {
                                    this.set_permission_mode("full-access", cx)
                                }),
                            ))
                            .child(self.render_option_card(
                                "每次询问确认",
                                "执行写盘与命令前弹窗授权",
                                current_perm == "ask-every-time",
                                cx.listener(|this, _, _, cx| {
                                    this.set_permission_mode("ask-every-time", cx)
                                }),
                            ))
                            .child(self.render_option_card(
                                "只读安全模式",
                                "严格禁止任何写盘与系统命令",
                                current_perm == "read-only",
                                cx.listener(|this, _, _, cx| {
                                    this.set_permission_mode("read-only", cx)
                                }),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("界面语言 (Language)"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(self.render_pill_button(
                                "简体中文 (zh-CN)",
                                current_lang == "zh-CN",
                                cx.listener(|this, _, _, cx| this.set_language("zh-CN", cx)),
                            ))
                            .child(self.render_pill_button(
                                "English (en-US)",
                                current_lang == "en-US",
                                cx.listener(|this, _, _, cx| this.set_language("en-US", cx)),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("外观主题 (Theme)"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(self.render_pill_button(
                                "浅色模式 (Light)",
                                current_theme == "light",
                                cx.listener(|this, _, _, cx| this.set_theme("light", cx)),
                            ))
                            .child(self.render_pill_button(
                                "深色模式 (Dark)",
                                current_theme == "dark",
                                cx.listener(|this, _, _, cx| this.set_theme("dark", cx)),
                            ))
                            .child(self.render_pill_button(
                                "跟随系统 (System)",
                                current_theme == "system",
                                cx.listener(|this, _, _, cx| this.set_theme("system", cx)),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("回车发送行为 (Enter Key Behavior)"),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_3()
                            .child(self.render_option_card(
                                "Enter 发送 / Shift+Enter 换行",
                                "标准即时通讯交互习惯",
                                current_enter == "queue",
                                cx.listener(|this, _, _, cx| this.set_enter_behavior("queue", cx)),
                            ))
                            .child(self.render_option_card(
                                "Ctrl+Enter 发送 / Enter 换行",
                                "多行代码编写与排版习惯",
                                current_enter == "newline",
                                cx.listener(|this, _, _, cx| {
                                    this.set_enter_behavior("newline", cx)
                                }),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_4()
                    .rounded_xl()
                    .bg(rgb(0xf9fafb))
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x111827))
                                    .child("配置与插件本地存储目录"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6b7280))
                                    .child(AppPaths::config_dir().to_string_lossy().to_string()),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xd1d5db))
                            .hover(|s| s.bg(rgb(0xf3f4f6)))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| this.open_config_folder()),
                            )
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0x374151))
                            .child("打开配置目录"),
                    ),
            )
    }

    fn render_models_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_deepseek = self.selected_provider == ProviderType::DeepSeek;
        let is_openai = self.selected_provider == ProviderType::OpenAI;
        let is_anthropic = self.selected_provider == ProviderType::Anthropic;
        let is_minimax = self.selected_provider == ProviderType::MiniMax;
        let is_moonshot = self.selected_provider == ProviderType::Moonshot;
        let is_qwen = self.selected_provider == ProviderType::Qwen;
        let is_ollama = self.selected_provider == ProviderType::Ollama;
        let is_custom = self.selected_provider == ProviderType::Custom;

        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("选择大模型服务商 (Model Provider)"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .child(self.render_provider_chip(
                                "DeepSeek (官方直连)",
                                is_deepseek,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::DeepSeek, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "OpenAI",
                                is_openai,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::OpenAI, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "Anthropic",
                                is_anthropic,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::Anthropic, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "MiniMax",
                                is_minimax,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::MiniMax, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "Moonshot / Kimi",
                                is_moonshot,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::Moonshot, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "Qwen / 通义千问",
                                is_qwen,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::Qwen, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "Ollama (本地模型)",
                                is_ollama,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::Ollama, cx)
                                }),
                            ))
                            .child(self.render_provider_chip(
                                "Custom (兼容协议)",
                                is_custom,
                                cx.listener(|this, _, _, cx| {
                                    this.select_provider(ProviderType::Custom, cx)
                                }),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_5()
                    .rounded_xl()
                    .bg(rgb(0xf9fafb))
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("API Key 接口密钥"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xffffff))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.api_key_input.clone()),
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
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("Base URL 接口端点"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xffffff))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.base_url_input.clone()),
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
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("模型名称 (Model Identifier)"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xffffff))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.model_name_input.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_1p5()
                                    .pt_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x9ca3af))
                                            .child("常用预设:"),
                                    )
                                    .child(self.render_quick_chip(
                                        "deepseek-reasoner",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("deepseek-reasoner", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "deepseek-chat",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("deepseek-chat", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "gpt-4o",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("gpt-4o", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "claude-3-5-sonnet-20241022",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("claude-3-5-sonnet-20241022", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "qwen-plus",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("qwen-plus", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "moonshot-v1-32k",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("moonshot-v1-32k", cx)
                                        }),
                                    ))
                                    .child(self.render_quick_chip(
                                        "llama3.2",
                                        cx.listener(|this, _, _, cx| {
                                            this.set_quick_model("llama3.2", cx)
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x374151))
                                            .child("Temperature (采样温度 0.0 ~ 2.0)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(rgb(0xffffff))
                                            .border_1()
                                            .border_color(rgb(0xd1d5db))
                                            .child(self.temperature_input.clone()),
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
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x374151))
                                            .child("Max Tokens (最大生成限制)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(rgb(0xffffff))
                                            .border_1()
                                            .border_color(rgb(0xd1d5db))
                                            .child(self.max_tokens_input.clone()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_3()
                            .border_t_1()
                            .border_color(rgb(0xe5e7eb))
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .hover(|s| s.bg(rgb(0xfee2e2)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.delete_confirm_provider =
                                                Some("当前服务商".to_string());
                                            cx.notify();
                                        }),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xef4444))
                                    .child("重置与清除此服务商"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0x3964fe))
                                    .hover(|s| s.bg(rgb(0x2d52db)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| this.save_model_config(cx)),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("保存并应用配置"),
                            ),
                    ),
            )
    }

    fn render_plugins_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_config = self.plugin_subtab == PluginSubTab::Config;
        let is_inventory = self.plugin_subtab == PluginSubTab::Inventory;

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pb_2()
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(if is_config {
                                rgb(0xeff6ff)
                            } else {
                                rgb(0x000000).opacity(0.0)
                            })
                            .text_color(if is_config {
                                rgb(0x3964fe)
                            } else {
                                rgb(0x4b5563)
                            })
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_xs()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.set_plugin_subtab(PluginSubTab::Config, cx)
                                }),
                            )
                            .child("插件配置 (Config)"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(if is_inventory {
                                rgb(0xeff6ff)
                            } else {
                                rgb(0x000000).opacity(0.0)
                            })
                            .text_color(if is_inventory {
                                rgb(0x3964fe)
                            } else {
                                rgb(0x4b5563)
                            })
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_xs()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.set_plugin_subtab(PluginSubTab::Inventory, cx)
                                }),
                            )
                            .child(format!(
                                "插件清单与开关 (Inventory - {})",
                                INSTALLED_PLUGINS.len()
                            )),
                    ),
            )
            .child(if is_config {
                self.render_plugins_config_subtab(cx).into_any_element()
            } else {
                self.render_plugins_inventory_subtab(cx).into_any_element()
            })
    }

    fn render_plugins_config_subtab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                self.render_plugin_config_card(
                    "Terminal / Shell 终端执行沙箱",
                    "@deepseek-ai/dsh-tool-bash",
                    "配置默认命令解释器（PowerShell / Bash / CMD）、子进程超时与安全执行环境。",
                    self.shell_expanded,
                    cx.listener(|this, _, _, cx| {
                        this.shell_expanded = !this.shell_expanded;
                        cx.notify();
                    }),
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_xs().text_color(rgb(0x4b5563)).child(
                            "默认 Shell 架构：原生 Windows PowerShell 5.1/7 与 cmd.exe 兼容池",
                        ))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x4b5563))
                                .child("单次命令超时时限：120 秒自动熔断，支持异步长会话 PTY 托管"),
                        ),
                ),
            )
            .child(
                self.render_plugin_config_card(
                    "Agent Loop 决策循环调度引擎",
                    "@deepseek-ai/dsh-agent-loop",
                    "配置单会话最大思考轮数限制、思维链流式双流解析与上下文 Token 熔断阈值。",
                    self.agent_loop_expanded,
                    cx.listener(|this, _, _, cx| {
                        this.agent_loop_expanded = !this.agent_loop_expanded;
                        cx.notify();
                    }),
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x4b5563))
                                .child("最大自主决策轮数：20 轮"),
                        )
                        .child(div().text_xs().text_color(rgb(0x4b5563)).child(
                            "双流推理模式：DeepSeek-R1 reasoning_content 独立面板实时折叠展示",
                        )),
                ),
            )
            .child(
                self.render_plugin_config_card(
                    "Web Search 联网检索工具",
                    "@deepseek-ai/dsh-tool-web-search",
                    "在线实时搜索、抓取关键开发文档与官方规范参考。",
                    self.web_search_expanded,
                    cx.listener(|this, _, _, cx| {
                        this.web_search_expanded = !this.web_search_expanded;
                        cx.notify();
                    }),
                    div().flex().flex_col().gap_3().child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x4b5563))
                            .child("搜索聚合引擎：Baidu / Bing / Google API 自适应调度"),
                    ),
                ),
            )
            .child(
                self.render_plugin_config_card(
                    "ModLens 视觉引擎与多模态摄取",
                    "@liustack/modlens",
                    "支持剪贴板截图直接 Ctrl+V 粘贴、本地图片多模态解析与附件关联。",
                    self.modlens_expanded,
                    cx.listener(|this, _, _, cx| {
                        this.modlens_expanded = !this.modlens_expanded;
                        cx.notify();
                    }),
                    div().flex().flex_col().gap_3().child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x4b5563))
                            .child("图片存储路径：.dsh/attachments/ 自动哈希归档"),
                    ),
                ),
            )
    }

    fn render_plugins_inventory_subtab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let search_text = self.plugin_search_input.read(cx).text().to_lowercase();

        let filtered_plugins: Vec<&PluginInfo> = INSTALLED_PLUGINS
            .iter()
            .filter(|p| {
                if search_text.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&search_text)
                        || p.id.to_lowercase().contains(&search_text)
                        || p.category.to_lowercase().contains(&search_text)
                }
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(rgb(0xf9fafb))
                    .border_1()
                    .border_color(rgb(0xd1d5db))
                    .child(self.plugin_search_input.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(filtered_plugins.into_iter().map(|p| {
                        let is_disabled = self.disabled_plugins.contains(p.id);
                        let plugin_id = p.id;

                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_4()
                            .rounded_xl()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .hover(|s| s.bg(rgb(0xf9fafb)))
                            .child(
                                div()
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
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x111827))
                                                    .child(p.name),
                                            )
                                            .child(
                                                div()
                                                    .px_1p5()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(rgb(0xf3f4f6))
                                                    .text_xs()
                                                    .text_color(rgb(0x4b5563))
                                                    .child(p.category),
                                            )
                                            .child(
                                                div()
                                                    .px_1p5()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(rgb(0xe0f2fe))
                                                    .text_xs()
                                                    .text_color(rgb(0x0369a1))
                                                    .child(p.version),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x6b7280))
                                            .child(p.description),
                                    )
                                    .child(div().text_xs().text_color(rgb(0x9ca3af)).child(p.id)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if !is_disabled {
                                                rgb(0x16a34a)
                                            } else {
                                                rgb(0x9ca3af)
                                            })
                                            .child(if !is_disabled {
                                                "已启用"
                                            } else {
                                                "已禁用"
                                            }),
                                    )
                                    .child(self.render_toggle_switch(
                                        !is_disabled,
                                        cx.listener(move |this, _, _, cx| {
                                            this.toggle_plugin_enabled(plugin_id, cx);
                                        }),
                                    )),
                            )
                    })),
            )
    }

    fn render_plugin_config_card(
        &self,
        title: &'static str,
        id: &'static str,
        desc: &'static str,
        expanded: bool,
        toggle_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
        content: impl IntoElement,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .rounded_xl()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xe5e7eb))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_4()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, toggle_handler)
                    .child(
                        div()
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
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x111827))
                                            .child(title),
                                    )
                                    .child(div().text_xs().text_color(rgb(0x9ca3af)).child(id)),
                            )
                            .child(div().text_xs().text_color(rgb(0x6b7280)).child(desc)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x3964fe))
                            .child(if expanded { "收起 ▲" } else { "配置 ▼" }),
                    ),
            )
            .child(if expanded {
                div()
                    .p_4()
                    .bg(rgb(0xf9fafb))
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(content)
                    .into_any_element()
            } else {
                div().into_any_element()
            })
    }

    fn render_presets_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_default = self.config.ui.agent_preset.as_str();

        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("官方内置预设 (Built-in Presets)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("预定义的智能体角色与能力配比，支持复制为自定义预设定制提示词与工具集。"),
                    ),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .children(BUILT_IN_PRESETS.iter().map(|preset| {
                        let is_default = current_default == preset.key;
                        let p_key = preset.key;

                        div()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .p_4()
                            .rounded_xl()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_default { rgb(0x3964fe) } else { rgb(0xe5e7eb) })
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
                                                            .text_color(rgb(0x111827))
                                                            .child(preset.name),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_1p5()
                                                            .py_0p5()
                                                            .rounded_md()
                                                            .bg(rgb(0xf3f4f6))
                                                            .text_xs()
                                                            .text_color(rgb(0x4b5563))
                                                            .child(preset.tag),
                                                    ),
                                            )
                                            .child(if is_default {
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded_full()
                                                    .bg(rgb(0x10b981))
                                                    .text_xs()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(rgb(0xffffff))
                                                    .child("当前默认")
                                                    .into_any_element()
                                            } else {
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded_full()
                                                    .bg(rgb(0xf3f4f6))
                                                    .hover(|s| s.bg(rgb(0xe5e7eb)))
                                                    .cursor_pointer()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                        this.set_agent_preset(p_key, cx);
                                                    }))
                                                    .text_xs()
                                                    .text_color(rgb(0x374151))
                                                    .child("设为默认")
                                                    .into_any_element()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .line_height(px(18.0))
                                            .text_color(rgb(0x6b7280))
                                            .child(preset.description),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .pt_3()
                                    .border_t_1()
                                    .border_color(rgb(0xf3f4f6))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x9ca3af))
                                            .child(format!("配置: {}", preset.file_name)),
                                    )
                                    .child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded_lg()
                                            .bg(rgb(0xf3f4f6))
                                            .hover(|s| s.bg(rgb(0xe5e7eb)))
                                            .cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.open_copy_preset_dialog(p_key, cx);
                                            }))
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x374151))
                                            .child("复制预设"),
                                    ),
                            )
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("我的自定义预设 (Custom Presets)"),
                    )
                    .child(if self.config.ui.custom_presets.is_empty() {
                        div()
                            .p_4()
                            .rounded_xl()
                            .bg(rgb(0xf9fafb))
                            .border_1()
                            .border_color(rgb(0xe5e7eb))
                            .text_xs()
                            .text_color(rgb(0x9ca3af))
                            .child("暂无自定义预设，您可以点击上方内置预设卡片中的【复制预设】快速创建定制角色。")
                            .into_any_element()
                    } else {
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_4()
                            .children(self.config.ui.custom_presets.iter().map(|preset| {
                                let pid = preset.id.clone();
                                let is_default = current_default == preset.id;

                                div()
                                    .flex()
                                    .flex_col()
                                    .justify_between()
                                    .p_4()
                                    .rounded_xl()
                                    .bg(rgb(0xffffff))
                                    .border_1()
                                    .border_color(if is_default { rgb(0x3964fe) } else { rgb(0xe5e7eb) })
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
                                                            .text_sm()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0x111827))
                                                            .child(preset.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded_full()
                                                            .bg(rgb(0xf3f4f6))
                                                            .hover(|s| s.bg(rgb(0xe5e7eb)))
                                                            .cursor_pointer()
                                                            .on_mouse_down(MouseButton::Left, cx.listener({
                                                                let pid = pid.clone();
                                                                move |this, _, _, cx| this.set_agent_preset(&pid, cx)
                                                            }))
                                                            .text_xs()
                                                            .text_color(rgb(0x374151))
                                                            .child(if is_default { "当前默认" } else { "设为默认" }),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .line_height(px(18.0))
                                                    .text_color(rgb(0x6b7280))
                                                    .child(preset.description.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .pt_3()
                                            .border_t_1()
                                            .border_color(rgb(0xf3f4f6))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x9ca3af))
                                                    .child(format!("ID: {}", preset.id)),
                                            )
                                            .child(
                                                div()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded_lg()
                                                    .hover(|s| s.bg(rgb(0xfee2e2)))
                                                    .cursor_pointer()
                                                    .on_mouse_down(MouseButton::Left, cx.listener({
                                                        let pid = pid.clone();
                                                        move |this, _, _, cx| this.delete_custom_preset(&pid, cx)
                                                    }))
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xef4444))
                                                    .child("删除"),
                                            ),
                                    )
                            }))
                            .into_any_element()
                    }),
            )
    }

    fn render_sidebar_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_open = self.sidebar_open_by_default;
        let auto_jobs = self.auto_open_jobs;
        let ws_tree = self.show_workspace_tree;
        let term_logs = self.show_terminal_logs;

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("侧边栏卡片与个性化偏好 (Sidebar Cards Preferences)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6b7280))
                            .child("自定义侧边栏组件的展示形态、自动弹出行为与开发辅助视图。"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.render_preference_switch_card(
                        "启动时默认展开侧边栏",
                        "应用启动或新建窗口时，保持侧边栏会话历史与工作区处于打开状态。",
                        sidebar_open,
                        cx.listener(|this, _, _, cx| this.toggle_sidebar_open_pref(cx)),
                    ))
                    .child(self.render_preference_switch_card(
                        "自动展开后台任务与 Subagent 面板",
                        "当 Agent 在后台创建长时间运行的子任务或独立作业时，自动在侧边栏弹出任务监控面板。",
                        auto_jobs,
                        cx.listener(|this, _, _, cx| this.toggle_auto_jobs_pref(cx)),
                    ))
                    .child(self.render_preference_switch_card(
                        "在侧边栏显示工作区文件树",
                        "在侧边栏实时扫描并展示当前项目工作区的文件与目录层级结构，方便快速查阅。",
                        ws_tree,
                        cx.listener(|this, _, _, cx| this.toggle_workspace_tree_pref(cx)),
                    ))
                    .child(self.render_preference_switch_card(
                        "在侧边栏显示终端日志与执行轨迹",
                        "在侧边栏底部提供实时终端子进程日志、退出码状态与多轮工具调用轨迹流。",
                        term_logs,
                        cx.listener(|this, _, _, cx| this.toggle_terminal_logs_pref(cx)),
                    )),
            )
    }

    fn render_option_card(
        &self,
        title: &'static str,
        desc: &'static str,
        selected: bool,
        select_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(if selected {
                rgb(0x3964fe)
            } else {
                rgb(0xe5e7eb)
            })
            .bg(if selected {
                rgb(0xf8faff)
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
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if selected {
                                        rgb(0x3964fe)
                                    } else {
                                        rgb(0x111827)
                                    })
                                    .child(title),
                            )
                            .child(if selected {
                                div()
                                    .size(px(14.0))
                                    .rounded_full()
                                    .bg(rgb(0x3964fe))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .child("✓")
                                    .into_any_element()
                            } else {
                                div()
                                    .size(px(14.0))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .into_any_element()
                            }),
                    )
                    .child(div().text_xs().text_color(rgb(0x6b7280)).child(desc)),
            )
    }

    fn render_pill_button(
        &self,
        label: &'static str,
        selected: bool,
        click_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .px_4()
            .py_2()
            .rounded_lg()
            .border_1()
            .border_color(if selected {
                rgb(0x3964fe)
            } else {
                rgb(0xd1d5db)
            })
            .bg(if selected {
                rgb(0xeff6ff)
            } else {
                rgb(0xffffff)
            })
            .text_color(if selected {
                rgb(0x3964fe)
            } else {
                rgb(0x374151)
            })
            .font_weight(FontWeight::MEDIUM)
            .text_xs()
            .hover(|s| s.bg(rgb(0xf3f4f6)))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, click_handler)
            .child(label)
    }

    fn render_provider_chip(
        &self,
        name: &'static str,
        selected: bool,
        click_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .px_3()
            .py_1p5()
            .rounded_lg()
            .border_1()
            .border_color(if selected {
                rgb(0x3964fe)
            } else {
                rgb(0xe5e7eb)
            })
            .bg(if selected {
                rgb(0x3964fe)
            } else {
                rgb(0xffffff)
            })
            .text_color(if selected {
                rgb(0xffffff)
            } else {
                rgb(0x374151)
            })
            .font_weight(FontWeight::MEDIUM)
            .text_xs()
            .hover(|s| if !selected { s.bg(rgb(0xf3f4f6)) } else { s })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, click_handler)
            .child(name)
    }

    fn render_quick_chip(
        &self,
        name: &'static str,
        click_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .px_2()
            .py_0p5()
            .rounded_md()
            .bg(rgb(0xe5e7eb))
            .hover(|s| s.bg(rgb(0xd1d5db)))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, click_handler)
            .text_xs()
            .text_color(rgb(0x374151))
            .child(name)
    }

    fn render_preference_switch_card(
        &self,
        title: &'static str,
        desc: &'static str,
        checked: bool,
        toggle_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .p_4()
            .rounded_xl()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xe5e7eb))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child(title),
                    )
                    .child(div().text_xs().text_color(rgb(0x6b7280)).child(desc)),
            )
            .child(self.render_toggle_switch(checked, toggle_handler))
    }

    fn render_toggle_switch(
        &self,
        checked: bool,
        toggle_handler: impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> impl IntoElement {
        div()
            .w(px(40.0))
            .h(px(22.0))
            .rounded_full()
            .bg(if checked {
                rgb(0x10b981)
            } else {
                rgb(0xd1d5db)
            })
            .p(px(2.0))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, toggle_handler)
            .child(
                div()
                    .size(px(18.0))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .shadow_sm()
                    .ml(if checked { px(18.0) } else { px(0.0) }),
            )
    }

    fn render_preset_copy_dialog(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let from_preset = self
            .preset_copying_from
            .clone()
            .unwrap_or_else(|| "standard".to_string());

        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).opacity(0.5))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(480.0))
                    .p_6()
                    .rounded_2xl()
                    .bg(rgb(0xffffff))
                    .shadow_2xl()
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x111827))
                                    .child("复制并定制 Agent 预设"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0xeff6ff))
                                    .text_color(rgb(0x3964fe))
                                    .child(format!("源预设: {}", from_preset)),
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
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("新预设唯一标识 (ID)"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xf9fafb))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.preset_id_input.clone()),
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
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("预设显示名称 (Name)"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xf9fafb))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.preset_name_input.clone()),
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
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0x374151))
                                    .child("预设用途描述 (Description)"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xf9fafb))
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .child(self.preset_desc_input.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .pt_2()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .hover(|s| s.bg(rgb(0xf3f4f6)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.preset_copy_dialog_open = false;
                                            cx.notify();
                                        }),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x374151))
                                    .child("取消"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0x3964fe))
                                    .hover(|s| s.bg(rgb(0x2d52db)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| this.confirm_copy_preset(cx)),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("确认复制"),
                            ),
                    ),
            )
    }

    fn render_delete_confirm_dialog(
        &mut self,
        provider_name: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let prov_clone = provider_name.to_string();
        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).opacity(0.5))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(400.0))
                    .p_6()
                    .rounded_2xl()
                    .bg(rgb(0xffffff))
                    .shadow_2xl()
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x111827))
                            .child("确认重置与清除？"),
                    )
                    .child(div().text_xs().text_color(rgb(0x6b7280)).child(format!(
                        "确定要重置 {} 的所有输入密钥与自定义端点吗？重置后将恢复默认配置。",
                        prov_clone
                    )))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .pt_2()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(rgb(0xd1d5db))
                                    .hover(|s| s.bg(rgb(0xf3f4f6)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.delete_confirm_provider = None;
                                            cx.notify();
                                        }),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0x374151))
                                    .child("取消"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(rgb(0xef4444))
                                    .hover(|s| s.bg(rgb(0xdc2626)))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.api_key_input.update(cx, |i, cx| i.clear(cx));
                                            this.delete_confirm_provider = None;
                                            this.show_toast("已清除接口密钥", cx);
                                        }),
                                    )
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("确认清除"),
                            ),
                    ),
            )
    }
}

impl gpui::Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Clean up expired toast
        if let Some((_, timestamp)) = self.status_toast {
            if timestamp.elapsed() > Duration::from_secs(3) {
                self.status_toast = None;
            }
        }

        let is_general = self.active_tab == SettingsTab::General;
        let is_models = self.active_tab == SettingsTab::Models;
        let is_plugins = self.active_tab == SettingsTab::Plugins;
        let is_presets = self.active_tab == SettingsTab::AgentPresets;
        let is_sidebar = self.active_tab == SettingsTab::SidebarCards;

        let content = match self.active_tab {
            SettingsTab::General => self.render_general_tab(cx).into_any_element(),
            SettingsTab::Models => self.render_models_tab(cx).into_any_element(),
            SettingsTab::Plugins => self.render_plugins_tab(cx).into_any_element(),
            SettingsTab::AgentPresets => self.render_presets_tab(cx).into_any_element(),
            SettingsTab::SidebarCards => self.render_sidebar_tab(cx).into_any_element(),
        };

        let copy_dialog_layer = if self.preset_copy_dialog_open {
            self.render_preset_copy_dialog(cx).into_any_element()
        } else {
            div().into_any_element()
        };

        let delete_provider = self.delete_confirm_provider.clone();
        let delete_confirm_layer = if let Some(ref provider) = delete_provider {
            self.render_delete_confirm_dialog(provider, cx)
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let toast_layer = if let Some((ref msg, _)) = self.status_toast {
            div()
                .absolute()
                .top(px(20.0))
                .right(px(20.0))
                .px_4()
                .py_2()
                .rounded_lg()
                .bg(rgb(0x111827))
                .border_1()
                .border_color(rgb(0x374151))
                .shadow_lg()
                .flex()
                .items_center()
                .gap_2()
                .child(div().size(px(8.0)).rounded_full().bg(rgb(0x10b981)))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xffffff))
                        .child(msg.clone()),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).opacity(0.45))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(820.0))
                    .h(px(640.0))
                    .rounded_2xl()
                    .bg(rgb(0xffffff))
                    .shadow_2xl()
                    .border_1()
                    .border_color(rgb(0xe5e7eb))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .relative()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_6()
                            .py_4()
                            .border_b_1()
                            .border_color(rgb(0xe5e7eb))
                            .child(
                                div().flex().items_center().gap_3().child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x111827))
                                        .child("系统设置与偏好 (Settings)"),
                                ),
                            )
                            .child(
                                div()
                                    .size(px(28.0))
                                    .rounded_lg()
                                    .hover(|s| s.bg(rgb(0xf3f4f6)))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| this.close(cx)),
                                    )
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x6b7280))
                                    .child("✕"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .overflow_hidden()
                            .child(
                                div()
                                    .w(px(200.0))
                                    .p_3()
                                    .bg(rgb(0xf9fafb))
                                    .border_r_1()
                                    .border_color(rgb(0xe5e7eb))
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(if is_general {
                                                rgb(0xeff6ff)
                                            } else {
                                                rgb(0x000000).opacity(0.0)
                                            })
                                            .text_color(if is_general {
                                                rgb(0x3964fe)
                                            } else {
                                                rgb(0x374151)
                                            })
                                            .font_weight(if is_general {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.set_tab(SettingsTab::General, cx)
                                                }),
                                            )
                                            .child("⚙ 常规设置 (General)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(if is_models {
                                                rgb(0xeff6ff)
                                            } else {
                                                rgb(0x000000).opacity(0.0)
                                            })
                                            .text_color(if is_models {
                                                rgb(0x3964fe)
                                            } else {
                                                rgb(0x374151)
                                            })
                                            .font_weight(if is_models {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.set_tab(SettingsTab::Models, cx)
                                                }),
                                            )
                                            .child("🧠 模型与服务商 (Models)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(if is_plugins {
                                                rgb(0xeff6ff)
                                            } else {
                                                rgb(0x000000).opacity(0.0)
                                            })
                                            .text_color(if is_plugins {
                                                rgb(0x3964fe)
                                            } else {
                                                rgb(0x374151)
                                            })
                                            .font_weight(if is_plugins {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.set_tab(SettingsTab::Plugins, cx)
                                                }),
                                            )
                                            .child("🧩 插件与扩展 (Plugins)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(if is_presets {
                                                rgb(0xeff6ff)
                                            } else {
                                                rgb(0x000000).opacity(0.0)
                                            })
                                            .text_color(if is_presets {
                                                rgb(0x3964fe)
                                            } else {
                                                rgb(0x374151)
                                            })
                                            .font_weight(if is_presets {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.set_tab(SettingsTab::AgentPresets, cx)
                                                }),
                                            )
                                            .child("🤖 智能体预设 (Presets)"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .bg(if is_sidebar {
                                                rgb(0xeff6ff)
                                            } else {
                                                rgb(0x000000).opacity(0.0)
                                            })
                                            .text_color(if is_sidebar {
                                                rgb(0x3964fe)
                                            } else {
                                                rgb(0x374151)
                                            })
                                            .font_weight(if is_sidebar {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::NORMAL
                                            })
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.set_tab(SettingsTab::SidebarCards, cx)
                                                }),
                                            )
                                            .child("📐 侧边栏卡片 (Sidebar)"),
                                    ),
                            )
                            .child(
                                div()
                                    .id("settings-content")
                                    .flex_1()
                                    .p_6()
                                    .overflow_y_scroll()
                                    .track_scroll(&self.content_scroll_handle)
                                    .child(content),
                            ),
                    )
                    .child(copy_dialog_layer)
                    .child(delete_confirm_layer)
                    .child(toast_layer),
            )
            .into_any_element()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn settings_tabs_cover_all_parity_views() {
        assert_eq!(SettingsTab::General as usize, 0);
        assert_eq!(SettingsTab::Models as usize, 1);
        assert_eq!(SettingsTab::Plugins as usize, 2);
        assert_eq!(SettingsTab::AgentPresets as usize, 3);
        assert_eq!(SettingsTab::SidebarCards as usize, 4);
    }

    #[test]
    fn test_installed_plugins_count() {
        assert_eq!(INSTALLED_PLUGINS.len(), 10);
    }

    #[test]
    fn preset_keys_match_protocol() {
        assert_eq!(BUILT_IN_PRESETS[0].key, "standard");
        assert_eq!(BUILT_IN_PRESETS[1].key, "code");
        assert_eq!(BUILT_IN_PRESETS[2].key, "minimal");
        assert_eq!(BUILT_IN_PRESETS[3].key, "cordis");
    }
}
