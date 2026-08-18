use crate::icons;
use gpui::{deferred, div, prelude::*, px, rgb, Context, FontWeight, IntoElement, Rgba, Window};

// Shared palette matching the official light theme in 01.png and 02.png.
fn bg_menu() -> Rgba {
    rgb(0xffffff)
}
fn bg_menu_item_hover() -> Rgba {
    rgb(0xf1f3f5)
}
fn border_menu() -> Rgba {
    rgb(0xe1e5eb)
}
fn text_primary() -> Rgba {
    rgb(0x0f1115)
}
fn text_muted() -> Rgba {
    rgb(0x61666b)
}
fn text_faint() -> Rgba {
    rgb(0x81858c)
}

/// The workspace picker chip shown on the new-session hero: a folder glyph
/// (open once a workspace is chosen, closed otherwise) + label + chevron.
pub struct WorkspaceSelector {
    pub is_open: bool,
    pub current_workspace: String,
    pub has_selection: bool,
}

impl WorkspaceSelector {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_workspace: "deepseek-harness-desktop".into(),
            has_selection: true,
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn set_workspace(&mut self, label: &str, cx: &mut Context<Self>) {
        self.current_workspace = label.to_string();
        self.has_selection = true;
        self.is_open = false;
        cx.notify();
    }
}

impl Render for WorkspaceSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.is_open;
        let current = self.current_workspace.clone();
        let has_selection = self.has_selection;

        let handle_toggle = cx.listener(|this, _, _, cx| {
            this.toggle(cx);
        });

        // Workspace rows render folder icon + title (official picker: no
        // description, "add workspace…" pinned in a footer below a divider).
        let workspaces: Vec<(&str, &str)> = vec![
            ("deepseek-harness-desktop", "deepseek-harness-desktop"),
            ("zed-fluid", "zed-fluid"),
        ];

        div()
            .relative()
            // Anchor chip button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .h(px(28.0))
                    .rounded(px(16.0))
                    .hover(|s| s.text_color(text_primary()))
                    .cursor_pointer()
                    .text_size(px(13.0))
                    .text_color(text_muted())
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child(if has_selection {
                        icons::folder_open(16.0, text_muted()).into_any_element()
                    } else {
                        icons::folder_close(16.0, text_muted()).into_any_element()
                    })
                    .child(current.clone())
                    .child(icons::chevron_down(14.0, text_faint())),
            )
            // Floating picker menu (deferred draw paints above the composer).
            .when(is_open, |this| {
                this.child(deferred(
                    div()
                        .absolute()
                        .top(px(32.0))
                        .left(px(0.0))
                        .w(px(260.0))
                        .rounded_xl()
                        .bg(bg_menu())
                        .border_1()
                        .border_color(border_menu())
                        .shadow_lg()
                        .p_1p5()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(workspaces.into_iter().map(|(name, key)| {
                            let key = key.to_string();
                            let handle_select = cx.listener(move |this, _, _, cx| {
                                this.set_workspace(&key, cx);
                            });
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py_1p5()
                                .rounded(px(10.0))
                                .hover(|s| s.bg(bg_menu_item_hover()))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_select)
                                .child(icons::folder_close(16.0, text_muted()))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(text_primary())
                                        .child(name),
                                )
                        }))
                        // Divider + pinned "add workspace…" action
                        .child(div().h(px(1.0)).bg(border_menu()).my_1())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py_1p5()
                                .rounded(px(10.0))
                                .hover(|s| s.bg(bg_menu_item_hover()))
                                .cursor_pointer()
                                .child(icons::plus(16.0, text_muted()))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(text_muted())
                                        .child("添加工作区…"),
                                ),
                        ),
                ))
            })
    }
}

/// The agent-preset chip (official "mode" selector): preset glyph + localized
/// name + chevron; the menu lists name + description with a trailing check on
/// the staged choice.
pub struct AgentPresetSelector {
    pub is_open: bool,
    pub current: String,
}

impl AgentPresetSelector {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current: "标准模式".into(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn set_preset(&mut self, name: &str, cx: &mut Context<Self>) {
        self.current = name.to_string();
        self.is_open = false;
        cx.notify();
    }
}

impl Render for AgentPresetSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.is_open;
        let current = self.current.clone();

        let handle_toggle = cx.listener(|this, _, _, cx| {
            this.toggle(cx);
        });

        // Built-in presets (zh localized name + description).
        let presets: Vec<(&str, &str)> = vec![
            (
                "标准模式",
                "功能完整的编码 Agent，支持文件编辑、Shell、文件与网页检索、Skills、计划、目标、子代理和工作流。",
            ),
            (
                "PTC 模式",
                "具备标准模式的全部能力，并通过 Code Mode SDK 呈现工具，让模型用一个 TypeScript 程序组合多步操作。",
            ),
            (
                "极简模式",
                "仅提供持久 bash 与 str_replace_editor 的双工具编码 Agent。",
            ),
            (
                "创造模式",
                "用于创建自定义 Agent preset：具备标准模式的全部能力，并提供运行时检查、插件实验和 preset 创作指导。",
            ),
        ];

        div()
            .relative()
            // Anchor chip button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .h(px(28.0))
                    .rounded(px(16.0))
                    .hover(|s| s.text_color(text_primary()))
                    .cursor_pointer()
                    .text_size(px(13.0))
                    .text_color(text_muted())
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child(icons::agent_preset(16.0, text_muted()))
                    .child(current.clone())
                    .child(icons::chevron_down(14.0, text_faint())),
            )
            // Floating preset menu
            .when(is_open, |this| {
                this.child(deferred(
                    div()
                        .absolute()
                        .top(px(32.0))
                        .left(px(0.0))
                        .w(px(320.0))
                        .rounded_xl()
                        .bg(bg_menu())
                        .border_1()
                        .border_color(border_menu())
                        .shadow_lg()
                        .p_1p5()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(presets.into_iter().map(|(name, desc)| {
                            let name = name.to_string();
                            let selected = name == current;
                            let closure_name = name.clone();
                            let handle_select = cx.listener(move |this, _, _, cx| {
                                this.set_preset(&closure_name, cx);
                            });
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .px_2()
                                .py_2()
                                .rounded(px(10.0))
                                .hover(|s| s.bg(bg_menu_item_hover()))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_select)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(text_primary())
                                                .child(name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(text_faint())
                                                .child(desc),
                                        ),
                                )
                                .when(selected, |this| {
                                    this.child(icons::check(16.0, text_primary()))
                                })
                        })),
                ))
            })
    }
}
