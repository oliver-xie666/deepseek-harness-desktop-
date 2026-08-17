use gpui::{
    div, prelude::*, rgb, Context, FontWeight, IntoElement, Window, WindowControlArea,
};

pub struct TitleBar {
    pub workspace_name: String,
    pub is_sidebar_open: bool,
}

impl TitleBar {
    pub fn new(workspace: &str) -> Self {
        Self {
            workspace_name: workspace.to_string(),
            is_sidebar_open: true,
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_10()
            .w_full()
            .bg(rgb(0x0d0f12))
            .border_b_1()
            .border_color(rgb(0x1a1c22))
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .window_control_area(WindowControlArea::Drag)
            // Left: deepseek [HARNESS] + ◫
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(div().text_sm().child("🐳"))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child("deepseek"),
                    )
                    .child(
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(0x4a4d56))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xd4d4d8))
                            .child("HARNESS"),
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
                            .hover(|s| s.bg(rgb(0x1a1c22)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .child("◫"),
                    ),
            )
            // Right: 🔧, ─, ▢, ✕
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    // Settings Wrench icon
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .hover(|s| s.bg(rgb(0x1a1c22)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .child("🔧"),
                    )
                    // Minimize
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .hover(|s| s.bg(rgb(0x1a1c22)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .child("─"),
                    )
                    // Maximize
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .hover(|s| s.bg(rgb(0x1a1c22)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.zoom_window();
                            })
                            .child("▢"),
                    )
                    // Close
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .hover(|s| s.bg(rgb(0xef4444)).text_color(rgb(0xffffff)))
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                window.remove_window();
                            })
                            .child("✕"),
                    ),
            )
    }
}
