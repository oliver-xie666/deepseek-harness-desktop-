use crate::icons;
use crate::model_catalog::model_options;
use dsh_common::AppPaths;
use dsh_core::{AppConfig, AppState, McpServerConfig, McpTransport};
use gpui::{
    div, prelude::*, px, rgb, Context, Entity, FontWeight, IntoElement, MouseButton, ScrollHandle,
    Window,
};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    Models,
    Plugins,
    AgentPresets,
}

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub config: AppConfig,
    pub mcp_servers: Vec<McpServerConfig>,
    pub model_editing: bool,
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
                icons::agent_preset(16.0, rgb(0x61666b)),
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
            .gap_1()
            .child(page_heading("通用设置", "调整新会话、权限、语言和外观。"))
            .child(setting_row(
                "Agent 预设",
                "新会话默认使用的 Agent 能力组合。",
                div().flex().gap_1().children([
                    choice_button("标准", preset == "standard", handle_standard).into_any_element(),
                    choice_button("PTC", preset == "code", handle_ptc).into_any_element(),
                    choice_button("极简", preset == "minimal", handle_minimal).into_any_element(),
                    choice_button("创造", preset == "cordis", handle_cordis).into_any_element(),
                ]),
            ))
            .child(setting_row(
                "权限",
                "控制 Agent 是否可以直接执行工具操作。",
                div().flex().gap_1().children([
                    choice_button("Full access", permission == "full-access", handle_full)
                        .into_any_element(),
                    choice_button(
                        "Workspace Write",
                        permission == "workspace-write",
                        handle_workspace,
                    )
                    .into_any_element(),
                    choice_button("Read Only", permission == "read-only", handle_read)
                        .into_any_element(),
                ]),
            ))
            .child(setting_row(
                "语言",
                "设置应用界面显示语言。",
                div().flex().gap_1().children([
                    choice_button("中文", language == "zh-CN", handle_language_zh)
                        .into_any_element(),
                    choice_button("English", language == "en-US", handle_language_en)
                        .into_any_element(),
                ]),
            ))
            .child(setting_row(
                "外观",
                "选择浅色、深色或跟随系统。",
                div().flex().gap_1().children([
                    choice_button("浅色", theme == "light", handle_light).into_any_element(),
                    choice_button("深色", theme == "dark", handle_dark).into_any_element(),
                    choice_button("跟随系统", theme == "system", handle_system).into_any_element(),
                ]),
            ))
            .child(setting_row(
                "繁忙时 Enter 键行为",
                "Agent 工作时按 Enter 的处理方式。",
                div().flex().gap_1().children([
                    choice_button("排队发送", enter_behavior == "queue", handle_queue)
                        .into_any_element(),
                    choice_button("换行", enter_behavior == "newline", handle_newline)
                        .into_any_element(),
                ]),
            ))
    }

    fn models_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.model_editing;
        let model_name = self.config.model.model_name.clone();
        let api_ready = !self.config.model.api_key.trim().is_empty();
        let handle_edit = cx.listener(|this, _, _, cx| this.toggle_model_editor(cx));
        let models = model_options(&model_name);

        div()
            .flex()
            .flex_col()
            .gap_3()
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
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x61666b))
                                    .child(if api_ready {
                                        "API 密钥已配置"
                                    } else {
                                        "未配置 API 密钥"
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .border_t_1()
                            .border_color(rgb(0xe5e7eb))
                            .pt_3()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div().text_xs().text_color(rgb(0x61666b)).child("当前模型"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x0f1115))
                                            .child(model_name.clone()),
                                    ),
                            )
                            .child(action_button("编辑", handle_edit)),
                    )
                    .when(editing, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .border_t_1()
                                .border_color(rgb(0xe5e7eb))
                                .pt_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x61666b))
                                        .child("可用模型"),
                                )
                                .child(div().flex().gap_1().children(models.into_iter().map(
                                    |model| {
                                        let selected = model_name == model;
                                        let model_for_handler = model.clone();
                                        let handle = cx.listener(move |this, _, _, cx| {
                                            this.set_model_name(&model_for_handler, cx);
                                        });
                                        choice_button(&model, selected, handle).into_any_element()
                                    },
                                ))),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .border_1()
                    .border_color(rgb(0xe1e5eb))
                    .bg(rgb(0xf9fafb))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x61666b))
                            .child("支持添加自定义提供方"),
                    )
                    .child(div().text_xs().text_color(rgb(0x81858c)).child("即将支持")),
            )
    }

    fn plugins_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(page_heading("插件", "配置和查看本地已安装的工具服务。"))
            .child(div().flex().flex_col().gap_1().children(
                self.mcp_servers.iter().enumerate().map(|(index, server)| {
                    let handle =
                        cx.listener(move |this, _, _, cx| this.toggle_mcp_server(index, cx));
                    let enabled = server.enabled;
                    let transport = match server.transport {
                        McpTransport::Stdio => "stdio",
                        McpTransport::Sse => "sse",
                    };
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_3()
                        .border_b_1()
                        .border_color(rgb(0xe5e7eb))
                        .hover(|s| s.bg(rgb(0xf9fafb)))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, handle)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(server.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .child(server.description.clone()),
                                )
                                .child(div().text_xs().text_color(rgb(0x81858c)).child(transport)),
                        )
                        .child(toggle_mark(enabled))
                }),
            ))
    }

    fn presets_body(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.config.ui.agent_preset.clone();
        let presets = [
            ("标准模式", "standard", "功能完整的编码 Agent，支持文件编辑、Shell、文件与网页检索、Skills、计划、目标、子代理和工作流。"),
            ("PTC 模式", "code", "具备标准模式的全部能力，并通过 Code Mode SDK 呈现工具。"),
            ("极简模式", "minimal", "仅提供持久 bash 与 str_replace_editor 的双工具编码 Agent。"),
            ("创造模式", "cordis", "用于创建自定义 Agent preset，并提供运行时检查与创作指导。"),
        ];

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(page_heading(
                "Agent 预设",
                "预设决定一个会话所运行的插件、提示词与能力。",
            ))
            .children(presets.into_iter().map(|(name, key, description)| {
                let handle_select =
                    cx.listener(move |this, _, _, cx| this.set_agent_preset(key, cx));
                let handle_copy = cx.listener(move |_this, _, _, cx| {
                    cx.write_to_clipboard(name.to_string().into());
                });
                preset_card(
                    name,
                    description,
                    current == key,
                    handle_select,
                    handle_copy,
                )
            }))
            .child(action_button(
                "用创造模式创作自定义预设",
                cx.listener(|this, _, _, cx| this.set_agent_preset("cordis", cx)),
            ))
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let handle_close = cx.listener(|this, _, _, cx| this.close(cx));
        let body = match self.active_tab {
            SettingsTab::General => self.general_body(cx).into_any_element(),
            SettingsTab::Models => self.models_body(cx).into_any_element(),
            SettingsTab::Plugins => self.plugins_body(cx).into_any_element(),
            SettingsTab::AgentPresets => self.presets_body(cx).into_any_element(),
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
                                    .px_3()
                                    .border_b_1()
                                    .border_color(rgb(0xe5e7eb))
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

#[cfg(test)]
mod tests {
    #[test]
    fn settings_body_uses_gpui_vertical_scroll_container() {
        let source = include_str!("settings_modal.rs");
        assert!(
            source.matches(".overflow_y_scroll()").count() > 1,
            "settings content must use GPUI's vertical scroll container"
        );
    }
}

fn page_heading(title: &str, description: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .pb_3()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x0f1115))
                .child(title.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x61666b))
                .child(description.to_string()),
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
    description: &str,
    selected: bool,
    select_handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    copy_handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
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
        .on_mouse_down(MouseButton::Left, select_handler)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x0f1115))
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x61666b))
                        .child(description.to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(if selected {
                    icons::check(16.0, rgb(0x3964fe)).into_any_element()
                } else {
                    div().size(px(16.0)).into_any_element()
                })
                .child(action_button("复制", copy_handler)),
        )
}

fn status_dot(configured: bool) -> impl IntoElement {
    div().size(px(8.0)).rounded_full().bg(if configured {
        rgb(0x16a34a)
    } else {
        rgb(0xd1d5db)
    })
}

fn toggle_mark(enabled: bool) -> impl IntoElement {
    div()
        .size(px(22.0))
        .rounded_full()
        .bg(if enabled {
            rgb(0x3964fe)
        } else {
            rgb(0xe1e5eb)
        })
        .flex()
        .items_center()
        .justify_center()
        .child(if enabled {
            icons::check(12.0, rgb(0xffffff)).into_any_element()
        } else {
            div().size(px(8.0)).into_any_element()
        })
}
