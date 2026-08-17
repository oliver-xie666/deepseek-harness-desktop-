use gpui::{div, prelude::*, rgb, Context, FontWeight, IntoElement, Window};

pub struct TitleBar {
    pub workspace_name: String,
    pub active_model: String,
    pub is_connected: bool,
}

impl TitleBar {
    pub fn new(workspace: &str, model: &str, is_connected: bool) -> Self {
        Self {
            workspace_name: workspace.to_string(),
            active_model: model.to_string(),
            is_connected,
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status_color = if self.is_connected {
            rgb(0x22c55e)
        } else {
            rgb(0xef4444)
        };

        div()
            .h_10()
            .w_full()
            .bg(rgb(0x13151b))
            .border_b_1()
            .border_color(rgb(0x23262d))
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            // Left: DeepSeek Brand & Workspace
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(div().text_base().child("🐳"))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x4176e6))
                            .child("DeepSeek Harness"),
                    )
                    .child(div().text_xs().text_color(rgb(0x61666b)).child("/"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .child(format!("📁 {}", self.workspace_name)),
                    ),
            )
            // Center: Draggable Window Region
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .cursor_default()
                    .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                        window.start_window_move();
                    }),
            )
            // Right: Model, Status & Window Controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    // Active Model Tag
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x1f2228))
                            .border_1()
                            .border_color(rgb(0x2c2c2e))
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xe4e4e7))
                            .child("🤖")
                            .child(self.active_model.clone()),
                    )
                    // Daemon Status
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .text_xs()
                            .text_color(rgb(0x979da6))
                            .child(div().size_2().rounded_full().bg(status_color))
                            .child(if self.is_connected { "Online" } else { "Disconnected" }),
                    )
                    // Divider
                    .child(div().w_px().h_4().bg(rgb(0x2c2c2e)))
                    // Window Control: Minimize & Close
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .size_6()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(0x979da6))
                                    .hover(|s| s.bg(rgb(0x282c34)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .child("─"),
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
                                    .hover(|s| s.bg(rgb(0xef4444)).text_color(rgb(0xffffff)))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                                        window.remove_window();
                                    })
                                    .child("✕"),
                            ),
                    ),
            )
    }
}
