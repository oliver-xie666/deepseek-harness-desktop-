use crate::icons;
use gpui::{div, prelude::*, px, rgb, Context, FontWeight, IntoElement, Window};

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    Models,
    AgentPresets,
}

pub struct SettingsModal {
    pub is_open: bool,
    pub active_tab: SettingsTab,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
}

impl SettingsModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_tab: SettingsTab::General,
            api_key: "sk-••••••••••••••••".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            model_name: "deepseek-chat".into(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        cx.notify();
    }
}

impl SettingsModal {
    fn nav_rows(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_general = cx.listener(|this, _, _, cx| {
            this.set_tab(SettingsTab::General, cx);
        });
        let handle_models = cx.listener(|this, _, _, cx| {
            this.set_tab(SettingsTab::Models, cx);
        });
        let handle_presets = cx.listener(|this, _, _, cx| {
            this.set_tab(SettingsTab::AgentPresets, cx);
        });

        let active = self.active_tab;

        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_2()
            .child(nav_cell(
                icons::settings(16.0, rgb(0x979da6)),
                "通用设置",
                active == SettingsTab::General,
                handle_general,
            ))
            .child(nav_cell(
                icons::data(16.0, rgb(0x979da6)),
                "模型",
                active == SettingsTab::Models,
                handle_models,
            ))
            .child(nav_cell(
                icons::agent_preset(16.0, rgb(0x979da6)),
                "Agent 预设",
                active == SettingsTab::AgentPresets,
                handle_presets,
            ))
    }

    fn general_body(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(section_label("语言"))
            .child(field_row("显示语言", "简体中文"))
            .child(section_label("外观"))
            .child(field_row("主题", "深色"))
    }

    fn models_body(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(section_label("DEEPSEEK API"))
            .child(field_row("API Key", &self.api_key))
            .child(field_row("Base URL", &self.base_url))
            .child(field_row("模型", &self.model_name))
    }

    fn presets_body(&self) -> impl IntoElement {
        let presets = [
            (
                "标准模式",
                "功能完整的编码 Agent，支持文件编辑、Shell、文件与网页检索、Skills、计划、目标、子代理和工作流。",
            ),
            (
                "PTC 模式",
                "具备标准模式的全部能力，并通过 Code Mode SDK 呈现工具。",
            ),
            (
                "极简模式",
                "仅提供持久 bash 与 str_replace_editor 的双工具编码 Agent。",
            ),
            (
                "创造模式",
                "用于创建自定义 Agent preset，提供运行时检查、插件实验和创作指导。",
            ),
        ];

        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(presets.into_iter().map(|(name, desc)| {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(0x191c22))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child(name),
                    )
                    .child(div().text_xs().text_color(rgb(0x979da6)).child(desc))
            }))
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let handle_close = cx.listener(|this, _, _, cx| {
            this.close(cx);
        });

        div()
            .absolute()
            .inset_0()
            .bg(rgba_black())
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(900.0))
                    .h(px(620.0))
                    .rounded_2xl()
                    .bg(rgb(0x15171b))
                    .border_1()
                    .border_color(rgb(0x282c34))
                    .flex()
                    .overflow_hidden()
                    .shadow_lg()
                    // Left nav rail
                    .child(
                        div()
                            .w_56()
                            .h_full()
                            .bg(rgb(0x13151b))
                            .border_r_1()
                            .border_color(rgb(0x23262d))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .px_5()
                                    .py_4()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("设置"),
                            )
                            .child(self.nav_rows(cx)),
                    )
                    // Content column
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            // Header with close button
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(0x23262d))
                                    .child(
                                        div()
                                            .size_7()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .hover(|s| s.bg(rgb(0x23262d)))
                                            .cursor_pointer()
                                            .on_mouse_down(gpui::MouseButton::Left, handle_close)
                                            .child(icons::close(14.0, rgb(0x979da6))),
                                    ),
                            )
                            // Section body
                            .child(div().flex_1().p_6().overflow_hidden().child(
                                match self.active_tab {
                                    SettingsTab::General => self.general_body().into_any_element(),
                                    SettingsTab::Models => self.models_body().into_any_element(),
                                    SettingsTab::AgentPresets => {
                                        self.presets_body().into_any_element()
                                    }
                                },
                            )),
                    ),
            )
    }
}

/// A nav rail cell (icon + label), highlighted when active.
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
        .px_3()
        .py_2()
        .rounded_lg()
        .bg(if active { rgb(0x1f2228) } else { rgb(0x13151b) })
        .hover(|s| s.bg(rgb(0x1f2228)))
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, handler)
        .child(icon)
        .child(
            div()
                .text_xs()
                .font_weight(if active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if active { rgb(0xffffff) } else { rgb(0x979da6) })
                .child(label.to_string()),
        )
}

fn section_label(text: &str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x979da6))
        .child(text.to_string())
}

fn field_row(label: &str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x979da6))
                .child(label.to_string()),
        )
        .child(
            div()
                .p_2p5()
                .rounded_lg()
                .bg(rgb(0x191c22))
                .border_1()
                .border_color(rgb(0x282c34))
                .text_xs()
                .text_color(rgb(0xe4e4e7))
                .child(value.to_string()),
        )
}

fn rgba_black() -> gpui::Rgba {
    gpui::rgba(0x00000088)
}
