use crate::icons;
use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, ScrollHandle, Window};

pub struct DetailsDrawer {
    pub is_open: bool,
    pub tool_name: String,
    pub duration_ms: u64,
    pub args_json: String,
    pub output_raw: String,
    content_scroll_handle: ScrollHandle,
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
            content_scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn open_tool(
        &mut self,
        name: &str,
        duration_ms: u64,
        args: &str,
        output: &str,
        cx: &mut Context<Self>,
    ) {
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
        let args_to_copy = self.args_json.clone();
        let handle_copy_args = cx.listener(move |_this, _, _, cx| {
            cx.write_to_clipboard(args_to_copy.clone().into());
        });
        let output_to_copy = self.output_raw.clone();
        let handle_copy_output = cx.listener(move |_this, _, _, cx| {
            cx.write_to_clipboard(output_to_copy.clone().into());
        });

        let content_scroll_handle = self.content_scroll_handle.clone();

        div()
            .w_80()
            .h_full()
            .bg(rgb(0xffffff))
            .border_l_1()
            .border_color(rgb(0xe5e7eb))
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
                    .border_color(rgb(0xe5e7eb))
                    .bg(rgb(0xf9fafb))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(icons::wrench(14.0, rgb(0x61666b)))
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_xs()
                                    .text_color(rgb(0x0f1115))
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
                            .text_color(rgb(0x81858c))
                            .hover(|s| s.bg(rgb(0xf1f3f5)).text_color(rgb(0x0f1115)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, handle_close)
                            .child(icons::close(14.0, rgb(0x81858c))),
                    ),
            )
            // Drawer Body: Input Parameters JSON
            .child(
                div()
                    .flex_1()
                    .id("details-content")
                    .overflow_y_scroll()
                    .track_scroll(&content_scroll_handle)
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
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x61666b))
                                            .child("PARAMETERS"),
                                    )
                                    .child(copy_button(handle_copy_args)),
                            )
                            .child(
                                div()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(rgb(0xf5f6f8))
                                    .border_1()
                                    .border_color(rgb(0xe1e5eb))
                                    .text_xs()
                                    .text_color(rgb(0x16a34a))
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
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x61666b))
                                            .child("OUTPUT"),
                                    )
                                    .child(copy_button(handle_copy_output)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .p_2p5()
                                    .rounded_lg()
                                    .bg(rgb(0xf5f6f8))
                                    .border_1()
                                    .border_color(rgb(0xe1e5eb))
                                    .text_xs()
                                    .text_color(rgb(0x3f454d))
                                    .child(self.output_raw.clone()),
                            ),
                    ),
            )
    }
}

fn copy_button(
    handler: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .rounded_md()
        .text_xs()
        .text_color(rgb(0x61666b))
        .hover(|style| style.bg(rgb(0xf1f3f5)).text_color(rgb(0x0f1115)))
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, handler)
        .child(icons::copy(12.0, rgb(0x61666b)))
        .child("复制")
}

#[cfg(test)]
mod tests {
    #[test]
    fn details_body_uses_gpui_vertical_scroll_container() {
        let source = include_str!("details_drawer.rs");
        assert!(
            source.contains(".overflow_y_scroll()"),
            "details drawer content must use GPUI's vertical scroll container"
        );
    }
}
