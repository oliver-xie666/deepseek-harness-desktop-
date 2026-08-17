use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct WorkspaceSelector {
    pub is_open: bool,
    pub current_workspace: String,
}

impl WorkspaceSelector {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_workspace: "选择工作区".into(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn set_workspace(&mut self, ws: &str, cx: &mut Context<Self>) {
        self.current_workspace = ws.to_string();
        self.is_open = false;
        cx.notify();
    }
}

impl Render for WorkspaceSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.is_open;
        let current = self.current_workspace.clone();

        let handle_toggle = cx.listener(|this, _, _, cx| {
            this.toggle(cx);
        });

        let workspaces = vec![
            ("📁 deepseek-harness-desktop", "D:\\rust\\deepseek-harness-desktop", "deepseek-harness-desktop"),
            ("📁 zed-fluid", "D:\\rust\\zed-fluid", "zed-fluid"),
            ("➕ 浏览本地目录...", "打开系统文件管理器选择新的工作区", "选择工作区"),
        ];

        div()
            .relative()
            // Anchor Chip Button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1()
                    .rounded_lg()
                    .bg(if is_open { rgb(0x212328) } else { rgb(0x0d0f12) })
                    .hover(|s| s.bg(rgb(0x1e2025)).text_color(rgb(0xffffff)))
                    .cursor_pointer()
                    .text_xs()
                    .text_color(rgb(0x979da6))
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child("📁")
                    .child(current)
                    .child("∨"),
            )
            // Floating Dropdown Panel
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(gpui::px(28.0))
                        .left(gpui::px(0.0))
                        .w(gpui::px(280.0))
                        .rounded_xl()
                        .bg(rgb(0x181a20))
                        .border_1()
                        .border_color(rgb(0x2a2d35))
                        .shadow_lg()
                        .p_1p5()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(workspaces.into_iter().map(|(name, desc, ws_key)| {
                            let key = ws_key.to_string();
                            let handle_select = cx.listener(move |this, _, _, cx| {
                                this.set_workspace(&key, cx);
                            });

                            div()
                                .p_2()
                                .rounded_lg()
                                .hover(|s| s.bg(rgb(0x212328)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_select)
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .child(name),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .child(desc),
                                )
                        })),
                )
            })
    }
}

pub struct ModelModeSelector {
    pub is_open: bool,
    pub current_mode: String,
}

impl ModelModeSelector {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_mode: "标准模式".into(),
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn set_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        self.current_mode = mode.to_string();
        self.is_open = false;
        cx.notify();
    }
}

impl Render for ModelModeSelector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_open = self.is_open;
        let current = self.current_mode.clone();

        let handle_toggle = cx.listener(|this, _, _, cx| {
            this.toggle(cx);
        });

        let modes = vec![
            ("Ꮬ 标准模式", "标准推理与通用编码", "标准模式"),
            ("🧠 深度思考 (R1)", "DeepSeek-R1 强化学习高强度推理", "深度思考 (R1)"),
            ("⚡ 快速模式 (V3)", "DeepSeek-V3 120 FPS 极速响应", "快速模式 (V3)"),
        ];

        div()
            .relative()
            // Anchor Chip Button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1()
                    .rounded_lg()
                    .bg(if is_open { rgb(0x212328) } else { rgb(0x0d0f12) })
                    .hover(|s| s.bg(rgb(0x1e2025)).text_color(rgb(0xffffff)))
                    .cursor_pointer()
                    .text_xs()
                    .text_color(rgb(0x979da6))
                    .on_mouse_down(gpui::MouseButton::Left, handle_toggle)
                    .child("Ꮬ")
                    .child(current)
                    .child("∨"),
            )
            // Floating Dropdown Panel
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(gpui::px(28.0))
                        .left(gpui::px(0.0))
                        .w(gpui::px(260.0))
                        .rounded_xl()
                        .bg(rgb(0x181a20))
                        .border_1()
                        .border_color(rgb(0x2a2d35))
                        .shadow_lg()
                        .p_1p5()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(modes.into_iter().map(|(name, desc, mode_key)| {
                            let key = mode_key.to_string();
                            let handle_select = cx.listener(move |this, _, _, cx| {
                                this.set_mode(&key, cx);
                            });

                            div()
                                .p_2()
                                .rounded_lg()
                                .hover(|s| s.bg(rgb(0x212328)))
                                .cursor_pointer()
                                .on_mouse_down(gpui::MouseButton::Left, handle_select)
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .child(name),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x61666b))
                                        .child(desc),
                                )
                        })),
                )
            })
    }
}
