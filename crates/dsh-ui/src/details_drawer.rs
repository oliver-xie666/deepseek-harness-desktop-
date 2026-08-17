use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct DetailsDrawer {
    pub is_open: bool,
    pub tool_name: String,
    pub duration_ms: u64,
    pub args_json: String,
    pub output_raw: String,
}

impl Default for DetailsDrawer {
    fn default() -> Self {
        Self::new()
    }
}

impl DetailsDrawer {
    pub fn new() -> Self {
        Self {
            is_open: false,
            tool_name: "grep_search".into(),
            duration_ms: 100,
            args_json: "{\n  \"query\": \"greet\",\n  \"path\": \"crates/\"\n}".into(),
            output_raw: "crates/dsh-core/src/lib.rs:84\n1 match found.".into(),
        }
    }

    pub fn open_tool(&mut self, name: &str, duration_ms: u64, args: &str, output: &str, cx: &mut Context<Self>) {
        self.tool_name = name.to_string();
        self.duration_ms = duration_ms;
        self.args_json = args.to_string();
        self.output_raw = output.to_string();
        self.is_open = true;
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        cx.notify();
    }
}

impl Render for DetailsDrawer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let handle_close = cx.listener(|this, _, _, cx| {
            this.close(cx);
        });

        div()
            .w_80()
            .h_full()
            .bg(rgb(0x15171b))
            .border_l_1()
            .border_color(rgb(0x23262d))
            .flex()
            .flex_col()
            .overflow_hidden()
            // Drawer Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(0x23262d))
                    .bg(rgb(0x13151b))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_sm().child("🔧"))
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .child(format!("{} ({}ms)", self.tool_name, self.duration_ms)),
                            ),
                    )
                    .child(
                        div()
                            .size_6()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .hover(|s| s.bg(rgb(0x1f2228)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_close)
                            .child("✕"),
                    ),
            )
            // Drawer Body: Input Parameters JSON
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .p_3()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x61666b))
                                    .child("PARAMETERS"),
                            )
                            .child(
                                div()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(rgb(0x0f1115))
                                    .border_1()
                                    .border_color(rgb(0x282c34))
                                    .text_xs()
                                    .text_color(rgb(0x4ade80))
                                    .child(self.args_json.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x61666b))
                                    .child("OUTPUT"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(rgb(0x0f1115))
                                    .border_1()
                                    .border_color(rgb(0x282c34))
                                    .text_xs()
                                    .text_color(rgb(0xe4e4e7))
                                    .child(self.output_raw.clone()),
                            ),
                    ),
            )
    }
}
